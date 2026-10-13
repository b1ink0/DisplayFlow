use crate::store::Shared;
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Write, path::Path};

const SERVICE: &str = "dev.displayflow.desktop";
const MAGIC: &[u8; 4] = b"DFV1";
const LIMIT: usize = 512 * 1024;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Data {
    version: u32,
    entries: BTreeMap<String, String>,
}
pub struct Vault {
    key: [u8; 32],
    data: Data,
}
fn entry(name: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, name).map_err(|e| format!("OS credential storage: {e}"))
}
fn read(name: &str) -> Result<Option<String>, String> {
    match entry(name)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("OS credential storage: {e}")),
    }
}
fn cipher(key: &[u8; 32]) -> Result<aead::LessSafeKey, String> {
    aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
        .map(aead::LessSafeKey::new)
        .map_err(|_| "Cannot initialize credential encryption".into())
}
fn decode(mut bytes: Vec<u8>, key: &[u8; 32], id: &str) -> Result<Data, String> {
    if bytes.len() < 32 || bytes.len() > LIMIT || &bytes[..4] != MAGIC {
        return Err("Credential vault is invalid; existing files were preserved".into());
    }
    let nonce: [u8; 12] = bytes[4..16].try_into().map_err(|_| "Invalid vault nonce")?;
    let plaintext = cipher(key)?
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(id.as_bytes()),
            &mut bytes[16..],
        )
        .map_err(|_| "Cannot authenticate the credential vault; existing files were preserved")?;
    let data: Data =
        serde_json::from_slice(plaintext).map_err(|_| "Invalid credential vault contents")?;
    if data.version != 1 {
        return Err("Unsupported credential vault version".into());
    }
    Ok(data)
}
fn persist(path: &Path, key: &[u8; 32], id: &str, data: &Data) -> Result<(), String> {
    let mut plaintext = serde_json::to_vec(data).map_err(|e| e.to_string())?;
    if plaintext.len() + 32 > LIMIT {
        return Err("Credential vault exceeds its size limit".into());
    }
    let mut nonce = [0u8; 12];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| "Cannot generate a vault nonce")?;
    cipher(key)?
        .seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(id.as_bytes()),
            &mut plaintext,
        )
        .map_err(|_| "Cannot encrypt credentials")?;
    let staging = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let work = || -> Result<(), String> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&staging).map_err(|e| e.to_string())?;
        file.write_all(MAGIC)
            .and_then(|_| file.write_all(&nonce))
            .and_then(|_| file.write_all(&plaintext))
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&staging, path).map_err(|e| e.to_string())
    };
    let result = work();
    if result.is_err() {
        let _ = std::fs::remove_file(staging);
    }
    result
}
fn load(state: &Shared) -> Result<Vault, String> {
    let config = state.config();
    let name = format!("vault-key-{}", config.device_id);
    let path = state.path.with_file_name("credentials.vault");
    let existing = read(&name)?;
    if path.exists() && existing.is_none() {
        return Err(
            "The vault encryption key is unavailable; existing credentials were preserved".into(),
        );
    }
    let key: [u8; 32] = match &existing {
        Some(encoded) => STANDARD
            .decode(encoded)
            .map_err(|_| "Invalid vault encryption key")?
            .try_into()
            .map_err(|_| "Invalid vault encryption key length")?,
        None => {
            let mut key = [0u8; 32];
            SystemRandom::new()
                .fill(&mut key)
                .map_err(|_| "Cannot generate a vault key")?;
            key
        }
    };
    if path.exists() {
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() || meta.len() > LIMIT as u64 {
            return Err("Invalid credential vault file".into());
        }
        return Ok(Vault {
            key,
            data: decode(
                std::fs::read(path).map_err(|e| e.to_string())?,
                &key,
                &config.device_id,
            )?,
        });
    }
    let mut names = vec![format!("identity-{}", config.device_id)];
    for peer in &config.peers {
        names.push(format!("inbound-{}-{}", config.device_id, peer.id));
        names.push(format!("outbound-{}-{}", config.device_id, peer.id));
    }
    let mut data = Data {
        version: 1,
        entries: BTreeMap::new(),
    };
    for legacy in &names {
        if let Some(value) = read(legacy)? {
            data.entries.insert(legacy.clone(), value);
        }
    }
    if existing.is_none() {
        entry(&name)?
            .set_password(&STANDARD.encode(key))
            .map_err(|e| format!("OS credential storage: {e}"))?;
        if read(&name)?.as_deref() != Some(STANDARD.encode(key).as_str()) {
            return Err("Vault key storage could not be verified".into());
        }
    }
    // Authenticate the replacement before removing any legacy credential.
    persist(&path, &key, &config.device_id, &data)?;
    if decode(
        std::fs::read(&path).map_err(|e| e.to_string())?,
        &key,
        &config.device_id,
    )? != data
    {
        return Err("Credential migration verification failed; legacy items were preserved".into());
    }
    let mut cleanup_failed = false;
    for legacy in data.entries.keys() {
        if let Err(error) = entry(legacy)?.delete_credential() {
            cleanup_failed |= !matches!(error, keyring::Error::NoEntry);
        }
    }
    if cleanup_failed {
        state.log(
            "warning",
            "Credentials migrated; some legacy OS credential items could not be removed.",
        );
    }
    Ok(Vault { key, data })
}
pub fn optional(state: &Shared, key: &str) -> Result<Option<String>, String> {
    let mut cached = state.credentials.lock().unwrap();
    if cached.is_none() {
        *cached = Some(load(state)?);
    }
    Ok(cached.as_ref().unwrap().data.entries.get(key).cloned())
}
pub fn get(state: &Shared, key: &str) -> Result<String, String> {
    optional(state, key)?
        .ok_or_else(|| "Pairing credential is unavailable; pair this computer again".into())
}
pub fn set(state: &Shared, key: &str, value: &str) -> Result<(), String> {
    let id = state.config().device_id;
    let mut cached = state.credentials.lock().unwrap();
    if cached.is_none() {
        *cached = Some(load(state)?);
    }
    let vault = cached.as_mut().unwrap();
    let mut next = vault.data.clone();
    next.entries.insert(key.into(), value.into());
    persist(
        &state.path.with_file_name("credentials.vault"),
        &vault.key,
        &id,
        &next,
    )?;
    vault.data = next;
    Ok(())
}
pub fn remove(state: &Shared, keys: &[String]) -> Result<(), String> {
    let id = state.config().device_id;
    let mut cached = state.credentials.lock().unwrap();
    if cached.is_none() {
        *cached = Some(load(state)?);
    }
    let vault = cached.as_mut().unwrap();
    let mut next = vault.data.clone();
    for key in keys {
        next.entries.remove(key);
    }
    persist(
        &state.path.with_file_name("credentials.vault"),
        &vault.key,
        &id,
        &next,
    )?;
    vault.data = next;
    Ok(())
}
