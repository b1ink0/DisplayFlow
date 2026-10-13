use crate::{model::Route, network, providers, store::Shared};
use serde_json::json;
pub async fn switch(state: &Shared, route_id: &str, destination: &str) -> Result<(), String> {
    let config = state.config();
    if config.paused {
        return Err("Routing is paused".into());
    }
    let coordinator = crate::shared_routes::coordinator(state, route_id).await?;
    if coordinator != config.device_id {
        return crate::shared_routes::select_remote(state, route_id, &coordinator, destination)
            .await;
    }
    if config
        .routes
        .iter()
        .any(|r| r.id == route_id && r.enabled && r.auto_detect && r.active_computer.is_none())
    {
        crate::detection::detect(state, route_id).await?;
    }
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
            {
                let mut config = state.settings.lock().unwrap();
                if let Some(r) = config.routes.iter_mut().find(|r| r.id == route_id) {
                    r.active_computer = Some(destination.into());
                }
            }
            state.save()?;
            state.route_detection.lock().unwrap().insert(
                route_id.into(),
                crate::model::DetectionStatus {
                    status: "requested".into(),
                    message: format!("Input requested: {destination}. Picture is not verified."),
                    checked_at: crate::model::now(),
                },
            );
            state.log("info", format!("{} → {destination}. {message}", route.name));
            crate::shared_routes::changed(state, route_id);
            drop(_guard);
            crate::shared_routes::publish(state, route_id).await;
            Ok(())
        }
        Err(error) => {
            {
                let mut config = state.settings.lock().unwrap();
                if let Some(r) = config.routes.iter_mut().find(|r| r.id == route_id) {
                    r.active_computer = None;
                }
            }
            let _ = state.save();
            state.route_detection.lock().unwrap().insert(route_id.into(),crate::model::DetectionStatus {status:"unknown".into(),message:"Input request failed. Confirm the visible computer once to share it with the route.".into(),checked_at:crate::model::now()});
            state.log(
                "error",
                format!("{}: {error}. Active input is now unknown.", route.name),
            );
            crate::shared_routes::changed(state, route_id);
            drop(_guard);
            crate::shared_routes::publish(state, route_id).await;
            Err(error)
        }
    }
}
