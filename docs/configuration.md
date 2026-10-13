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

The input dropdown offers common MCCS presets for built-in providers, or presets supplied by an external module. Choose **Custom value...** to enter a decimal number or hexadecimal value prefixed with `0x`, from 0 to 65535. Input labels are editable. Selecting a value does not change the command protocol: standard control uses VCP `0x60`, while an alternative transport requires a provider implementing it. The standard mappings are listed in [ddcutil's input-source reference](https://www.ddcutil.com/vcpinfo_output/).

Create a separate route for each physical monitor. The current limits are 64 routes, with 2-16 computer attachments per route.

Confirm the currently visible computer once using the route selector on any assigned computer. The coordinator shares this confirmation and subsequent requested sources. A restarted agent fetches the live coordinator's source; a coordinator restart clears it until a reliable getter or manual confirmation supplies it again. Changed assignments, a failed request, a manual joystick switch, a cable change or a power cycle can also require reconfirmation.

Enable **Detect the visible input automatically** in the route editor to read input evidence from its assigned agents. Detection runs every ten seconds while switching is resumed, and when a switch needs an unknown source. **Detect now** performs one read-only pass. All participating agents need a version supporting input reads, and external modules must implement `getInput`. Built-in providers read standard VCP `0x60`.

Only a unique, consistent result selects a computer. Some devices report a stale input, accept queries on inactive inputs, or cannot read their current input at all. Leave detection off for an unreliable getter. An unsupported read preserves an existing manual confirmation or switch assumption; conflicting replies clear it. The manual visible-computer selector remains available.

The agent driving the current input sends the next command. This supports monitors whose inactive inputs cannot accept commands. If that source is unavailable, DisplayFlow reports a failure rather than retrying a possibly accepted command.

## Deskflow

Enter the server log file only on the coordinator computer. Enable file logging in Deskflow at a level that includes events such as:

```text
switch from "Workstation" to "Laptop"
```

DisplayFlow skips historical entries, reads new transitions, and maps the destination explicitly. A destination with no monitor attachment leaves the input unchanged. Local enter/exit hooks are not required. Stop other cursor-driven input controllers when activating DisplayFlow as the coordinator.

## Peer pairing

Networking is disabled by default. The default IPv4 TCP port is 24862. Allow it through the OS firewall for your private LAN only; no router forwarding is needed. On Windows, **Settings -> Local network access -> Configure firewall access** requests administrator approval for a rule limited to the installed executable, saved TCP port, and local subnet on Private and Domain network profiles. The app itself runs without elevation. The same section can remove its rule. Save a changed port and update the rule if necessary. Public network profiles require your administrator to choose an appropriate policy.

On macOS 15 and later, allow the Local Network prompt when pairing. **Open network privacy settings** leads to the OS controls if access was declined. The bundle includes a purpose description for this access, as described in [Apple's local network privacy guide](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy). The app also needs permission to read your chosen log and external module files, and uses the OS keychain for pairing credentials. If access is denied, check the reported operation and its corresponding OS permission.

Enter each machine's local IPv4 address in Computers. Create an invitation on one machine and accept it on the other. Exchange it directly through a trusted channel: it includes a pinned certificate and a one-use token. It expires in ten minutes. Pairing creates reciprocal access; keep both agents running and resumed for routine input requests.

Receiving agents require locally enabled assignments authorizing the target monitor and input. Export/import assignments between paired machines rather than redefining every attachment. Review imported assignments before resuming. IP changes require pairing again in this version. Revoke a relationship on both computers to remove both directions.

### Shared route detection

After importing and reviewing matching assignments, choose the route's coordinator once. Routes without a coordinator use the configured Deskflow server when the other agents discover it. Each assigned agent must be paired with that coordinator.

Enable **Detect the visible input automatically** from any computer's route editor. The coordinator distributes this setting to all assigned agents and performs periodic read-only detection while resumed. **Detect now** is available even when periodic detection is off; one click asks the coordinator to query all assigned computers and share the result. The visible-input selector also shares a manual confirmation, so it does not need to be repeated on each computer.

**Refresh monitors** refreshes inventory. Getter support varies by monitor; unavailable or conflicting evidence can require manual confirmation. A missing reply alone cannot identify a particular input. Pending synchronization is shown on the route, and reconnecting agents fetch the latest state automatically. Physical assignments still require review and transfer when their wiring or monitor identifiers change.

## macOS BetterDisplay

For standard control, register the display's exact BetterDisplay name in Settings. BetterDisplay must be installed in Applications, running, and accepting CLI commands. Registered names must uniquely identify displays. This provider sends standard MCCS input selection. An external provider can implement a different transport while using the same route model.

## Background operation

Windows shows a notification-area icon; the window also appears on the taskbar while open. Closing the window hides it and keeps the agent active. macOS provides a template menu bar icon and a normal Dock item while the configuration window is open; hiding the window leaves the menu bar agent active. The menu provides **Open**, **Hide**, **Pause / resume**, and **Quit**.

Install the release application in its permanent location, then enable **Settings -> Start at login**. This registers a per-user login startup entry and launches with `--background`, keeping the configuration window closed. Disable the same option to remove DisplayFlow's startup entry. Development builds and browser preview cannot enable startup.

Startup preserves the saved pause setting. A restarted agent clears its cached source and fetches the coordinator's current state. After the coordinator restarts, a reliable getter or one shared manual confirmation must establish the visible source. Closing the window keeps the agent running; **Quit** stops it until the next manual launch or login.

The application stores settings and an authenticated, encrypted credential vault in the OS application data directory. A single installation-specific encryption key is stored in the OS credential store. Credentials are cached for the running process; routine routing does not repeatedly read the OS store. Keep the vault and its OS key together when backing up an installation. Browser preview configuration is separate and cannot operate hardware.

## Troubleshooting

- **Agent unavailable:** check that DisplayFlow is running, networking is enabled, and the configured port is allowed through the firewall. Check the computer's saved address after network changes.
- **Visible input unknown:** select the computer whose picture is currently visible in the route. Reconfirm after restarting the coordinator or changing inputs manually if detection cannot identify the source.
- **Input command fails or has no effect:** verify the route's monitor ID and input value, then check hardware support with a manual request. A successful command reports transmission, not visual confirmation.
- **macOS input control unavailable:** check BetterDisplay is running with CLI access enabled and the registered display name is unique and exact.
- **Module changed:** review and add the updated module again. The saved executable hash and manifest must match its current files.
- **Keychain prompt:** allow DisplayFlow to access its credential vault key. A locked login Keychain or a changed app signing identity can require approval again. For source builds, use the consistent signing identity described in the README.

Use Activity for the operation's error message. When reporting an issue, include the platform, provider, and relevant messages, with credentials and invitation tokens removed.
