# Release distribution

Run `bun run build` on the target platform. Tauri writes bundles to `src-tauri/target/release/bundle`.

## Windows

The build produces NSIS and MSI installers. Publisher signing is not configured. To distribute signed installers, configure your own signing certificate using [Tauri's Windows signing guide](https://v2.tauri.app/distribute/sign/windows/).

## macOS

The build produces an application bundle and disk image. `tauri.macos.conf.json` configures ad-hoc signing for local builds. Verify the final application bundle with:

```sh
codesign --verify --deep --strict /path/to/DisplayFlow.app
```

For public distribution, configure your Apple Developer signing identity and notarization credentials. Ad-hoc signing does not provide a verified publisher identity. Follow [Tauri's macOS signing guide](https://v2.tauri.app/distribute/sign/macos/).

## Upgrade

Quit the existing application before replacing it. Install the new build in the same permanent location. Settings and paired credentials are stored separately from the application bundle.

After relaunching, check that agents are listening and confirm the visible computer for each monitor route. If you change the executable location or listening port on Windows, update firewall access in Settings.
