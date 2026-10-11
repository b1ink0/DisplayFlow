use crate::{routing, store::Shared};
use regex::Regex;
use std::io::{Read, Seek, SeekFrom};
use tokio::time::{sleep, Duration};

pub async fn watch(state: Shared) {
    let transition = Regex::new(r#"(?:switch|jump) from \"([^\"]+)\" to \"([^\"]+)\""#).unwrap();
    let mut watched = String::new();
    let mut file: Option<std::fs::File> = None;
    let mut position = 0;
    let mut pending = String::new();
    loop {
        sleep(Duration::from_millis(180)).await;
        let settings = state.config();
        if settings.deskflow_log.is_empty() {
            *state.log_status.lock().unwrap() = "Choose the server log".into();
            file = None;
            watched.clear();
            continue;
        }
        if watched != settings.deskflow_log || file.is_none() {
            match std::fs::File::open(&settings.deskflow_log) {
                Ok(mut opened) => {
                    position = opened.seek(SeekFrom::End(0)).unwrap_or(0);
                    file = Some(opened);
                    watched = settings.deskflow_log.clone();
                    pending.clear();
                    *state.log_status.lock().unwrap() = "Watching new transitions".into();
                    state.log(
                        "info",
                        "Deskflow log attached; historical transitions skipped",
                    );
                }
                Err(e) => {
                    *state.log_status.lock().unwrap() = format!("Cannot open log: {e}");
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            }
        }
        let Some(opened) = file.as_mut() else {
            continue;
        };
        // Resume at the end after log rotation to skip historical events.
        if let Ok(meta) = std::fs::metadata(&watched) {
            if meta.len() < position {
                file = None;
                continue;
            }
            if let Ok(old) = opened.metadata() {
                if old.created().ok() != meta.created().ok() {
                    file = None;
                    continue;
                }
            }
        } else {
            file = None;
            continue;
        }
        let mut bytes = vec![0; 65_536];
        let count = match opened.read(&mut bytes) {
            Ok(n) => n,
            Err(_) => {
                file = None;
                continue;
            }
        };
        position += count as u64;
        pending.push_str(&String::from_utf8_lossy(&bytes[..count]));
        let mut destination = None;
        while let Some(end) = pending.find('\n') {
            let line: String = pending.drain(..=end).collect();
            if let Some(captures) = transition.captures(&line) {
                destination = Some(captures[2].to_string());
            }
        }
        if pending.len() > 65_536 {
            pending.clear();
        }
        if settings.paused {
            continue;
        }
        if let Some(destination) = destination {
            for route in settings
                .routes
                .iter()
                .filter(|r| r.enabled && r.attachments.iter().any(|a| a.computer == destination))
            {
                if let Err(error) = routing::switch(&state, &route.id, &destination).await {
                    state.log(
                        "error",
                        format!("Deskflow destination {destination}: {error}"),
                    );
                }
            }
        }
    }
}
