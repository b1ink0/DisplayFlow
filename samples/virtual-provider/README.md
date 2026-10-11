# Virtual provider

A complete module example that runs without hardware access. Build from the DisplayFlow repository root:

```sh
bun run module:sample:build
```

Add `samples/virtual-provider/dist` through Modules → Review → Trust and add. Refresh inventory. The module adds a virtual display and a **Sample tools** tab. Its status action displays values; reset requires confirmation. Its input setter changes only its ignored local sample state.

Copy this source into your own external module repository to implement a real provider. Replace the simulated transport and keep the JSON protocol and manifest. The main application does not need a source change to load a new module or its declarative panels. See [extension development](../../docs/extensions.md).
