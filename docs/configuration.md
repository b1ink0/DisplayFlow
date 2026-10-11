# Configuration

## Monitor routes

A route represents one physical monitor. Each attachment names a Deskflow computer, a DisplayFlow agent, that agent's monitor ID, and the input used by that computer on the physical monitor.

Computer names must exactly match Deskflow. They can differ from OS hostnames. Pairing identities identify agents, not monitors; display names alone do not determine physical cabling. Verify input assignments using the monitor menu.

Standard MCCS input values are commonly:

| Input         | Decimal | Hex    |
| ------------- | ------- | ------ |
| DisplayPort 1 | 15      | `0x0F` |
| HDMI 1        | 17      | `0x11` |
| HDMI 2        | 18      | `0x12` |

These are suggestions, not a guarantee of monitor support. External providers may translate logical values into another transport. All attachments in a shared route must use compatible logical input values.

Confirm the currently visible computer using the route selector. After an accepted request, DisplayFlow records an assumed source. After restart, editing a route, importing assignments, or a failed request, reconfirm the source. A manual joystick switch, cable change, or power cycle also requires reconfirmation.

The agent driving the current input sends the next command. This supports monitors whose inactive inputs cannot accept commands. If that source is unavailable, DisplayFlow reports a failure rather than retrying a possibly accepted command.

## Deskflow

Enter the server log file only on the coordinator computer. Enable file logging in Deskflow at a level that includes events such as:

```text
switch from "Workstation" to "Laptop"
```

DisplayFlow skips historical entries, reads new transitions, and maps the destination explicitly. A destination with no monitor attachment leaves the input unchanged. Local enter/exit hooks are not required. Stop other cursor-driven input controllers when activating DisplayFlow as the coordinator.

## Peer pairing

Networking is disabled by default. The default IPv4 TCP port is 24862. Allow it through the OS firewall for your private LAN only; no router forwarding is needed. On Windows, **Settings → Local network access → Configure firewall access** requests administrator approval for a rule limited to the installed executable, saved TCP port, and local subnet on Private and Domain network profiles. The app itself runs without elevation. The same section can remove its rule. Save a changed port and update the rule if necessary. Public network profiles require your administrator to choose an appropriate policy.

On macOS 15 and later, allow the Local Network prompt when pairing. **Open network privacy settings** leads to the OS controls if access was declined. The bundle includes a purpose description for this access, as described in [Apple's local network privacy guide](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy). The app also needs permission to read your chosen log and external module files, and uses the OS keychain for pairing credentials. If access is denied, check the reported operation and its corresponding OS permission.

Enter each machine's local IPv4 address in Computers. Create an invitation on one machine and accept it on the other. Exchange it directly through a trusted channel: it includes a pinned certificate and a one-use token. It expires in ten minutes. Pairing creates reciprocal access; keep both agents running and resumed for routine input requests.

Receiving agents require locally enabled assignments authorizing the target monitor and input. Export/import assignments between paired machines rather than redefining every attachment. Review imported assignments before resuming. IP changes require pairing again in this version. Revoke a relationship on both computers to remove both directions.

## macOS BetterDisplay

For standard control, register the display's exact BetterDisplay name in Settings. BetterDisplay must be installed in Applications, running, and accepting CLI commands. Registered names must uniquely identify displays. This provider sends standard MCCS input selection. An external provider can implement a different transport while using the same route model.

## Background operation

Windows shows a notification-area icon; the window also appears on the taskbar while open. Closing the window hides it and keeps the agent active. macOS provides a template menu bar icon and a normal Dock item while the configuration window is open; hiding the window leaves the menu bar agent active. The menu provides **Open**, **Hide**, **Pause / resume**, and **Quit**.

Install the release application in its permanent location, then enable **Settings → Start at login**. This registers a per-user login startup entry and launches with `--background`, keeping the configuration window closed. Disable the same option to remove DisplayFlow's startup entry. Development builds and browser preview cannot enable startup.

Startup preserves the saved pause setting. Every process restart clears assumed input state: confirm the currently visible computer for each route before it can switch an input. Closing the window keeps the agent running; **Quit** stops it until the next manual launch or login.

The application stores settings in the OS application data directory and private credentials in the OS credential store. Browser preview configuration is separate and cannot operate hardware.

## Troubleshooting

- **Agent unavailable:** check that DisplayFlow is running, networking is enabled, and the configured port is allowed through the firewall. Check the computer's saved address after network changes.
- **Visible input unknown:** select the computer whose picture is currently visible in the route. Reconfirm after restarting the app or changing inputs manually.
- **Input command fails or has no effect:** verify the route's monitor ID and input value, then check hardware support with a manual request. A successful command reports transmission, not visual confirmation.
- **macOS input control unavailable:** check BetterDisplay is running with CLI access enabled and the registered display name is unique and exact.
- **Module changed:** review and add the updated module again. The saved executable hash and manifest must match its current files.
- **Keychain prompt:** DisplayFlow stores its TLS identity and peer credentials in the OS credential store. Approve access for the installed application. Replacing a development build can trigger another prompt.

Use Activity for the operation's error message. When reporting an issue, include the platform, provider, and relevant messages, with credentials and invitation tokens removed.
