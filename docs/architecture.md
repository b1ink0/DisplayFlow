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

Startup invalidates saved active-input assumptions. The user confirms the visible source using the route selector. Accepted switch requests update the assumed source; they do not verify the picture. A failed or timed-out request marks the source unknown. Reconfirm after a manual input change or monitor power loss.

The Deskflow adapter skips historical lines, follows new server transitions, coalesces destinations in each read batch, and ignores unmapped machines. Input requests are serialized. There is no background retry after a timeout. Slow commands can delay subsequent transitions; latest transitions are coalesced when reading resumes. Confirmation-needed errors appear in Activity.

## Security boundary

The native Rust backend owns hardware, process invocation, network, and storage. CSP and Tauri capabilities restrict the Vue webview's access to native operations. Modules run as the local user after a trust review. Their manifest and entry-executable hash are checked before each invocation; modules verify their own supporting files. Module UI contributions use declarative panels, text fields, and registered local actions.

Networking is off by default. Each device generates a self-signed TLS identity and stores its private key in the OS credential store. Invitations carry the server certificate and a one-use, ten-minute capability token. An invitation must arrive through a trusted channel because its certificate establishes trust. Pairing installs independent reciprocal credentials in OS storage; requests then require a known device ID and matching credential. Clients verify against the paired certificate with the fixed `displayflow.local` TLS name.

The peer API exposes inventory and locally authorized input selection. Module panel actions and recovery are local operations. Receiving agents must be resumed and have an enabled route authorizing monitor/input values. Messages are capped at 64 KiB, connections at eight, and exchanges have timeouts. Input setters are not retried after an uncertain response. Binding and invitation addresses use IPv4. Windows provides an administrator-approved firewall setup action in Settings.

Peer revocation is local; remove the peer on both computers to remove both directions. IP addressing and physical wiring assignments are configured manually. Re-pair to change an address or certificate.

## Extension points

- `providers/`: standard native monitor implementations.
- External process protocol: additional input providers and monitor tools.
- `deskflow.rs`: reads server transition events.
- `network.rs`: handles paired agent connections.

DisplayFlow runs in the user's desktop session and reads Deskflow server transition logs. The UI configures assignments and pairing; the tray provides background controls.
