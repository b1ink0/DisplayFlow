use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub computer: String,
    pub device_id: String,
    pub monitor_id: String,
    pub input: u16,
    pub label: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub id: String,
    pub name: String,
    pub attachments: Vec<Attachment>,
    pub active_computer: Option<String>,
    pub enabled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub certificate: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MountedModule {
    pub manifest: super::modules::Manifest,
    pub path: String,
    pub executable_hash: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub revision: u64,
    pub schema_version: u32,
    pub device_id: String,
    pub computer_name: String,
    pub paused: bool,
    pub deskflow_log: String,
    pub network_enabled: bool,
    pub listen_port: u16,
    pub routes: Vec<Route>,
    pub peers: Vec<Peer>,
    pub modules: Vec<MountedModule>,
    pub manual_monitors: Vec<Monitor>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            revision: 0,
            schema_version: 1,
            device_id: uuid::Uuid::new_v4().to_string(),
            computer_name: hostname::get().unwrap_or_default().to_string_lossy().into(),
            paused: true,
            deskflow_log: String::new(),
            network_enabled: false,
            listen_port: 24862,
            routes: vec![],
            peers: vec![],
            modules: vec![],
            manual_monitors: vec![],
        }
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub at: u64,
    pub level: String,
    pub message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub settings: Settings,
    pub monitors: Vec<Monitor>,
    pub activity: Vec<Activity>,
    pub platform: String,
    pub agent_status: String,
    pub log_status: String,
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
