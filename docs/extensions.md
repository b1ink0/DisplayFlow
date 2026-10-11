# Extension development

An extension is an external native executable plus a `displayflow-module.json` manifest. It communicates with DisplayFlow using one JSON request on stdin and one response on stdout. It can expose a monitor provider, diagnostics, optional local recovery, and additional application tabs.

## Start from the working example

```sh
bun run module:sample:build
```

Add `samples/virtual-provider/dist` in Modules, review the executable, and trust it. The **Sample tools** tab appears automatically. Select the virtual display and run its status or reset action. Its setter changes only an ignored local file; it does not access physical monitors.

Copy `samples/virtual-provider` into a separate module repository. Replace its simulated transport, update the manifest ID and description, and build for the platform where the module will run. No application source change or rebuild is required to mount an existing compiled module.

## Contribute tabs

Add an `ui.panels` section to the manifest:

```json
{
  "ui": {
    "panels": [
      {
        "id": "diagnostics",
        "title": "Provider diagnostics",
        "description": "Inspect this provider's connected display.",
        "actions": [
          {
            "id": "status",
            "label": "Check status",
            "description": "Read diagnostic information without changing the monitor.",
            "requiresConfirmation": false
          }
        ]
      }
    ]
  }
}
```

Declare the `action` capability. DisplayFlow renders the tab, monitor selector, buttons, confirmation dialog, status badge, and response fields. It sends:

```json
{
  "apiVersion": 1,
  "operation": "action",
  "panelId": "diagnostics",
  "actionId": "status",
  "monitorId": "your-monitor-id",
  "confirmed": false
}
```

Example response:

```json
{
  "ok": true,
  "message": "Connected display identified.",
  "recoveryState": "notApplicable",
  "fields": { "Connection": "Available", "Transport": "Native API" }
}
```

Contributions are declarative: DisplayFlow renders labels and results as text. A module may contribute up to eight panels with twelve actions each. Panel actions run locally.

`requiresConfirmation` causes DisplayFlow to prompt before invoking an action. The module must validate the confirmation too. Monitor recovery should use the dedicated `recover` operation so its visible-input requirement remains explicit.

## Trust and integrity

Modules run with the local user's permissions and can access the same resources as that user. The executable must resolve inside the mounted directory. DisplayFlow compares its hash and manifest before each invocation; changed modules require another trust review. The module must verify its own supporting scripts, configuration, payloads, and physical device identity.

Use `fields` for readable status values. Report `unknown` when a state cannot be verified. A request timestamp can be supplied as `lastRecoveryAt`, but it is displayed separately from verified recovery state.

See [process protocol](modules.md) for all operations, response limits, and timeouts.
