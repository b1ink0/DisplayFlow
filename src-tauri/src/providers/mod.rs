use crate::{
    model::{InputReading, Monitor},
    modules,
    store::Shared,
};
use serde_json::{json, Value};
#[cfg(windows)]
mod windows;

pub async fn enumerate(state: &Shared) -> Result<Vec<Monitor>, String> {
    #[allow(unused_mut)]
    let mut monitors = state.config().manual_monitors;
    #[cfg(windows)]
    {
        monitors.extend(
            tokio::task::spawn_blocking(windows::enumerate)
                .await
                .map_err(|e| e.to_string())??,
        );
    }
    monitors.extend(modules::enumerate(state).await);
    Ok(monitors)
}
pub async fn set_input(state: &Shared, monitor: &str, input: u16) -> Result<String, String> {
    if let Some(id) = monitor.strip_prefix("module:") {
        let (module_id, target) = id.split_once(':').ok_or("Invalid module monitor ID")?;
        let module = state
            .config()
            .modules
            .into_iter()
            .find(|m| m.manifest.id == module_id)
            .ok_or("Module not mounted")?;
        let result = modules::request(
            &module,
            "setInput",
            json!({"monitorId": target, "input": input}),
        )
        .await?;
        return Ok(result["message"]
            .as_str()
            .unwrap_or("Request sent; verify the picture")
            .into());
    }
    #[cfg(windows)]
    if monitor.starts_with("ddc:") {
        let target = monitor.to_string();
        tokio::task::spawn_blocking(move || windows::set_input(&target, input))
            .await
            .map_err(|e| e.to_string())??;
        return Ok("DDC request sent; picture is not verified".into());
    }
    #[cfg(target_os = "macos")]
    if let Some(name) = monitor.strip_prefix("betterdisplay:") {
        let output = tokio::process::Command::new(
            "/Applications/BetterDisplay.app/Contents/MacOS/BetterDisplay",
        )
        .args([
            "set",
            &format!("-name={name}"),
            "-feature=ddc",
            "-vcp=0x60",
            &format!("-value={input}"),
        ])
        .kill_on_drop(true)
        .output();
        let output = tokio::time::timeout(std::time::Duration::from_secs(8), output)
            .await
            .map_err(|_| "BetterDisplay timed out; outcome unknown")?
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("BetterDisplay declined input selection".into());
        }
        return Ok("BetterDisplay request sent; picture is not verified".into());
    }
    Err("No provider is available for this monitor on this computer".into())
}
pub async fn get_input(state: &Shared, monitor: &str) -> Result<InputReading, String> {
    if let Some(id) = monitor.strip_prefix("module:") {
        let (module_id, target) = id.split_once(':').ok_or("Invalid module monitor ID")?;
        let module = state
            .config()
            .modules
            .into_iter()
            .find(|m| m.manifest.id == module_id)
            .ok_or("Module not mounted")?;
        let result = modules::request(&module, "getInput", json!({"monitorId":target})).await?;
        let reading: InputReading =
            serde_json::from_value(result).map_err(|e| format!("Invalid input reading: {e}"))?;
        if reading.input.is_none() && !reading.active_connection {
            return Err("Module could not determine the visible input".into());
        }
        return Ok(reading);
    }
    #[cfg(windows)]
    if monitor.starts_with("ddc:") {
        let target = monitor.to_string();
        let input = tokio::task::spawn_blocking(move || windows::get_input(&target))
            .await
            .map_err(|e| e.to_string())??;
        return Ok(InputReading {
            input: Some(input),
            active_connection: false,
        });
    }
    #[cfg(target_os = "macos")]
    if let Some(name) = monitor.strip_prefix("betterdisplay:") {
        let output = tokio::process::Command::new(
            "/Applications/BetterDisplay.app/Contents/MacOS/BetterDisplay",
        )
        .args([
            "get",
            &format!("-name={name}"),
            "-feature=ddc",
            "-vcp=inputSelect",
            "-value",
        ])
        .kill_on_drop(true)
        .output();
        let output = tokio::time::timeout(std::time::Duration::from_secs(5), output)
            .await
            .map_err(|_| "BetterDisplay input read timed out")?
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("BetterDisplay could not read the current input".into());
        }
        let value = String::from_utf8(output.stdout).map_err(|_| "Invalid BetterDisplay reply")?;
        let value = value.trim();
        let input = if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            u16::from_str_radix(hex, 16)
        } else {
            value.parse::<u16>()
        }
        .map_err(|_| "BetterDisplay did not return an integer input value")?;
        return Ok(InputReading {
            input: Some(input),
            active_connection: false,
        });
    }
    Err("This provider does not support reading the current input".into())
}
pub async fn module_action(
    state: &Shared,
    monitor: &str,
    operation: &str,
) -> Result<Value, String> {
    let (id, target) = monitor
        .strip_prefix("module:")
        .ok_or("Select an external module monitor")?
        .split_once(':')
        .ok_or("Invalid monitor ID")?;
    let module = state
        .config()
        .modules
        .into_iter()
        .find(|m| m.manifest.id == id)
        .ok_or("Module not mounted")?;
    modules::request(
        &module,
        operation,
        json!({"monitorId": target, "confirmed": operation == "recover"}),
    )
    .await
}
