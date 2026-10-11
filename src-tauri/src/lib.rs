mod access;
mod deskflow;
mod lifecycle;
mod model;
mod modules;
mod network;
mod providers;
mod routing;
mod setup;
mod startup;
mod store;

use model::*;
use serde_json::Value;
use store::Shared;
use tauri::{Manager, State};

#[tauri::command]
async fn snapshot(state: State<'_, Shared>, refresh: bool) -> Result<Snapshot, String> {
    if refresh {
        let monitors = providers::enumerate(&state).await?;
        *state.monitors.lock().unwrap() = monitors;
    }
    Ok(Snapshot {
        settings: state.config(),
        monitors: state.monitors.lock().unwrap().clone(),
        activity: state.activity.lock().unwrap().iter().cloned().collect(),
        platform: std::env::consts::OS.into(),
        agent_status: state.agent_status.lock().unwrap().clone(),
        log_status: state.log_status.lock().unwrap().clone(),
    })
}
#[tauri::command]
async fn save_settings(state: State<'_, Shared>, settings: Settings) -> Result<(), String> {
    let _guard = state.switch_lock.lock().await;
    if settings.computer_name.trim().is_empty() || settings.computer_name.len() > 128 {
        return Err("Enter the exact Deskflow computer name (1–128 characters)".into());
    }
    if settings.listen_port < 1024 {
        return Err("Choose a port from 1024 to 65535".into());
    }
    if settings.routes.len() > 64 || settings.manual_monitors.len() > 64 {
        return Err("Too many routes or monitors".into());
    }
    let mut route_ids = std::collections::HashSet::new();
    for route in &settings.routes {
        if route.id.is_empty()
            || route.id.len() > 128
            || !route_ids.insert(&route.id)
            || route.name.trim().is_empty()
            || route.name.len() > 128
        {
            return Err("Routes need unique IDs and names of 1–128 characters".into());
        }
        if route.attachments.len() < 2 || route.attachments.len() > 16 {
            return Err("Each route needs 2–16 attachments".into());
        }
        let mut names = std::collections::HashSet::new();
        for attachment in &route.attachments {
            if attachment.computer.trim().is_empty()
                || attachment.computer.len() > 128
                || attachment.device_id.is_empty()
                || attachment.monitor_id.len() > 512
                || attachment.monitor_id.is_empty()
                || !names.insert(&attachment.computer)
            {
                return Err("Every attachment needs a unique computer name and monitor ID".into());
            }
        }
        if route
            .active_computer
            .as_ref()
            .is_some_and(|v| !names.contains(v))
        {
            return Err("Active computer is not assigned to this route".into());
        }
    }
    if settings.manual_monitors.iter().any(|m| {
        !m.id.starts_with("betterdisplay:") || m.name.trim().is_empty() || m.name.len() > 128
    }) {
        return Err(
            "Registered displays need an exact BetterDisplay name (1–128 characters)".into(),
        );
    }
    {
        let mut current = state.settings.lock().unwrap();
        if current.revision != settings.revision {
            return Err("Workspace changed while editing. Refresh and save again.".into());
        }
        current.computer_name = settings.computer_name.trim().into();
        current.paused = settings.paused;
        current.deskflow_log = settings.deskflow_log;
        current.network_enabled = settings.network_enabled;
        current.listen_port = settings.listen_port;
        current.routes = settings.routes;
        current.manual_monitors = settings.manual_monitors;
    }
    state.save()
}
#[tauri::command]
fn inspect_module(path: String) -> Result<MountedModule, String> {
    modules::inspect(&path)
}
#[tauri::command]
async fn mount_module(
    state: State<'_, Shared>,
    reviewed: MountedModule,
    trusted: bool,
) -> Result<(), String> {
    if !trusted {
        return Err("Explicit executable trust is required".into());
    }
    let current = modules::inspect(&reviewed.path)?;
    if current.executable_hash != reviewed.executable_hash
        || serde_json::to_string(&current.manifest).unwrap()
            != serde_json::to_string(&reviewed.manifest).unwrap()
    {
        return Err("Module changed since review".into());
    }
    {
        let mut settings = state.settings.lock().unwrap();
        settings
            .modules
            .retain(|m| m.manifest.id != current.manifest.id);
        settings.modules.push(current);
    }
    state.save()?;
    state.log("info", "Module added.");
    Ok(())
}
#[tauri::command]
fn unmount_module(state: State<'_, Shared>, id: String) -> Result<(), String> {
    state
        .settings
        .lock()
        .unwrap()
        .modules
        .retain(|m| m.manifest.id != id);
    state.save()
}
#[tauri::command]
async fn module_action(
    state: State<'_, Shared>,
    monitor_id: String,
    operation: String,
    confirmed: bool,
) -> Result<Value, String> {
    if !["inspect", "recover"].contains(&operation.as_str()) {
        return Err("Unsupported action".into());
    }
    if operation == "recover" && !confirmed {
        return Err("Confirm that this computer's input is visible before recovery".into());
    }
    let _guard = state.switch_lock.lock().await;
    let result = providers::module_action(&state, &monitor_id, &operation).await?;
    state.log(
        "info",
        format!(
            "Module {operation}: {}",
            result["message"]
                .as_str()
                .unwrap_or("Complete; recovery state is unknown unless verified by the module")
        ),
    );
    Ok(result)
}
#[tauri::command]
async fn select_destination(
    state: State<'_, Shared>,
    route_id: String,
    destination: String,
) -> Result<(), String> {
    routing::switch(&state, &route_id, &destination).await
}
#[tauri::command]
async fn panel_action(
    state: State<'_, Shared>,
    module_id: String,
    panel_id: String,
    action_id: String,
    monitor_id: String,
    confirmed: bool,
) -> Result<Value, String> {
    let module = state
        .config()
        .modules
        .into_iter()
        .find(|m| m.manifest.id == module_id)
        .ok_or("Module is not mounted")?;
    let panel = module
        .manifest
        .ui
        .as_ref()
        .and_then(|ui| ui.panels.iter().find(|p| p.id == panel_id))
        .ok_or("Panel is not registered")?;
    let action = panel
        .actions
        .iter()
        .find(|a| a.id == action_id)
        .ok_or("Action is not registered")?;
    if action.requires_confirmation && !confirmed {
        return Err("This local action requires explicit confirmation".into());
    }
    let monitors = state.monitors.lock().unwrap().clone();
    let full_id = format!("module:{module_id}:{monitor_id}");
    if !monitors
        .iter()
        .any(|m| m.id == full_id && m.provider == module_id)
    {
        return Err("Choose a monitor reported by this module".into());
    }
    let _guard = state.switch_lock.lock().await;
    let result = modules::request(&module, "action", serde_json::json!({"panelId":panel_id,"actionId":action_id,"monitorId":monitor_id,"confirmed":confirmed})).await?;
    state.log(
        "info",
        format!(
            "{} / {}: {}",
            module.manifest.name,
            action.label,
            result["message"].as_str().unwrap_or("Complete")
        ),
    );
    Ok(result)
}
#[tauri::command]
fn create_invitation(state: State<'_, Shared>, address: String) -> Result<String, String> {
    network::invitation(&state, address)
}
#[tauri::command]
async fn pair_peer(
    state: State<'_, Shared>,
    invitation: String,
    address: String,
) -> Result<(), String> {
    network::pair(&state, invitation, address).await
}
#[tauri::command]
fn remove_peer(state: State<'_, Shared>, id: String) -> Result<(), String> {
    let own = state.config().device_id;
    state.settings.lock().unwrap().peers.retain(|p| p.id != id);
    store::secret_remove(&format!("inbound-{own}-{id}"));
    store::secret_remove(&format!("outbound-{own}-{id}"));
    state.save()
}
#[tauri::command]
async fn peer_inventory(state: State<'_, Shared>, id: String) -> Result<Vec<Monitor>, String> {
    let peer = state
        .config()
        .peers
        .into_iter()
        .find(|p| p.id == id)
        .ok_or("Peer is not paired")?;
    let response = network::remote(&state, &peer, "inventory", serde_json::json!({})).await?;
    serde_json::from_value(response["monitors"].clone()).map_err(|e| e.to_string())
}
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            setup::dispatch(app, &args);
            if !args.iter().any(|arg| arg == "--background") {
                lifecycle::reopen(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .setup(|app| {
            let state = store::Runtime::load(app.path().app_data_dir()?).map_err(std::io::Error::other)?;
            state.log("info", "Ready. Routing starts paused until configured; active inputs require confirmation.");
            app.manage(state.clone());
            lifecycle::install(app)?;
            if std::env::args().any(|arg| arg == "--background") {
                lifecycle::hide(app.handle());
            } else {
                lifecycle::reopen(app.handle());
            }
            tauri::async_runtime::spawn(network::serve(state.clone()));
            tauri::async_runtime::spawn(deskflow::watch(state));
            setup::dispatch(app.handle(), &std::env::args().collect::<Vec<_>>());
            Ok(())
        })
        .on_window_event(|window, event| { if let tauri::WindowEvent::CloseRequested { api, .. } = event { api.prevent_close(); lifecycle::hide(window.app_handle()); } })
        .invoke_handler(tauri::generate_handler![snapshot, save_settings, inspect_module, mount_module, unmount_module, module_action, panel_action, select_destination, create_invitation, pair_peer, remove_peer, peer_inventory, startup::startup_status, startup::set_startup, access::configure_firewall, access::open_network_permissions])
        .run(tauri::generate_context!()).expect("DisplayFlow failed to start");
}
