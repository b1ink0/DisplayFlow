use crate::{
    model::{DetectionStatus, Route, RouteSyncStatus},
    network,
    store::Shared,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SharedRoute {
    route_id: String,
    assignment_hash: String,
    coordinator_device_id: String,
    session_id: String,
    sequence: u64,
    auto_detect: bool,
    active_computer: Option<String>,
    detection: Option<DetectionStatus>,
}

pub fn fingerprint(route: &Route) -> String {
    let mut assignments: Vec<_> = route
        .attachments
        .iter()
        .map(|a| (&a.device_id, &a.monitor_id, &a.computer, a.input))
        .collect();
    assignments.sort_unstable();
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&assignments).unwrap())
    )
}
fn route(state: &Shared, id: &str) -> Result<Route, String> {
    state
        .config()
        .routes
        .into_iter()
        .find(|r| r.id == id)
        .ok_or("Route is unavailable".into())
}
fn sync_status(state: &Shared, id: &str, status: &str, message: String) {
    state.route_sync.lock().unwrap().insert(
        id.into(),
        RouteSyncStatus {
            status: status.into(),
            message,
        },
    );
}
pub fn changed(state: &Shared, id: &str) {
    if require_coordinator(state, id).is_err() {
        return;
    }
    let mut versions = state.shared_versions.lock().unwrap();
    let value = versions
        .entry(id.into())
        .or_insert((state.instance_id.clone(), 0));
    value.1 = value.1.saturating_add(1);
}
pub async fn describe(state: &Shared, id: &str) -> Result<Value, String> {
    let _guard = state.switch_lock.lock().await;
    let config = state.config();
    let route = route(state, id)?;
    let coordinator = route
        .coordinator_device_id
        .clone()
        .or_else(|| (!config.deskflow_log.is_empty()).then(|| config.device_id.clone()));
    let shared = if coordinator.as_deref() == Some(&config.device_id) {
        let sequence = state
            .shared_versions
            .lock()
            .unwrap()
            .get(id)
            .map(|v| v.1)
            .unwrap_or(0);
        Some(SharedRoute {
            route_id: id.into(),
            assignment_hash: fingerprint(&route),
            coordinator_device_id: config.device_id.clone(),
            session_id: state.instance_id.clone(),
            sequence,
            auto_detect: route.auto_detect,
            active_computer: route.active_computer,
            detection: state.route_detection.lock().unwrap().get(id).cloned(),
        })
    } else {
        None
    };
    Ok(json!({"ok":true,"coordinatorDeviceId":coordinator,"shared":shared}))
}
pub fn authorize(state: &Shared, sender: &str, payload: &Value) -> Result<String, String> {
    let id = payload["routeId"].as_str().ok_or("Missing route")?;
    let route = route(state, id)?;
    if payload["assignmentHash"].as_str() != Some(fingerprint(&route).as_str()) {
        return Err(
            "Route assignments differ; review and transfer matching assignments first".into(),
        );
    }
    if !route.attachments.iter().any(|a| a.device_id == sender)
        && route.coordinator_device_id.as_deref() != Some(sender)
    {
        return Err("Paired computer is not assigned to this route".into());
    }
    Ok(id.into())
}
pub fn require_coordinator(state: &Shared, id: &str) -> Result<(), String> {
    let config = state.config();
    if route(state, id)?.coordinator_device_id.as_deref() != Some(&config.device_id) {
        return Err("This computer does not coordinate the shared route".into());
    }
    Ok(())
}
async fn apply(
    state: &Shared,
    shared: SharedRoute,
    sender: &str,
    allow_session_change: bool,
) -> Result<(), String> {
    let _guard = state.switch_lock.lock().await;
    let local = route(state, &shared.route_id)?;
    if shared.coordinator_device_id != sender || fingerprint(&local) != shared.assignment_hash {
        return Err("Shared route does not match its coordinator or local assignments".into());
    }
    if local
        .coordinator_device_id
        .as_deref()
        .is_some_and(|id| id != sender)
    {
        return Err("Shared route came from a different coordinator".into());
    }
    if shared
        .active_computer
        .as_ref()
        .is_some_and(|name| !local.attachments.iter().any(|a| &a.computer == name))
    {
        return Err("Shared visible computer has no assignment".into());
    }
    {
        let mut versions = state.shared_versions.lock().unwrap();
        if let Some((session, sequence)) = versions.get(&shared.route_id) {
            if session == &shared.session_id && *sequence > shared.sequence {
                return Ok(());
            }
            if session != &shared.session_id && !allow_session_change {
                return Ok(());
            }
        }
        versions.insert(
            shared.route_id.clone(),
            (shared.session_id.clone(), shared.sequence),
        );
    }
    let changed = {
        let mut config = state.settings.lock().unwrap();
        let current = config
            .routes
            .iter_mut()
            .find(|r| r.id == shared.route_id)
            .ok_or("Route was removed")?;
        let changed = current.auto_detect != shared.auto_detect
            || current.active_computer != shared.active_computer
            || current.coordinator_device_id.as_deref() != Some(sender);
        current.auto_detect = shared.auto_detect;
        current.active_computer = shared.active_computer;
        current.coordinator_device_id = Some(sender.into());
        changed
    };
    let mut statuses = state.route_detection.lock().unwrap();
    if let Some(detection) = shared.detection {
        statuses.insert(shared.route_id.clone(), detection);
    } else {
        statuses.remove(&shared.route_id);
    }
    drop(statuses);
    if changed {
        state.save()?;
    }
    sync_status(
        state,
        &shared.route_id,
        "synced",
        "Detection and visible input are synchronized with the coordinator.".into(),
    );
    Ok(())
}
pub async fn receive(state: &Shared, sender: &str, payload: &Value) -> Result<Value, String> {
    let id = authorize(state, sender, payload)?;
    let shared: SharedRoute =
        serde_json::from_value(payload["shared"].clone()).map_err(|e| e.to_string())?;
    if shared.route_id != id {
        return Err("Shared route ID differs from the request".into());
    }
    apply(state, shared, sender, false).await?;
    Ok(json!({"ok":true}))
}

pub async fn coordinator(state: &Shared, id: &str) -> Result<String, String> {
    let config = state.config();
    let local = route(state, id)?;
    if let Some(coordinator) = &local.coordinator_device_id {
        return Ok(coordinator.clone());
    }
    if !config.deskflow_log.is_empty() {
        state
            .settings
            .lock()
            .unwrap()
            .routes
            .iter_mut()
            .find(|r| r.id == id)
            .unwrap()
            .coordinator_device_id = Some(config.device_id.clone());
        state.save()?;
        changed(state, id);
        return Ok(config.device_id);
    }
    let mut probes = tokio::task::JoinSet::new();
    let hash = fingerprint(&local);
    for peer in config
        .peers
        .iter()
        .filter(|p| local.attachments.iter().any(|a| a.device_id == p.id))
    {
        let state = state.clone();
        let peer = peer.clone();
        let id = id.to_string();
        let hash = hash.clone();
        probes.spawn(async move {
            let response = network::remote(
                &state,
                &peer,
                "getRoute",
                json!({"routeId":id,"assignmentHash":hash}),
            )
            .await;
            (peer.id, response)
        });
    }
    let mut authorities = Vec::new();
    while let Some(result) = probes.join_next().await {
        if let Ok((peer, Ok(response))) = result {
            if response["coordinatorDeviceId"].as_str() == Some(&peer)
                && !response["shared"].is_null()
            {
                authorities.push((peer, response));
            }
        }
    }
    if authorities.len() != 1 {
        return Err(
            "Choose one route coordinator, or start the paired Deskflow server to discover it."
                .into(),
        );
    }
    let (owner, response) = authorities.pop().unwrap();
    apply_response(state, id, &owner, response).await?;
    Ok(owner)
}
async fn apply_response(
    state: &Shared,
    id: &str,
    owner: &str,
    response: Value,
) -> Result<(), String> {
    let shared: SharedRoute = serde_json::from_value(response["shared"].clone())
        .map_err(|_| "Coordinator did not return shared route state")?;
    if shared.route_id != id {
        return Err("Coordinator returned a different route".into());
    }
    apply(state, shared, owner, true).await
}
async fn request(
    state: &Shared,
    id: &str,
    owner: &str,
    operation: &str,
    mut payload: Value,
) -> Result<Value, String> {
    let config = state.config();
    let peer = config
        .peers
        .iter()
        .find(|p| p.id == owner)
        .ok_or("Route coordinator is not paired on this computer")?;
    payload["routeId"] = json!(id);
    payload["assignmentHash"] = json!(fingerprint(&route(state, id)?));
    network::remote(state, peer, operation, payload).await
}
pub async fn detect(state: &Shared, id: &str) -> Result<DetectionStatus, String> {
    let owner = coordinator(state, id).await?;
    if owner == state.config().device_id {
        return crate::detection::detect(state, id).await;
    }
    let response = request(state, id, &owner, "detectRoute", json!({})).await?;
    let result = serde_json::from_value(response["shared"]["detection"].clone())
        .map_err(|_| "Coordinator did not return a detection result")?;
    apply_response(state, id, &owner, response).await?;
    Ok(result)
}
pub async fn configure(state: &Shared, id: &str, enabled: bool) -> Result<(), String> {
    let owner = coordinator(state, id).await?;
    if owner != state.config().device_id {
        let response = request(
            state,
            id,
            &owner,
            "configureRoute",
            json!({"autoDetect":enabled}),
        )
        .await?;
        return apply_response(state, id, &owner, response).await;
    }
    {
        let _guard = state.switch_lock.lock().await;
        state
            .settings
            .lock()
            .unwrap()
            .routes
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("Route was removed")?
            .auto_detect = enabled;
        state.save()?;
        changed(state, id);
    }
    publish(state, id).await;
    Ok(())
}
pub async fn confirm(state: &Shared, id: &str, computer: &str) -> Result<(), String> {
    let owner = coordinator(state, id).await?;
    if !route(state, id)?
        .attachments
        .iter()
        .any(|a| a.computer == computer)
    {
        return Err("Visible computer has no assignment".into());
    }
    if owner != state.config().device_id {
        let response = request(
            state,
            id,
            &owner,
            "confirmRoute",
            json!({"computer":computer}),
        )
        .await?;
        return apply_response(state, id, &owner, response).await;
    }
    {
        let _guard = state.switch_lock.lock().await;
        state
            .settings
            .lock()
            .unwrap()
            .routes
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("Route was removed")?
            .active_computer = Some(computer.into());
        state.route_detection.lock().unwrap().insert(
            id.into(),
            DetectionStatus {
                status: "confirmed".into(),
                message: format!("Visible input confirmed: {computer}"),
                checked_at: crate::model::now(),
            },
        );
        state.save()?;
        changed(state, id);
    }
    publish(state, id).await;
    Ok(())
}
pub async fn select_remote(
    state: &Shared,
    id: &str,
    owner: &str,
    destination: &str,
) -> Result<(), String> {
    let response = request(
        state,
        id,
        owner,
        "selectRoute",
        json!({"destination":destination}),
    )
    .await?;
    apply_response(state, id, owner, response).await
}
pub async fn publish(state: &Shared, id: &str) {
    let _publish = state.publish_lock.lock().await;
    if require_coordinator(state, id).is_err() {
        return;
    }
    let Ok(description) = describe(state, id).await else {
        return;
    };
    let Ok(route) = route(state, id) else {
        return;
    };
    let config = state.config();
    let mut pending: Vec<_> = route
        .attachments
        .iter()
        .filter(|a| {
            a.device_id != config.device_id && !config.peers.iter().any(|p| p.id == a.device_id)
        })
        .map(|a| a.computer.clone())
        .collect();
    let mut requests = tokio::task::JoinSet::new();
    for peer in config
        .peers
        .into_iter()
        .filter(|p| route.attachments.iter().any(|a| a.device_id == p.id))
    {
        let state = state.clone();
        let id = id.to_string();
        let shared = description["shared"].clone();
        let hash = fingerprint(&route);
        requests.spawn(async move {
            let result = network::remote(
                &state,
                &peer,
                "pushRoute",
                json!({"routeId":id,"assignmentHash":hash,"shared":shared}),
            )
            .await;
            (peer.name, result)
        });
    }
    while let Some(result) = requests.join_next().await {
        if let Ok((name, Err(_))) = result {
            pending.push(name);
        }
    }
    if pending.is_empty() {
        sync_status(
            state,
            id,
            "synced",
            "Shared detection and visible input are synchronized.".into(),
        );
    } else {
        sync_status(state,id,"pending",format!("Waiting for shared-state synchronization: {}. Reconnected agents fetch the latest state.",pending.join(", ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Attachment;
    struct Fixture {
        state: Shared,
        path: std::path::PathBuf,
    }
    impl Fixture {
        fn new(id: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("displayflow-sync-{}", uuid::Uuid::new_v4()));
            let state = crate::store::Runtime::load(path.clone()).unwrap();
            {
                let mut config = state.settings.lock().unwrap();
                config.device_id = id.into();
                config.routes = vec![Route {
                    id: "display".into(),
                    name: "Shared display".into(),
                    enabled: true,
                    auto_detect: false,
                    coordinator_device_id: Some("hub".into()),
                    active_computer: None,
                    attachments: [("hub", 15), ("first", 17), ("second", 18)]
                        .into_iter()
                        .map(|(id, input)| Attachment {
                            computer: id.into(),
                            device_id: id.into(),
                            monitor_id: format!("ddc:{id}"),
                            input,
                            label: format!("Input {input}"),
                        })
                        .collect(),
                }];
            }
            Self { state, path }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.path.join("settings.json"));
            let _ = std::fs::remove_dir(&self.path);
        }
    }
    fn message(state: &Shared, session: &str, sequence: u64) -> SharedRoute {
        SharedRoute {
            route_id: "display".into(),
            assignment_hash: fingerprint(&route(state, "display").unwrap()),
            coordinator_device_id: "hub".into(),
            session_id: session.into(),
            sequence,
            auto_detect: true,
            active_computer: Some("first".into()),
            detection: Some(DetectionStatus {
                status: "detected".into(),
                message: "Visible input detected: first".into(),
                checked_at: 1,
            }),
        }
    }
    #[tokio::test]
    async fn shared_setting_and_source_reach_each_follower() {
        for id in ["first", "second"] {
            let f = Fixture::new(id);
            apply(&f.state, message(&f.state, "session", 1), "hub", true)
                .await
                .unwrap();
            let r = route(&f.state, "display").unwrap();
            assert!(r.auto_detect);
            assert_eq!(r.active_computer.as_deref(), Some("first"));
            assert_eq!(
                f.state.route_detection.lock().unwrap()["display"].status,
                "detected"
            );
        }
    }
    #[tokio::test]
    async fn older_update_cannot_replace_newer_source() {
        let f = Fixture::new("first");
        apply(&f.state, message(&f.state, "session", 2), "hub", true)
            .await
            .unwrap();
        let mut old = message(&f.state, "session", 1);
        old.active_computer = Some("second".into());
        apply(&f.state, old, "hub", false).await.unwrap();
        assert_eq!(
            route(&f.state, "display")
                .unwrap()
                .active_computer
                .as_deref(),
            Some("first")
        );
    }
    #[tokio::test]
    async fn restart_requires_a_pull_before_accepting_new_session_pushes() {
        let f = Fixture::new("first");
        apply(&f.state, message(&f.state, "old", 5), "hub", true)
            .await
            .unwrap();
        let mut restarted = message(&f.state, "new", 0);
        restarted.active_computer = None;
        apply(&f.state, restarted.clone(), "hub", false)
            .await
            .unwrap();
        assert!(route(&f.state, "display")
            .unwrap()
            .active_computer
            .is_some());
        apply(&f.state, restarted, "hub", true).await.unwrap();
        assert!(route(&f.state, "display")
            .unwrap()
            .active_computer
            .is_none());
    }
    #[tokio::test]
    async fn different_mapping_or_coordinator_is_rejected() {
        let f = Fixture::new("first");
        let mut wrong = message(&f.state, "session", 1);
        wrong.assignment_hash = "different".into();
        assert!(apply(&f.state, wrong, "hub", true).await.is_err());
        assert!(
            apply(&f.state, message(&f.state, "session", 1), "second", true)
                .await
                .is_err()
        );
        assert!(!route(&f.state, "display").unwrap().auto_detect);
    }
    #[test]
    fn assignment_order_does_not_change_identity_but_input_does() {
        let f = Fixture::new("first");
        let mut r = route(&f.state, "display").unwrap();
        let hash = fingerprint(&r);
        r.attachments.reverse();
        assert_eq!(fingerprint(&r), hash);
        r.attachments[0].input += 1;
        assert_ne!(fingerprint(&r), hash);
    }
    #[tokio::test]
    async fn unchanged_version_restores_a_failed_local_setting_edit() {
        let f = Fixture::new("first");
        let m = message(&f.state, "session", 1);
        apply(&f.state, m.clone(), "hub", true).await.unwrap();
        f.state.settings.lock().unwrap().routes[0].auto_detect = false;
        apply(&f.state, m, "hub", true).await.unwrap();
        assert!(route(&f.state, "display").unwrap().auto_detect);
    }
    #[tokio::test]
    async fn snapshot_waits_for_source_and_sequence_to_commit_together() {
        let f = Fixture::new("hub");
        let guard = f.state.switch_lock.lock().await;
        f.state.settings.lock().unwrap().routes[0].active_computer = Some("second".into());
        let state = f.state.clone();
        let reading = tokio::spawn(async move { describe(&state, "display").await.unwrap() });
        tokio::task::yield_now().await;
        assert!(!reading.is_finished());
        changed(&f.state, "display");
        drop(guard);
        let value = reading.await.unwrap();
        assert_eq!(value["shared"]["sequence"], 1);
        assert_eq!(value["shared"]["activeComputer"], "second");
    }
}
pub async fn watch(state: Shared) {
    loop {
        let config = state.config();
        if config.network_enabled {
            for route in &config.routes {
                let result = async {
                    let owner = coordinator(&state, &route.id).await?;
                    if owner == config.device_id {
                        let pending = state
                            .route_sync
                            .lock()
                            .unwrap()
                            .get(&route.id)
                            .is_some_and(|s| s.status == "pending");
                        if pending {
                            publish(&state, &route.id).await;
                        }
                        return Ok::<(), String>(());
                    }
                    let response =
                        request(&state, &route.id, &owner, "getRoute", json!({})).await?;
                    apply_response(&state, &route.id, &owner, response).await
                }
                .await;
                if let Err(message) = result {
                    sync_status(&state, &route.id, "unavailable", message);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
