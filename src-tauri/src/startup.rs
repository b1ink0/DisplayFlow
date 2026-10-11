use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    enabled: bool,
    can_enable: bool,
}

#[tauri::command]
pub fn startup_status(app: AppHandle) -> Result<StartupStatus, String> {
    Ok(StartupStatus {
        enabled: app.autolaunch().is_enabled().map_err(|e| e.to_string())?,
        can_enable: !cfg!(debug_assertions),
    })
}

#[tauri::command]
pub fn set_startup(app: AppHandle, enabled: bool) -> Result<StartupStatus, String> {
    if enabled && cfg!(debug_assertions) {
        return Err("Install a release build before enabling Start at login.".into());
    }
    if enabled {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }
    startup_status(app)
}
