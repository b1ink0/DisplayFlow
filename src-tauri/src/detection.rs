use crate::{
    model::{DetectionStatus, InputReading},
    network, providers,
    store::Shared,
};
use serde_json::json;
use std::collections::HashSet;

pub async fn detect(state: &Shared, route_id: &str) -> Result<DetectionStatus, String> {
    crate::shared_routes::require_coordinator(state, route_id)?;
    let _cycle = state.detection_lock.lock().await;
    let config = state.config();
    let route = config
        .routes
        .iter()
        .find(|r| r.id == route_id)
        .cloned()
        .ok_or("Route is unavailable")?;
    let mut probes = tokio::task::JoinSet::new();
    for attachment in route.attachments.clone() {
        let state = state.clone();
        let peer = config
            .peers
            .iter()
            .find(|p| p.id == attachment.device_id)
            .cloned();
        let local = attachment.device_id == config.device_id;
        probes.spawn(async move {
            let result: Result<InputReading, String> = if local {
                let _guard = state.switch_lock.lock().await;
                providers::get_input(&state, &attachment.monitor_id).await
            } else if let Some(peer) = peer {
                network::remote(
                    &state,
                    &peer,
                    "getInput",
                    json!({"monitorId": attachment.monitor_id}),
                )
                .await
                .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
            } else {
                Err("Computer is not paired".into())
            };
            (attachment, result)
        });
    }
    let mut candidates = HashSet::new();
    let mut replies = 0;
    while let Some(result) = probes.join_next().await {
        let Ok((source, result)) = result else {
            continue;
        };
        if let Ok(reading) = result {
            if reading.input.is_none() && !reading.active_connection {
                continue;
            }
            replies += 1;
            if reading.active_connection {
                candidates.insert(source.computer);
            }
            if let Some(input) = reading.input {
                let matches: Vec<_> = route
                    .attachments
                    .iter()
                    .filter(|a| a.input == input)
                    .collect();
                if matches.len() == 1 {
                    candidates.insert(matches[0].computer.clone());
                } else {
                    candidates.insert(String::new());
                }
            }
        }
    }
    let _guard = state.switch_lock.lock().await;
    if state.config().revision != config.revision {
        return Ok(DetectionStatus {
            status: "pending".into(),
            message: "Assignments or input changed during detection; this reading was discarded."
                .into(),
            checked_at: crate::model::now(),
        });
    }
    let active = if candidates.len() == 1 {
        candidates.iter().next().filter(|v| !v.is_empty()).cloned()
    } else {
        None
    };
    let status = DetectionStatus {
        status: if active.is_some() { "detected" } else { "unknown" }.into(),
        message: match &active {
            Some(computer) => format!("Visible input detected: {computer}"),
            None if replies == 0 && route.active_computer.is_some() => "Automatic input readings are unavailable. Keeping the shared confirmed or requested input.".into(),
            None if replies == 0 => "No provider returned usable input evidence. Confirm the visible computer once to share it with all assigned computers.".into(),
            None => "Input replies are conflicting or do not match a unique assignment. Confirm the visible computer manually.".into(),
        },
        checked_at: crate::model::now(),
    };
    let changed = {
        let mut settings = state.settings.lock().unwrap();
        let current = settings
            .routes
            .iter_mut()
            .find(|r| r.id == route_id)
            .ok_or("Route was removed")?;
        let changed = current.active_computer != active;
        if replies > 0 {
            current.active_computer = active;
            changed
        } else {
            false
        }
    };
    state
        .route_detection
        .lock()
        .unwrap()
        .insert(route_id.into(), status.clone());
    if changed {
        state.save()?;
    }
    crate::shared_routes::changed(state, route_id);
    drop(_guard);
    crate::shared_routes::publish(state, route_id).await;
    Ok(status)
}

pub async fn watch(state: Shared) {
    loop {
        let config = state.config();
        if !config.paused {
            for route in config.routes.iter().filter(|r| {
                r.enabled
                    && r.auto_detect
                    && r.coordinator_device_id.as_deref() == Some(&config.device_id)
            }) {
                let _ = detect(&state, &route.id).await;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
