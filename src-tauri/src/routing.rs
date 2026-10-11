use crate::{model::Route, network, providers, store::Shared};
use serde_json::json;
pub async fn switch(state: &Shared, route_id: &str, destination: &str) -> Result<(), String> {
    let _guard = state.switch_lock.lock().await;
    let settings = state.config();
    if settings.paused {
        return Err("Routing is paused".into());
    }
    let route: Route = settings
        .routes
        .iter()
        .find(|r| r.id == route_id && r.enabled)
        .cloned()
        .ok_or("Route is unavailable or disabled")?;
    let target = route
        .attachments
        .iter()
        .find(|a| a.computer == destination)
        .ok_or("Destination has no assignment")?;
    let active = route
        .active_computer
        .as_deref()
        .ok_or("Confirm the currently visible computer first")?;
    if active == destination {
        return Ok(());
    }
    let source = route
        .attachments
        .iter()
        .find(|a| a.computer == active)
        .ok_or("Active attachment has no assignment")?;
    let result = if source.device_id == settings.device_id {
        providers::set_input(state, &source.monitor_id, target.input).await
    } else {
        let peer = settings
            .peers
            .iter()
            .find(|p| p.id == source.device_id)
            .ok_or("Active computer is not paired")?;
        network::remote(
            state,
            peer,
            "setInput",
            json!({"monitorId":source.monitor_id,"input":target.input}),
        )
        .await
        .map(|_| "Peer request sent; picture is not verified".to_string())
    };
    match result {
        Ok(message) => {
            let mut config = state.settings.lock().unwrap();
            if let Some(r) = config.routes.iter_mut().find(|r| r.id == route_id) {
                r.active_computer = Some(destination.into());
            }
            drop(config);
            state.save()?;
            state.log("info", format!("{} → {destination}. {message}", route.name));
            Ok(())
        }
        Err(error) => {
            let mut config = state.settings.lock().unwrap();
            if let Some(r) = config.routes.iter_mut().find(|r| r.id == route_id) {
                r.active_computer = None;
            }
            drop(config);
            let _ = state.save();
            state.log(
                "error",
                format!("{}: {error}. Active input is now unknown.", route.name),
            );
            Err(error)
        }
    }
}
