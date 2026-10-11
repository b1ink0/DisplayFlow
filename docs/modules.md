# External module protocol v1

A module is an executable and a `displayflow-module.json` in a directory selected by the user. Modules run with the user's permissions after a trust review. A complete virtual provider example is included in `samples/virtual-provider`.

Example manifest (generic; requires your own executable):

```json
{
  "apiVersion": 1,
  "id": "my-input-provider",
  "name": "My input provider",
  "version": "1.0.0",
  "description": "Local input control and optional recovery",
  "executable": "provider.exe",
  "arguments": [],
  "capabilities": ["enumerate", "setInput", "inspect", "recover"]
}
```

Use a relative executable path resolving inside the module directory. On macOS, give the file executable permission. Include a launcher or interpreter inside that directory if needed. Platform-specific module directories may expose the same module ID.

Declare the capabilities implemented by the executable:

| Capability  | Purpose                                    |
| ----------- | ------------------------------------------ |
| `enumerate` | List supported monitors                    |
| `setInput`  | Select a monitor input                     |
| `inspect`   | Read monitor or provider status            |
| `recover`   | Perform a confirmed local recovery action  |
| `action`    | Run actions declared in contributed panels |

For an input provider, implement `enumerate` and `setInput`. Diagnostics, recovery, and panels are optional.

Each invocation starts a fresh process with that directory as its working directory. DisplayFlow sends **one UTF-8 JSON request and newline** to stdin, closes stdin, and expects exactly one JSON response on stdout followed by process exit. Exit status must be zero and response `ok` must be `true`. Write diagnostics to stderr (discarded), never stdout. Timeout defaults to 15 seconds; `timeoutSeconds` and `recoveryTimeoutSeconds` can override it from 1 to 60 seconds. Output is limited to 64 KiB. Requests are serialized with other local switching/recovery actions. Do not retry writes inside a module unless you know they were not accepted.

## Requests and responses

Inventory:

```json
{ "apiVersion": 1, "operation": "enumerate" }
```

```json
{
  "ok": true,
  "monitors": [
    {
      "id": "unit-serial-123",
      "name": "Desk display",
      "provider": "ignored",
      "detail": "Identified physical unit",
      "inputPresets": [
        { "label": "HDMI 1", "value": 17 },
        { "label": "DisplayPort", "value": 15 }
      ]
    }
  ]
}
```

DisplayFlow prefixes IDs as `module:<module-id>:<monitor-id>` and replaces `provider` with the module ID. Use a stable physical identifier and reject ambiguous identification. Registering a module executes inventory only after trust confirmation and refresh.

The optional `inputPresets` array supplies choices for the route editor. Each entry has a numeric `value` (0–65535) and a nonempty `label` of at most 64 UTF-8 bytes. Supply at most 32 entries with unique values. These are the logical values accepted by the module's `setInput` operation; the module implements its transport. Invalid preset data is rejected. Older modules can omit the array, and users can enter custom values in decimal or hexadecimal.

Routine input selection:

```json
{
  "apiVersion": 1,
  "operation": "setInput",
  "monitorId": "unit-serial-123",
  "input": 15
}
```

```json
{ "ok": true, "message": "Request sent; picture has not been verified" }
```

Inspect (read-only, no hardware writes):

```json
{
  "apiVersion": 1,
  "operation": "inspect",
  "monitorId": "unit-serial-123",
  "confirmed": false
}
```

```json
{
  "ok": true,
  "recoveryState": "unknown",
  "message": "No reliable verification is available"
}
```

Explicit local recovery:

```json
{
  "apiVersion": 1,
  "operation": "recover",
  "monitorId": "unit-serial-123",
  "confirmed": true
}
```

```json
{
  "ok": true,
  "recoveryState": "applied",
  "message": "Guarded recovery complete"
}
```

`recoveryState` is `unknown`, `required`, `applied`, or `notApplicable`. Report `applied` when current evidence verifies the recovered state, and `unknown` when verification is unavailable. Recovery is an optional local action requiring user confirmation; routing uses `setInput`.

Failures:

```json
{ "ok": false, "message": "Selected monitor is unavailable" }
```

## Module implementation

- Identify the intended monitor before each command and reject unsupported inputs.
- Validate requests and confirmation requirements within the module.
- Verify supporting scripts and data; DisplayFlow checks the entry executable and manifest.
- Keep `enumerate` and `inspect` read-only. Use `setInput` for routine input selection and `recover` for an optional recovery action.
- Return clear errors and accurate status. Make recovery idempotent where possible.

TypeScript protocol types are in `packages/module-sdk/src/index.ts`. Modules can use any implementation language.

## Response fields and contributed panels

Use `fields` for a map of readable scalar diagnostic values. An optional `lastRecoveryAt` UNIX timestamp describes a completed recovery command and is displayed separately from `recoveryState`. It is not proof of persistence.

The optional `ui.panels` manifest contributes tabs and actions. Declare the `action` capability and implement the corresponding `action` request, including `panelId`, `actionId`, `monitorId`, and `confirmed`. These operations are local-only. See [extension development](extensions.md) and the complete [virtual provider](../samples/virtual-provider/README.md).
