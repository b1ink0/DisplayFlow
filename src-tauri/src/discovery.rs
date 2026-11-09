use crate::store::Shared;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, UdpSocket as StdSocket},
    time::Duration,
};
use tokio::net::UdpSocket;

pub const PORT: u16 = 24863;
const QUERY: &[u8] = b"displayflow-discovery-v1";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
}

pub fn local_address(target: &str) -> Result<String, String> {
    let socket = StdSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    // UDP connect selects an interface without sending a packet.
    socket.connect((target, PORT)).map_err(|e| e.to_string())?;
    let ip = socket.local_addr().map_err(|e| e.to_string())?.ip();
    if ip.is_unspecified() || ip.is_loopback() {
        return Err(
            "No LAN address found. Connect to your local network or enter an address manually."
                .into(),
        );
    }
    Ok(ip.to_string())
}

pub async fn local_addresses() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    let mut command = {
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let shell =
            std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut command = tokio::process::Command::new(&shell);
        command.env(
            "PSModulePath",
            shell
                .parent()
                .ok_or("PowerShell directory unavailable")?
                .join("Modules"),
        );
        command.args(["-NoProfile", "-NonInteractive", "-Command", "Get-NetIPAddress -AddressFamily IPv4 -AddressState Preferred | ForEach-Object IPAddress"]);
        command.creation_flags(0x0800_0000);
        command
    };
    #[cfg(not(windows))]
    let mut command = tokio::process::Command::new("/sbin/ifconfig");
    command.kill_on_drop(true);
    let mut addresses = Vec::new();
    if let Ok(Ok(output)) = tokio::time::timeout(Duration::from_secs(3), command.output()).await {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            #[cfg(windows)]
            let candidate = line.trim();
            #[cfg(not(windows))]
            let candidate = {
                let mut words = line.split_whitespace();
                if words.next() != Some("inet") {
                    continue;
                }
                words.next().unwrap_or("")
            };
            if candidate
                .parse::<Ipv4Addr>()
                .is_ok_and(|ip| !ip.is_loopback() && (ip.is_private() || ip.is_link_local()))
                && !addresses.iter().any(|a| a == candidate)
            {
                addresses.push(candidate.to_string());
            }
        }
    }
    if let Ok(primary) = local_address("192.0.2.1") {
        addresses.retain(|a| a != &primary);
        addresses.insert(0, primary);
    }
    addresses.truncate(16);
    Ok(addresses)
}

async fn scan_interface(address: &str) -> Result<Vec<Device>, String> {
    let socket = UdpSocket::bind((address, 0))
        .await
        .map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    socket
        .send_to(QUERY, (Ipv4Addr::BROADCAST, PORT))
        .await
        .map_err(|e| e.to_string())?;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(900);
    let mut devices = Vec::<Device>::new();
    let mut buffer = [0; 2048];
    while let Ok(Ok((length, source))) =
        tokio::time::timeout_at(deadline, socket.recv_from(&mut buffer)).await
    {
        if crate::network::validate_address(&source.ip().to_string()).is_err() {
            continue;
        }
        if let Ok(mut device) = serde_json::from_slice::<Device>(&buffer[..length]) {
            if device.id.len() > 128 || device.name.len() > 128 || device.port < 1024 {
                continue;
            }
            device.address = source.ip().to_string();
            if !devices
                .iter()
                .any(|d| d.id == device.id && d.address == device.address)
                && devices.len() < 64
            {
                devices.push(device);
            }
        }
    }
    Ok(devices)
}

pub async fn scan() -> Result<Vec<Device>, String> {
    let addresses = local_addresses().await?;
    let mut tasks = tokio::task::JoinSet::new();
    for address in addresses
        .into_iter()
        .chain(std::iter::once("0.0.0.0".to_string()))
    {
        tasks.spawn(async move { scan_interface(&address).await });
    }
    let mut devices = Vec::<Device>::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok(Ok(found)) = result {
            for device in found {
                if devices.len() < 64
                    && !devices
                        .iter()
                        .any(|d| d.id == device.id && d.address == device.address)
                {
                    devices.push(device);
                }
            }
        }
    }
    Ok(devices)
}
pub async fn serve(state: Shared) {
    loop {
        let socket = match UdpSocket::bind(("0.0.0.0", PORT)).await {
            Ok(socket) => socket,
            Err(error) => {
                state.log(
                    "warning",
                    format!("LAN discovery unavailable: {error}. Manual invitations still work."),
                );
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };
        let mut buffer = [0; 2048];
        while let Ok((length, source)) = socket.recv_from(&mut buffer).await {
            let config = state.config();
            if &buffer[..length] != QUERY
                || !config.network_enabled
                || *state.agent_status.lock().unwrap() != "Listening"
                || crate::network::validate_address(&source.ip().to_string()).is_err()
            {
                continue;
            }
            let device = Device {
                id: config.device_id,
                name: config.computer_name,
                address: String::new(),
                port: config.listen_port,
            };
            if let Ok(bytes) = serde_json::to_vec(&device) {
                let _ = socket.send_to(&bytes, source).await;
            }
        }
    }
}
