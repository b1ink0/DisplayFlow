# Architecture

```text
Deskflow server log
        |
        v
DisplayFlow coordinator (server computer)
  destination -> physical monitor route
  current input -> source attachment
        |
        +-- source is local --> input provider
        |
        +-- source is peer ---> pinned TLS + scoped credential
                                  |
                                  v
                             peer local agent
                             authorized assignments
                                  |
                                  v
                             input provider
                                  |
                   +--------------+---------------+
                   |                              |
           standard OS / CLI API          external trusted process
           Windows DDC / BetterDisplay    module executable
```

The monitor receives its input command through the source attachment that currently drives it. Cursor destination is separate from monitor input state. Every physical monitor has its own route with two or more attachments; an attachment maps an exact Deskflow computer name to an agent identity, that agent's monitor ID, and this physical monitor's destination input value.

## State and failures

On startup, a participant fetches the running coordinator's current input state. After a coordinator restart, the user confirms the visible source using the route selector, or enables optional read-only detection for a reliable provider. Detection queries assigned agents and accepts only a unique, consistent input value or verified active-connection response. Missing evidence preserves manual confirmation; contradictory evidence clears the source. Accepted switch requests update the assumed source; they do not verify the picture. A failed or timed-out request marks the source unknown.

Each shared route has one coordinator. Routes without an explicit coordinator on the configured Deskflow server select that device on startup; other agents discover the unique paired coordinator with matching assignments. A route selects its creating computer by default. All assigned agents must pair with the coordinator; agents do not need direct connections to every other agent.

Detection settings, manual confirmations, getter results and switch outcomes are shared through the coordinator. A detection request from any assigned agent delegates to that coordinator, which probes all attachments concurrently and distributes one result. Only the coordinator performs periodic detection and serializes route switching. The source agent still sends the physical input command. Pause state, local permissions and reviewed attachment mappings remain local.

Shared-state messages authenticate through the existing pinned TLS connection and must match the route ID and a hash of its device/monitor/computer/input assignments. They cannot create routes or broaden local monitor permissions. A coordinator process identifier and increasing sequence reject older messages; reconnecting agents pull the current state every three seconds. A coordinator restart clears its input assumption, and followers accept the new process identifier through a fresh pull. Pending or unavailable synchronization appears in the route UI. Failed switch requests are not retried by the synchronization loop.

The Deskflow adapter skips historical lines, follows new server transitions, coalesces destinations in each read batch, and ignores unmapped machines. Input requests are serialized. There is no background retry after a timeout. Slow commands can delay subsequent transitions; latest transitions are coalesced when reading resumes. Confirmation-needed errors appear in Activity.

## Security boundary

The native Rust backend owns hardware, process invocation, network, and storage. CSP and Tauri capabilities restrict the Vue webview's access to native operations. Modules run as the local user after a trust review. Their manifest and entry-executable hash are checked before each invocation; modules verify their own supporting files. Module UI contributions use declarative panels, text fields, and registered local actions.

Networking is off by default. Each device generates a self-signed TLS identity. Its private key and independent reciprocal pairing credentials live in an authenticated encrypted vault, protected by one installation-specific key in the OS credential store. The running process caches this vault. Invitations carry the server certificate and a one-use, ten-minute capability token. An invitation must arrive through a trusted channel because its certificate establishes trust. Requests require a known device ID and matching credential. Clients verify against the paired certificate with the fixed `displayflow.local` TLS name.

The peer API exposes inventory, read-only input queries for locally assigned monitors, and locally authorized input selection. Module panel actions and recovery are local operations. Receiving agents must be resumed and have an enabled route authorizing monitor/input values. Messages are capped at 64 KiB, connections at eight, and exchanges have timeouts. Input setters are not retried after an uncertain response. Binding and invitation addresses use IPv4. Windows provides an administrator-approved firewall setup action in Settings.

Peer revocation is local; remove the peer on both computers to remove both directions. IP addressing and physical wiring assignments are configured manually. Re-pair to change an address or certificate.

## Extension points

- `providers/`: standard native monitor implementations.
- External process protocol: additional input providers and monitor tools.
- `deskflow.rs`: reads server transition events.
- `network.rs`: handles paired agent connections.

DisplayFlow runs in the user's desktop session and reads Deskflow server transition logs. The UI configures assignments and pairing; the tray provides background controls.
