use crate::{
    model::{Monitor, MountedModule},
    store::Shared,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub api_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_timeout_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<ModuleUi>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModuleUi {
    #[serde(default)]
    pub panels: Vec<ModulePanel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoveryUi>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryUi {
    pub label: String,
    pub description: String,
    pub confirmation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModulePanel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub actions: Vec<PanelAction>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PanelAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub requires_confirmation: bool,
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
}
fn executable(root: &Path, manifest: &Manifest) -> Result<PathBuf, String> {
    let path = root
        .join(&manifest.executable)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(root) || !path.is_file() {
        return Err("Executable must be a file inside the module directory".into());
    }
    Ok(path)
}
pub fn inspect(path: &str) -> Result<MountedModule, String> {
    let root = Path::new(path).canonicalize().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(root.join("displayflow-module.json")).map_err(|e| e.to_string())?;
    if bytes.len() > 16_384 {
        return Err("Manifest exceeds 16 KiB".into());
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if manifest.api_version != 1 || !valid_id(&manifest.id) {
        return Err("Invalid module ID or unsupported API version".into());
    }
    if manifest.capabilities.iter().any(|c| {
        ![
            "enumerate",
            "inspect",
            "getInput",
            "setInput",
            "recover",
            "action",
        ]
        .contains(&c.as_str())
    }) {
        return Err("Unknown module capability".into());
    }
    if [manifest.timeout_seconds, manifest.recovery_timeout_seconds]
        .into_iter()
        .flatten()
        .any(|n| !(1..=60).contains(&n))
    {
        return Err("Module timeouts must be from 1 to 60 seconds".into());
    }
    if let Some(ui) = &manifest.ui {
        if let Some(recovery) = &ui.recovery {
            if recovery.label.is_empty()
                || recovery.label.len() > 64
                || recovery.description.len() > 512
                || recovery.confirmation.is_empty()
                || recovery.confirmation.len() > 512
            {
                return Err("Invalid recovery UI description".into());
            }
        }
        let mut ids = std::collections::HashSet::new();
        if ui.panels.len() > 8 {
            return Err("A module can contribute at most eight panels".into());
        }
        for panel in &ui.panels {
            if !valid_id(&panel.id)
                || !ids.insert(&panel.id)
                || panel.title.is_empty()
                || panel.title.len() > 64
                || panel.description.len() > 512
                || panel.actions.len() > 12
            {
                return Err("Invalid module panel".into());
            }
            let mut actions = std::collections::HashSet::new();
            for action in &panel.actions {
                if !valid_id(&action.id)
                    || !actions.insert(&action.id)
                    || action.label.is_empty()
                    || action.label.len() > 64
                    || action.description.len() > 512
                {
                    return Err("Invalid panel action".into());
                }
            }
            if !panel.actions.is_empty() && !manifest.capabilities.iter().any(|c| c == "action") {
                return Err("Panels with actions require the action capability".into());
            }
        }
    }
    let hash = format!(
        "{:x}",
        Sha256::digest(std::fs::read(executable(&root, &manifest)?).map_err(|e| e.to_string())?)
    );
    Ok(MountedModule {
        path: root.to_string_lossy().into(),
        manifest,
        executable_hash: hash,
    })
}
pub async fn request(
    module: &MountedModule,
    operation: &str,
    mut payload: Value,
) -> Result<Value, String> {
    if !module.manifest.capabilities.iter().any(|c| c == operation) {
        return Err("Module does not support this operation".into());
    }
    let current = inspect(&module.path)?;
    if current.executable_hash != module.executable_hash
        || serde_json::to_string(&current.manifest).unwrap()
            != serde_json::to_string(&module.manifest).unwrap()
    {
        return Err("Module changed; review and mount it again".into());
    }
    payload["apiVersion"] = json!(1);
    payload["operation"] = json!(operation);
    let path = executable(Path::new(&module.path), &module.manifest)?;
    let mut command = Command::new(path);
    command
        .args(&module.manifest.arguments)
        .current_dir(&module.path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let mut input = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    input.push(b'\n');
    let mut stdin = child.stdin.take().ok_or("Module stdin missing")?;
    stdin.write_all(&input).await.map_err(|e| e.to_string())?;
    drop(stdin);
    let mut stdout = child
        .stdout
        .take()
        .ok_or("Module stdout missing")?
        .take(65_537);
    let mut output = Vec::new();
    let work = async {
        stdout
            .read_to_end(&mut output)
            .await
            .map_err(|e| e.to_string())?;
        if output.len() > 65_536 {
            return Err("Module output exceeds 64 KiB".into());
        }
        let status = child.wait().await.map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("Module exited with {status}"));
        }
        let result: Value =
            serde_json::from_slice(&output).map_err(|e| format!("Invalid module response: {e}"))?;
        if result["ok"] != true {
            return Err(result["message"]
                .as_str()
                .unwrap_or("Module declined request")
                .into());
        }
        Ok(result)
    };
    let timeout = if operation == "getInput" {
        Some(module.manifest.timeout_seconds.unwrap_or(5).min(5))
    } else if operation == "recover" {
        module
            .manifest
            .recovery_timeout_seconds
            .or(module.manifest.timeout_seconds)
    } else {
        module.manifest.timeout_seconds
    }
    .unwrap_or(15);
    tokio::time::timeout(std::time::Duration::from_secs(timeout), work)
        .await
        .map_err(|_| "Module timed out; request outcome is unknown".to_string())?
}
pub async fn enumerate(state: &Shared) -> Vec<Monitor> {
    let mut monitors = vec![];
    for module in state.config().modules {
        if !module
            .manifest
            .capabilities
            .iter()
            .any(|c| c == "enumerate")
        {
            continue;
        }
        match request(&module, "enumerate", json!({})).await {
            Ok(result) => {
                match serde_json::from_value::<Vec<Monitor>>(result["monitors"].clone()) {
                    Ok(items) => monitors.extend(items.into_iter().map(|mut m| {
                        let mut values = std::collections::HashSet::new();
                        if m.input_presets.len() > 32
                            || m.input_presets.iter().any(|p| {
                                p.label.trim().is_empty()
                                    || p.label.len() > 64
                                    || !values.insert(p.value)
                            })
                        {
                            state.log(
                                "error",
                                format!("{}: invalid input presets", module.manifest.name),
                            );
                            m.input_presets.clear();
                        }
                        m.id = format!("module:{}:{}", module.manifest.id, m.id);
                        m.provider = module.manifest.id.clone();
                        m
                    })),
                    Err(e) => {
                        state.log("error", format!("{} inventory: {e}", module.manifest.name))
                    }
                }
            }
            Err(e) => state.log("error", format!("{}: {e}", module.manifest.name)),
        }
    }
    monitors
}
