# Local setup API

DisplayFlow supports local provisioning alongside its configuration window. It uses the same module trust checks, settings validation, pairing protocol, and OS startup manager as the UI. It is not available through the peer network protocol.

Launch the installed release executable with `--background --setup-file /absolute/path/request.json`. On macOS, use the executable inside the application bundle. An already running instance receives the request. The response is written beside the request as `request.result.json`; that file must not already exist. Use a fresh request filename for each operation.

Requests and responses may contain pairing invitations or account configuration. Store them in a directory accessible only to your user, exchange invitations through a trusted channel, and remove them after provisioning. Responses are created with mode `600` on macOS; Windows uses the containing directory's ACL. Invitations are not printed to the activity log.

Each request has `apiVersion: 1` and one `operation`. Unknown fields and requests larger than 64 KiB are rejected. The response has `ok` and either `result` or `message`.

```json
{
  "apiVersion": 1,
  "operation": {
    "type": "configure",
    "computerName": "Workstation",
    "networkEnabled": true
  }
}
```

| Type        | Fields                                                                                        | Result                                                |
| ----------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `status`    | Optional `refresh`                                                                            | Settings, inventory, activity, and startup status     |
| `configure` | Optional `computerName`, `deskflowLog`, `networkEnabled`, `listenPort`, `routes`              | Validated configuration saved                         |
| `mount`     | Absolute module directory `path`, reviewed SHA-256 `executableHash`, explicit `trusted: true` | External module mounted                               |
| `invite`    | Local IPv4 `address`                                                                          | One-use invitation, valid for ten minutes             |
| `pair`      | Invitation JSON string `invitation`, own local IPv4 `address`                                 | Reciprocal peer pairing                               |
| `inventory` | Paired `peerId`                                                                               | Read-only peer monitor inventory                      |
| `firewall`  | `enabled` (Windows only)                                                                      | Firewall setup or removal with administrator approval |
| `startup`   | `enabled`                                                                                     | OS login startup status                               |

`status` uses cached inventory by default. Set `refresh: true` to discover displays and invoke mounted providers. The default remains available while an external module is waiting for file-access permission.

The Windows-only `firewall` operation accepts `enabled` and invokes the same administrator approval and private-LAN rule setup/removal as the UI.

Configuration pauses routing and clears assumed active inputs. Confirm the visible computer and resume in the UI when ready. Mounting requires the executable hash to match; the app checks the manifest and executable again when it runs the module. Enabling startup requires an installed release build in a permanent location.

Example module mount:

```json
{
  "apiVersion": 1,
  "operation": {
    "type": "mount",
    "path": "/absolute/path/provider",
    "executableHash": "SHA256_HEX_OF_REVIEWED_EXECUTABLE",
    "trusted": true
  }
}
```
