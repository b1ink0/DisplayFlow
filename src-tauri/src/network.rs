use crate::{
    credentials,
    model::{now, Peer},
    providers,
    store::Shared,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{TlsAcceptor, TlsConnector};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Invitation {
    pub version: u32,
    pub peer: Peer,
    pub token: String,
    pub expires: u64,
}
fn token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    B64.encode(bytes)
}
fn equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}
fn identity(state: &Shared) -> Result<(String, String), String> {
    let _guard = state.identity_lock.lock().unwrap();
    let id = state.config().device_id;
    let entry = format!("identity-{id}");
    match credentials::optional(state, &entry)? {
        Some(value) => serde_json::from_str(&value).map_err(|e| e.to_string()),
        None => {
            let rcgen::CertifiedKey { cert, key_pair } =
                rcgen::generate_simple_self_signed(vec!["displayflow.local".into()])
                    .map_err(|e| e.to_string())?;
            let value = (B64.encode(cert.der()), B64.encode(key_pair.serialize_der()));
            credentials::set(state, &entry, &serde_json::to_string(&value).unwrap())?;
            Ok(value)
        }
    }
}
pub fn invitation(state: &Shared, address: String) -> Result<String, String> {
    let address = if address.trim().is_empty() {
        crate::discovery::local_address("192.0.2.1")?
    } else {
        address.trim().to_string()
    };
    let config = state.config();
    if !config.network_enabled || *state.agent_status.lock().unwrap() != "Listening" {
        return Err("Enable the peer agent and wait until it is listening first".into());
    }
    validate_address(&address)?;
    let (cert, _) = identity(state)?;
    let secret = token();
    let expires = now() + 600;
    *state.invitation.lock().unwrap() = Some((secret.clone(), expires));
    serde_json::to_string(&Invitation {
        version: 1,
        peer: Peer {
            id: config.device_id,
            name: config.computer_name,
            address,
            port: config.listen_port,
            certificate: cert,
        },
        token: secret,
        expires,
    })
    .map_err(|e| e.to_string())
}
pub(crate) fn validate_address(address: &str) -> Result<(), String> {
    let ip: std::net::IpAddr = address
        .parse()
        .map_err(|_| "Use a local-network IP address")?;
    let local = match ip {
        std::net::IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
        std::net::IpAddr::V6(_) => false,
    };
    if !local {
        return Err("Only local-network addresses are supported".into());
    }
    Ok(())
}
async fn exchange(state: &Shared, peer: &Peer, payload: Value) -> Result<Value, String> {
    validate_address(&peer.address)?;
    let bytes = B64.decode(&peer.certificate).map_err(|e| e.to_string())?;
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(bytes))
        .map_err(|e| e.to_string())?;
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let input_read = payload["operation"] == "getInput";
    let shared_read = payload["operation"] == "getRoute" || payload["operation"] == "pushRoute";
    let timeout = if input_read || shared_read {
        12
    } else if payload["operation"] == "selectRoute" {
        35
    } else {
        20
    };
    let work = async {
        let connector = TlsConnector::from(Arc::new(config));
        let connect = |address: String, port| {
            let connector = connector.clone();
            async move {
                let stream = tokio::time::timeout(
                    Duration::from_millis(1200),
                    TcpStream::connect((address.as_str(), port)),
                )
                .await
                .map_err(|_| "Connection timed out".to_string())?
                .map_err(|e| e.to_string())?;
                tokio::time::timeout(
                    Duration::from_millis(1200),
                    connector.connect(
                        ServerName::try_from("displayflow.local")
                            .unwrap()
                            .to_owned(),
                        stream,
                    ),
                )
                .await
                .map_err(|_| "TLS handshake timed out".to_string())?
                .map_err(|e| format!("Pinned TLS connection failed: {e}"))
            }
        };
        let (stream, address, port) = match connect(peer.address.clone(), peer.port).await {
            Ok(stream) => (stream, peer.address.clone(), peer.port),
            Err(original) => {
                let candidates = crate::discovery::scan().await?;
                let mut found = None;
                for candidate in candidates.into_iter().filter(|d| d.id == peer.id).take(4) {
                    if let Ok(stream) = connect(candidate.address.clone(), candidate.port).await {
                        found = Some((stream, candidate.address, candidate.port));
                        break;
                    }
                }
                found.ok_or_else(|| format!("Cannot reach {}: {original}. Check that both agents are Listening and allow DisplayFlow's TCP port and UDP 24863 on your private LAN. If discovery is blocked, use a fresh invitation.", peer.name))?
            }
        };
        let mut stream = BufReader::new(stream);
        let mut bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        stream
            .get_mut()
            .write_all(&bytes)
            .await
            .map_err(|e| e.to_string())?;
        let mut result = vec![];
        let mut limited = tokio::io::AsyncReadExt::take(&mut stream, 65_537);
        let mut reader = BufReader::new(&mut limited);
        reader
            .read_until(b'\n', &mut result)
            .await
            .map_err(|e| e.to_string())?;
        if result.len() > 65_536 {
            return Err("Peer response exceeds 64 KiB".into());
        }
        let mut result: Value = serde_json::from_slice(&result).map_err(|e| e.to_string())?;
        if result["ok"] != true {
            return Err(result["message"]
                .as_str()
                .unwrap_or("Peer declined request")
                .into());
        }
        // Discovery is only an address hint. Persist it after pinned TLS and authentication succeed.
        let changed = {
            let mut config = state.settings.lock().unwrap();
            if let Some(saved) = config
                .peers
                .iter_mut()
                .find(|p| p.id == peer.id && p.certificate == peer.certificate)
            {
                let changed = saved.address != address || saved.port != port;
                saved.address = address.clone();
                saved.port = port;
                changed
            } else {
                false
            }
        };
        if changed {
            if let Err(error) = state.save() {
                state.log(
                    "warning",
                    format!("Peer address changed but could not be saved: {error}"),
                );
            }
        }
        result["connectedAddress"] = json!(address);
        result["connectedPort"] = json!(port);
        Ok(result)
    };
    tokio::time::timeout(Duration::from_secs(timeout), work)
        .await
        .map_err(|_| {
            if input_read {
                "Peer input query timed out; no input evidence was obtained.".to_string()
            } else if payload["operation"] == "pair" {
                "Pairing timed out. Check both agents and LAN permissions. Create a fresh invitation before retrying.".to_string()
            } else if payload["operation"] != "setInput" && payload["operation"] != "selectRoute" {
                "Peer request timed out. Check the agent and LAN permissions.".to_string()
            } else {
                "Peer timed out; switch outcome is unknown. Reconfirm the active input before retrying.".to_string()
            }
        })?
}
pub async fn pair(
    state: &Shared,
    invitation_text: String,
    own_address: String,
) -> Result<(), String> {
    let mut invite: Invitation = serde_json::from_str(invitation_text.trim()).map_err(|_| {
        "This is not a complete invitation. Copy the entire invitation from the other computer."
            .to_string()
    })?;
    let settings = state.config();
    if !settings.network_enabled {
        return Err("Enable this computer's peer agent first".into());
    }
    if invite.version != 1 || invite.expires < now() || invite.peer.id == settings.device_id {
        return Err("Invitation is expired, unsupported, or belongs to this device".into());
    }
    let own_address = if own_address.trim().is_empty() {
        crate::discovery::local_address(&invite.peer.address)?
    } else {
        own_address.trim().to_string()
    };
    validate_address(&own_address)?;
    let (certificate, _) = identity(state)?;
    let me = Peer {
        id: settings.device_id.clone(),
        name: settings.computer_name,
        address: own_address,
        port: settings.listen_port,
        certificate,
    };
    let reverse_token = token();
    credentials::set(
        state,
        &format!("inbound-{}-{}", settings.device_id, invite.peer.id),
        &reverse_token,
    )?;
    let response = exchange(
        state,
        &invite.peer,
        json!({"operation":"pair", "token":invite.token, "peer":me, "reverseToken":reverse_token}),
    )
    .await?;
    let credential = response["token"]
        .as_str()
        .ok_or("Pairing response missing credential")?;
    invite.peer.address = response["connectedAddress"]
        .as_str()
        .ok_or("Missing connected address")?
        .into();
    invite.peer.port = response["connectedPort"]
        .as_u64()
        .ok_or("Missing connected port")? as u16;
    credentials::set(
        state,
        &format!("outbound-{}-{}", settings.device_id, invite.peer.id),
        credential,
    )?;
    {
        let mut config = state.settings.lock().unwrap();
        config.peers.retain(|p| p.id != invite.peer.id);
        config.peers.push(invite.peer.clone());
    }
    state.save()?;
    state.log("success", format!("Paired with {}", invite.peer.name));
    Ok(())
}
pub async fn remote(
    state: &Shared,
    peer: &Peer,
    operation: &str,
    mut payload: Value,
) -> Result<Value, String> {
    let id = state.config().device_id;
    payload["operation"] = json!(operation);
    payload["deviceId"] = json!(id);
    payload["token"] = json!(credentials::get(
        state,
        &format!("outbound-{id}-{}", peer.id)
    )?);
    exchange(state, peer, payload).await
}
async fn handle(state: &Shared, payload: Value, source: std::net::IpAddr) -> Result<Value, String> {
    if !state.config().network_enabled {
        return Err("Peer agent is disabled".into());
    }
    let operation = payload["operation"].as_str().ok_or("Missing operation")?;
    if operation == "pair" {
        let mut peer: Peer =
            serde_json::from_value(payload["peer"].clone()).map_err(|e| e.to_string())?;
        peer.address = source.to_string();
        validate_address(&peer.address)?;
        let secret = payload["token"]
            .as_str()
            .ok_or("Missing invitation token")?;
        let reverse = payload["reverseToken"]
            .as_str()
            .ok_or("Missing reciprocal credential")?;
        if reverse.len() != 44 {
            return Err("Invalid reciprocal credential".into());
        }
        {
            let mut invitation = state.invitation.lock().unwrap();
            match &*invitation {
                Some((value, expires)) if *expires >= now() && equal(value, secret) => {}
                _ => return Err("Invitation expired or already used".into()),
            }
            *invitation = None;
        }
        let token = token();
        let id = state.config().device_id;
        credentials::set(state, &format!("inbound-{id}-{}", peer.id), &token)?;
        credentials::set(state, &format!("outbound-{id}-{}", peer.id), reverse)?;
        {
            let mut config = state.settings.lock().unwrap();
            config.peers.retain(|p| p.id != peer.id);
            config.peers.push(peer.clone());
        }
        state.save()?;
        state.log("success", format!("Paired with {}", peer.name));
        return Ok(json!({"ok":true,"token":token}));
    }
    let sender = payload["deviceId"].as_str().ok_or("Missing device ID")?;
    let config = state.config();
    if !config.peers.iter().any(|p| p.id == sender) {
        return Err("Unpaired device".into());
    }
    let expected = credentials::get(state, &format!("inbound-{}-{sender}", config.device_id))?;
    if !equal(&expected, payload["token"].as_str().unwrap_or("")) {
        return Err("Authentication failed".into());
    }
    let changed = {
        let mut settings = state.settings.lock().unwrap();
        if let Some(peer) = settings.peers.iter_mut().find(|p| p.id == sender) {
            let address = source.to_string();
            let changed = peer.address != address;
            peer.address = address;
            changed
        } else {
            false
        }
    };
    if changed {
        state.save()?;
    }
    match operation {
        "getRoute" | "configureRoute" | "confirmRoute" | "detectRoute" | "selectRoute" => {
            let id = crate::shared_routes::authorize(state, sender, &payload)?;
            if operation == "getRoute" {
                return crate::shared_routes::describe(state, &id).await;
            }
            crate::shared_routes::require_coordinator(state, &id)?;
            match operation {
                "configureRoute" => {
                    crate::shared_routes::configure(
                        state,
                        &id,
                        payload["autoDetect"]
                            .as_bool()
                            .ok_or("Missing detection setting")?,
                    )
                    .await?
                }
                "confirmRoute" => {
                    crate::shared_routes::confirm(
                        state,
                        &id,
                        payload["computer"]
                            .as_str()
                            .ok_or("Missing visible computer")?,
                    )
                    .await?
                }
                "detectRoute" => {
                    crate::detection::detect(state, &id).await?;
                }
                "selectRoute" => {
                    crate::routing::switch(
                        state,
                        &id,
                        payload["destination"]
                            .as_str()
                            .ok_or("Missing destination")?,
                    )
                    .await?;
                }
                _ => unreachable!(),
            }
            crate::shared_routes::describe(state, &id).await
        }
        "pushRoute" => crate::shared_routes::receive(state, sender, &payload).await,
        "inventory" => Ok(json!({"ok":true,"monitors":providers::enumerate(state).await?})),
        "getInput" => {
            let monitor = payload["monitorId"].as_str().ok_or("Missing monitor")?;
            if !config.routes.iter().any(|r| {
                r.attachments
                    .iter()
                    .any(|a| a.device_id == config.device_id && a.monitor_id == monitor)
            }) {
                return Err(
                    "Monitor is not allowed by this computer's local route configuration".into(),
                );
            }
            let _guard = state.switch_lock.lock().await;
            let reading = providers::get_input(state, monitor).await?;
            Ok(
                json!({"ok":true,"input":reading.input,"activeConnection":reading.active_connection}),
            )
        }
        "setInput" => {
            if config.paused {
                return Err("Receiving agent is paused".into());
            }
            let monitor = payload["monitorId"].as_str().ok_or("Missing monitor")?;
            let input = payload["input"]
                .as_u64()
                .filter(|v| *v <= 65_535)
                .ok_or("Invalid input")? as u16;
            let allowed = config.routes.iter().any(|r| {
                r.enabled
                    && r.attachments
                        .iter()
                        .any(|a| a.device_id == config.device_id && a.monitor_id == monitor)
                    && r.attachments.iter().any(|a| a.input == input)
            });
            if !allowed {
                return Err(
                    "Monitor/input is not allowed by this computer's local route configuration"
                        .into(),
                );
            }
            let _guard = state.switch_lock.lock().await;
            let message = providers::set_input(state, monitor, input).await?;
            state.log("info", format!("Paired input request: {message}"));
            Ok(json!({"ok":true,"message":message}))
        }
        _ => Err("Operation is not available remotely".into()),
    }
}
pub async fn serve(state: Shared) {
    loop {
        let config = state.config();
        if !config.network_enabled {
            *state.agent_status.lock().unwrap() = "Disabled".into();
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        let setup = (|| -> Result<(TlsAcceptor, (String, String)), String> {
            let identity = identity(&state)?;
            let cert = CertificateDer::from(B64.decode(&identity.0).map_err(|e| e.to_string())?);
            let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                B64.decode(&identity.1).map_err(|e| e.to_string())?,
            ));
            let tls = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![cert], key)
                .map_err(|e| e.to_string())?;
            Ok((TlsAcceptor::from(Arc::new(tls)), identity))
        })();
        let listener = TcpListener::bind(("0.0.0.0", config.listen_port)).await;
        let (acceptor, listener) = match (setup, listener) {
            (Ok((a, _)), Ok(l)) => (a, l),
            (a, l) => {
                let error = a
                    .err()
                    .or_else(|| l.err().map(|e| e.to_string()))
                    .unwrap_or_default();
                *state.agent_status.lock().unwrap() = format!("Error: {error}");
                state.log("error", error);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        *state.agent_status.lock().unwrap() = "Listening".into();
        let permits = Arc::new(tokio::sync::Semaphore::new(8));
        loop {
            let current = state.config();
            if !current.network_enabled || current.listen_port != config.listen_port {
                break;
            }
            let accepted = tokio::time::timeout(Duration::from_secs(1), listener.accept()).await;
            if let Ok(Ok((socket, remote_address))) = accepted {
                if validate_address(&remote_address.ip().to_string()).is_err() {
                    continue;
                }
                let Ok(permit) = permits.clone().try_acquire_owned() else {
                    continue;
                };
                let state = state.clone();
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    let work = async {
                        let tls = acceptor.accept(socket).await.map_err(|e| e.to_string())?;
                        let mut stream = BufReader::new(tls);
                        let mut bytes = vec![];
                        {
                            let mut limited = tokio::io::AsyncReadExt::take(&mut stream, 65_537);
                            let mut reader = BufReader::new(&mut limited);
                            reader
                                .read_until(b'\n', &mut bytes)
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                        if bytes.len() > 65_536 {
                            return Err("Request exceeds 64 KiB".to_string());
                        }
                        let result = match serde_json::from_slice::<Value>(&bytes) {
                            Ok(value) => handle(&state, value, remote_address.ip()).await,
                            Err(_) => Err("Invalid request".into()),
                        };
                        let value = match result {
                            Ok(value) => value,
                            Err(message) => json!({"ok":false,"message":message}),
                        };
                        let mut bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
                        bytes.push(b'\n');
                        stream
                            .get_mut()
                            .write_all(&bytes)
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok::<_, String>(())
                    };
                    let _ = tokio::time::timeout(Duration::from_secs(35), work).await;
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn exercise_response(reply: bool) -> Result<Value, String> {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();
        let directory =
            std::env::temp_dir().join(format!("displayflow-network-test-{}", uuid::Uuid::new_v4()));
        let state = crate::store::Runtime::load(directory.clone()).unwrap();
        let key = rcgen::generate_simple_self_signed(vec!["displayflow.local".into()]).unwrap();
        let certificate = B64.encode(key.cert.der());
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(key.cert.der().to_vec())],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.key_pair.serialize_der())),
            )
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer = Peer {
            id: "test-peer".into(),
            name: "Test peer".into(),
            address: "127.0.0.1".into(),
            port: listener.local_addr().unwrap().port(),
            certificate,
        };
        let task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let tls = TlsAcceptor::from(Arc::new(config))
                .accept(socket)
                .await
                .unwrap();
            let mut stream = BufReader::new(tls);
            let mut line = String::new();
            stream.read_line(&mut line).await.unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["operation"],
                "inventory"
            );
            if reply {
                stream
                    .get_mut()
                    .write_all(b"{\"ok\":true,\"monitors\":[]}\n")
                    .await
                    .unwrap();
            }
            // Keep the listener open briefly: a resend would create a second connection.
            assert!(
                tokio::time::timeout(Duration::from_millis(150), listener.accept())
                    .await
                    .is_err()
            );
        });
        let result = exchange(&state, &peer, json!({"operation":"inventory"})).await;
        task.await.unwrap();
        std::fs::remove_dir(directory).unwrap();
        result
    }

    #[tokio::test]
    async fn pinned_peer_reply_is_returned_with_actual_endpoint() {
        let reply = exercise_response(true).await.unwrap();
        assert_eq!(reply["connectedAddress"], "127.0.0.1");
        assert_eq!(reply["monitors"], json!([]));
    }

    #[tokio::test]
    async fn lost_response_does_not_resend_a_transmitted_request() {
        assert!(exercise_response(false).await.is_err());
    }
}
