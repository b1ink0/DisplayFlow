use crate::store::Shared;
use tauri::State;

#[tauri::command]
pub async fn configure_firewall(state: State<'_, Shared>, enabled: bool) -> Result<String, String> {
    #[cfg(windows)]
    {
        use base64::{engine::general_purpose::STANDARD, Engine};
        if cfg!(debug_assertions) {
            return Err("Install a release build before configuring firewall access".into());
        }
        let port = state.config().listen_port;
        if port < 1024 {
            return Err("Save a valid listen port first".into());
        }
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe = exe
            .to_str()
            .ok_or("Executable path is not valid Unicode")?
            .replace('\'', "''");
        let action = if enabled {
            format!("New-NetFirewallRule -Name 'DisplayFlow-LAN-Private' -DisplayName 'DisplayFlow private LAN' -Direction Inbound -Action Allow -Protocol TCP -LocalPort {port} -RemoteAddress LocalSubnet -Profile Private,Domain -Program '{exe}' | Out-Null")
        } else {
            String::new()
        };
        let elevated = format!("$ErrorActionPreference='Stop'; try {{ if(Get-NetFirewallRule -Name 'DisplayFlow-LAN-Private' -ErrorAction SilentlyContinue){{Remove-NetFirewallRule -Name 'DisplayFlow-LAN-Private'}}; {action}; exit 0 }} catch {{exit 1}}");
        let encode = |s: &str| {
            STANDARD.encode(
                s.encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .collect::<Vec<_>>(),
            )
        };
        let elevated = encode(&elevated);
        let launcher = format!("$ErrorActionPreference='Stop'; try {{ $p=Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -Verb RunAs -ArgumentList '-NoProfile -NonInteractive -EncodedCommand {elevated}' -WindowStyle Hidden -Wait -PassThru; exit $p.ExitCode }} catch {{ Write-Error $_; exit 2 }}");
        let shell = std::path::PathBuf::from(
            std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
        )
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut command = tokio::process::Command::new(shell);
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-EncodedCommand",
                &encode(&launcher),
            ])
            .kill_on_drop(true);
        command.creation_flags(0x0800_0000); // Hide only the helper console; Windows owns the UAC prompt.
        let output = tokio::time::timeout(std::time::Duration::from_secs(120), command.output()).await
            .map_err(|_| "Firewall setup timed out. Check Windows Firewall before trying again; the elevated request may still be pending.")?
            .map_err(|e| format!("Cannot start Windows firewall setup: {e}"))?;
        if !output.status.success() {
            return Err("Windows did not complete firewall setup. Approve the administrator prompt, or ask your administrator to allow DisplayFlow's saved TCP port on your private LAN. The app continues running without elevation.".into());
        }
        Ok(if enabled {
            format!("Firewall access configured for TCP {port} on the local subnet of Private and Domain networks. DisplayFlow still runs without administrator rights.")
        } else {
            "DisplayFlow's private-LAN firewall rule removed.".into()
        })
    }
    #[cfg(not(windows))]
    {
        let _ = (state, enabled);
        Err("Windows firewall setup is available on Windows only".into())
    }
}

#[tauri::command]
pub async fn open_network_permissions() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = tokio::process::Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_LocalNetwork")
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(
                "Open System Settings → Privacy & Security → Local Network and allow DisplayFlow"
                    .into(),
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Open your OS network privacy settings".into())
    }
}
