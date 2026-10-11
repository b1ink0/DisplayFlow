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
pub fn secret_set(name: &str, value: &str) -> Result<(), String> {
    keyring::Entry::new("dev.displayflow.desktop", name)
        .map_err(|e| e.to_string())?
        .set_password(value)
        .map_err(|e| format!("OS credential storage: {e}"))
}
pub fn secret_get(name: &str) -> Result<String, String> {
    keyring::Entry::new("dev.displayflow.desktop", name)
        .map_err(|e| e.to_string())?
        .get_password()
        .map_err(|e| format!("OS credential storage: {e}"))
}
pub fn secret_remove(name: &str) {
    if let Ok(entry) = keyring::Entry::new("dev.displayflow.desktop", name) {
        let _ = entry.delete_credential();
    }
}
pub fn secret_optional(name: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new("dev.displayflow.desktop", name).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("OS credential storage: {e}")),
    }
}
