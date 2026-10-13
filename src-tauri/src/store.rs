use crate::model::*;
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct Runtime {
    pub settings: Mutex<Settings>,
    pub activity: Mutex<VecDeque<Activity>>,
    pub path: PathBuf,
    pub switch_lock: tokio::sync::Mutex<()>,
    pub setup_lock: tokio::sync::Mutex<()>,
    pub agent_status: Mutex<String>,
    pub log_status: Mutex<String>,
    pub invitation: Mutex<Option<(String, u64)>>,
    pub monitors: Mutex<Vec<Monitor>>,
    pub save_lock: Mutex<()>,
    pub identity_lock: Mutex<()>,
    pub credentials: Mutex<Option<crate::credentials::Vault>>,
    pub detection_lock: tokio::sync::Mutex<()>,
    pub route_detection: Mutex<std::collections::HashMap<String, DetectionStatus>>,
    pub route_sync: Mutex<std::collections::HashMap<String, RouteSyncStatus>>,
    pub shared_versions: Mutex<std::collections::HashMap<String, (String, u64)>>,
    pub instance_id: String,
    pub publish_lock: tokio::sync::Mutex<()>,
}
pub type Shared = Arc<Runtime>;
impl Runtime {
    pub fn load(directory: PathBuf) -> Result<Shared, String> {
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let path = directory.join("settings.json");
        let settings = if path.exists() {
            serde_json::from_slice::<Settings>(&std::fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("Settings need attention; preserved original file: {e}"))?
        } else {
            Settings::default()
        };
        if settings.schema_version != 1 {
            return Err("Unsupported settings version".into());
        }
        let mut settings = settings;
        // Reconfirm visible inputs after restart, including intervening manual changes.
        for route in &mut settings.routes {
            route.active_computer = None;
            if route.coordinator_device_id.is_none() && !settings.deskflow_log.is_empty() {
                route.coordinator_device_id = Some(settings.device_id.clone());
            }
        }
        Ok(Arc::new(Self {
            settings: Mutex::new(settings),
            path,
            activity: Mutex::new(VecDeque::new()),
            switch_lock: tokio::sync::Mutex::new(()),
            setup_lock: tokio::sync::Mutex::new(()),
            agent_status: Mutex::new("Disabled".into()),
            log_status: Mutex::new("Choose the server log".into()),
            invitation: Mutex::new(None),
            monitors: Mutex::new(vec![]),
            save_lock: Mutex::new(()),
            identity_lock: Mutex::new(()),
            credentials: Mutex::new(None),
            detection_lock: tokio::sync::Mutex::new(()),
            route_detection: Mutex::new(std::collections::HashMap::new()),
            route_sync: Mutex::new(std::collections::HashMap::new()),
            shared_versions: Mutex::new(std::collections::HashMap::new()),
            instance_id: uuid::Uuid::new_v4().to_string(),
            publish_lock: tokio::sync::Mutex::new(()),
        }))
    }
    pub fn save(&self) -> Result<(), String> {
        let _guard = self.save_lock.lock().unwrap();
        let bytes = {
            let mut settings = self.settings.lock().unwrap();
            settings.revision = settings.revision.saturating_add(1);
            serde_json::to_vec_pretty(&*settings).map_err(|e| e.to_string())?
        };
        let staging = self.path.with_extension("tmp");
        std::fs::write(&staging, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&staging, &self.path).map_err(|e| e.to_string())
    }
    pub fn log(&self, level: &str, message: impl Into<String>) {
        let mut activity = self.activity.lock().unwrap();
        activity.push_front(Activity {
            at: now(),
            level: level.into(),
            message: message.into(),
        });
        activity.truncate(150);
    }
    pub fn config(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }
}
