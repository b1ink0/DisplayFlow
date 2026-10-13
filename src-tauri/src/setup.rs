//! Local file-based provisioning with the UI's validation and trust checks.
use crate::{model::Route, store::Shared};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{io::Write, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    api_version: u32,
    operation: Operation,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum Operation {
    Status {
        #[serde(default)]
        refresh: bool,
    },
    Configure {
        #[serde(rename = "computerName")]
        computer_name: Option<String>,
        #[serde(rename = "deskflowLog")]
        deskflow_log: Option<String>,
        #[serde(rename = "networkEnabled")]
        network_enabled: Option<bool>,
        #[serde(rename = "listenPort")]
        listen_port: Option<u16>,
        routes: Option<Vec<Route>>,
    },
    Mount {
        path: String,
        #[serde(rename = "executableHash")]
        executable_hash: String,
        trusted: bool,
    },
    Invite {
        address: String,
    },
    Pair {
        invitation: String,
        address: String,
    },
    Inventory {
        #[serde(rename = "peerId")]
        peer_id: String,
    },
    Detect {
        #[serde(rename = "routeId")]
        route_id: String,
    },
    AutoDetect {
        #[serde(rename = "routeId")]
        route_id: String,
        enabled: bool,
    },
    Confirm {
        #[serde(rename = "routeId")]
        route_id: String,
        computer: String,
    },
    Startup {
        enabled: bool,
    },
    Firewall {
        enabled: bool,
    },
}

async fn execute(app: &AppHandle, request: Request) -> Result<Value, String> {
    if request.api_version != 1 {
        return Err("Unsupported setup API version".into());
    }
    let state = app.state::<Shared>();
    let _setup_guard = state.setup_lock.lock().await;
    match request.operation {
        Operation::Status { refresh } => {
            let snapshot = crate::snapshot(app.state(), refresh).await?;
            Ok(
                json!({"snapshot": snapshot, "startup": crate::startup::startup_status(app.clone())?}),
            )
        }
        Operation::Configure {
            computer_name,
            deskflow_log,
            network_enabled,
            listen_port,
            routes,
        } => {
            let mut settings = state.config();
            if let Some(value) = computer_name {
                settings.computer_name = value;
            }
            if let Some(value) = deskflow_log {
                settings.deskflow_log = value;
            }
            if let Some(value) = network_enabled {
                settings.network_enabled = value;
            }
            if let Some(value) = listen_port {
                settings.listen_port = value;
            }
            if let Some(value) = routes {
                settings.routes = value;
            }
            // Configuration changes require the user to reconfirm inputs and resume routing.
            settings.paused = true;
            for route in &mut settings.routes {
                route.active_computer = None;
            }
            crate::save_settings(app.state(), settings).await?;
            Ok(
                json!({"message": "Configuration saved; routing is paused and active inputs need confirmation"}),
            )
        }
        Operation::Mount {
            path,
            executable_hash,
            trusted,
        } => {
            if !PathBuf::from(&path).is_absolute() {
                return Err("Use an absolute module directory path".into());
            }
            let reviewed = crate::inspect_module(path)?;
            if reviewed.executable_hash != executable_hash {
                return Err("Module executable does not match the reviewed hash".into());
            }
            crate::mount_module(app.state(), reviewed, trusted).await?;
            Ok(json!({"message": "Reviewed external module mounted"}))
        }
        Operation::Invite { address } => {
            for _ in 0..40 {
                if *state.agent_status.lock().unwrap() == "Listening" {
                    break;
                }
                if !state.config().network_enabled {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Ok(json!({"invitation": crate::create_invitation(app.state(), address)?}))
        }
        Operation::Pair {
            invitation,
            address,
        } => {
            crate::pair_peer(app.state(), invitation, address).await?;
            Ok(json!({"message": "Peer paired"}))
        }
        Operation::Inventory { peer_id } => {
            Ok(json!({"monitors": crate::peer_inventory(app.state(), peer_id).await?}))
        }
        Operation::Detect { route_id } => {
            Ok(json!({"detection": crate::shared_routes::detect(&state, &route_id).await?}))
        }
        Operation::AutoDetect { route_id, enabled } => {
            crate::shared_routes::configure(&state, &route_id, enabled).await?;
            Ok(json!({"message":"Shared detection setting updated"}))
        }
        Operation::Confirm { route_id, computer } => {
            crate::shared_routes::confirm(&state, &route_id, &computer).await?;
            Ok(json!({"message":"Visible input confirmation shared"}))
        }
        Operation::Startup { enabled } => {
            Ok(json!({"startup": crate::startup::set_startup(app.clone(), enabled)?}))
        }
        Operation::Firewall { enabled } => {
            Ok(json!({"message": crate::access::configure_firewall(app.state(), enabled).await?}))
        }
    }
}

pub fn dispatch(app: &AppHandle, args: &[String]) {
    let Some(index) = args.iter().position(|arg| arg == "--setup-file") else {
        return;
    };
    let Some(path) = args.get(index + 1) else {
        app.state::<Shared>()
            .log("error", "--setup-file requires an absolute JSON path");
        return;
    };
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        app.state::<Shared>()
            .log("error", "Setup requires an absolute JSON path");
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result_path = path.with_extension("result.json");
        // Reserving the response file prevents duplicate request execution.
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut output = match options.open(result_path) {
            Ok(file) => file,
            Err(e) => {
                app.state::<Shared>()
                    .log("error", format!("Cannot reserve setup response: {e}"));
                return;
            }
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = output.set_permissions(std::fs::Permissions::from_mode(0o600)) {
                app.state::<Shared>()
                    .log("error", format!("Cannot protect setup response: {e}"));
                return;
            }
        }
        let work = async {
            let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            if meta.len() > 65_536 {
                return Err("Setup request exceeds 64 KiB".into());
            }
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.len() > 65_536 {
                return Err("Setup request exceeds 64 KiB".into());
            }
            let request = serde_json::from_slice::<Request>(&bytes).map_err(|e| e.to_string())?;
            execute(&app, request).await
        }
        .await;
        let result = match work {
            Ok(value) => json!({"ok": true, "result": value}),
            Err(error) => {
                app.state::<Shared>()
                    .log("error", format!("Local setup failed: {error}"));
                json!({"ok": false, "message": error})
            }
        };
        // Write invitation credentials only to the user's response file.
        if let Err(e) = output.write_all(serde_json::to_string_pretty(&result).unwrap().as_bytes())
        {
            app.state::<Shared>()
                .log("error", format!("Cannot write setup response: {e}"));
        }
    });
}
