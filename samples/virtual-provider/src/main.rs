use serde_json::{json, Value};
use std::{
    io::{self, Read},
    path::Path,
};

fn handle(root: &Path, request: Value) -> Result<Value, String> {
    if request["apiVersion"] != 1 {
        return Err("Unsupported API version".into());
    }
    let operation = request["operation"].as_str().ok_or("Missing operation")?;
    if operation != "enumerate" && request["monitorId"] != "virtual-display" {
        return Err("Unknown sample monitor".into());
    }
    let path = root.join("sample-state.local.json");
    let mut state = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .unwrap_or(json!({"input":15,"requests":0}));
    let response = match operation {
        "enumerate" => {
            json!({"ok":true,"monitors":[{"id":"virtual-display","name":"Virtual sample display","provider":"sample","detail":"Simulation only; this provider performs no hardware I/O","inputPresets":[{"label":"HDMI 1","value":17},{"label":"HDMI 2","value":18},{"label":"DisplayPort","value":15}]}]})
        }
        "inspect" => {
            json!({"ok":true,"recoveryState":"notApplicable","message":"This virtual display demonstrates the process protocol without controlling hardware.","fields":{"Simulated input":state["input"],"Request count":state["requests"],"Hardware I/O":"None"}})
        }
        "setInput" => {
            let input = request["input"]
                .as_u64()
                .filter(|n| *n <= 65535)
                .ok_or("Invalid input")?;
            state["input"] = json!(input);
            state["requests"] = json!(state["requests"].as_u64().unwrap_or(0) + 1);
            json!({"ok":true,"message":"Simulated input updated; no physical monitor command was sent."})
        }
        "getInput" => json!({"ok":true,"input":state["input"],"activeConnection":false}),
        "action" => {
            if request["panelId"] != "sample-tools" {
                return Err("Unknown panel".into());
            }
            match request["actionId"].as_str() {
                Some("describe") => {
                    json!({"ok":true,"recoveryState":"notApplicable","message":"A module contributed this tab and its actions through its manifest.","fields":{"Protocol":"JSON stdin/stdout v1","Simulated input":state["input"],"Request count":state["requests"],"Hardware I/O":"None"}})
                }
                Some("reset") => {
                    if request["confirmed"] != true {
                        return Err("Reset requires confirmation".into());
                    }
                    state = json!({"input":15,"requests":0});
                    json!({"ok":true,"message":"Sample state reset. No hardware was accessed.","fields":{"Simulated input":15,"Request count":0}})
                }
                _ => return Err("Unknown action".into()),
            }
        }
        _ => return Err("Unsupported operation".into()),
    };
    if operation == "setInput" || operation == "action" && request["actionId"] == "reset" {
        std::fs::write(path, serde_json::to_vec(&state).unwrap()).map_err(|e| e.to_string())?;
    }
    Ok(response)
}
fn main() {
    let result = (|| {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let root = executable.parent().ok_or("Module directory unavailable")?;
        let mut input = String::new();
        io::stdin()
            .take(65_537)
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > 65_536 {
            return Err("Request exceeds 64 KiB".into());
        }
        handle(
            root,
            serde_json::from_str(&input).map_err(|e| e.to_string())?,
        )
    })();
    println!(
        "{}",
        match result {
            Ok(value) => value,
            Err(message) => json!({"ok":false,"message":message}),
        }
    );
}
