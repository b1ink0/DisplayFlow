import type { Snapshot, Settings, Monitor } from '../../apps/ui/src/types'
import manifest from '../../samples/virtual-provider/manifest.json'
import type { ModuleManifest } from '@displayflow/module-sdk'

const monitors: Monitor[] = [
  {
    id: 'ddc:example-desk',
    name: 'Shared display',
    provider: 'DDC/CI',
    detail: 'Input selection available',
  },
  {
    id: 'module:virtual-provider:sample-display',
    name: 'Virtual display',
    provider: 'virtual-provider',
    detail: 'Simulated monitor for module development',
  },
]

const snapshot: Snapshot = {
  platform: 'windows',
  agentStatus: 'Listening',
  logStatus: 'Watching new transitions',
  monitors,
  settings: {
    schemaVersion: 1,
    revision: 1,
    deviceId: 'example-workstation',
    computerName: 'Workstation',
    paused: false,
    deskflowLog: 'C:\\Deskflow\\server.log',
    networkEnabled: true,
    listenPort: 24862,
    routes: [
      {
        id: 'example-desk',
        name: 'Shared desk display',
        enabled: true,
        autoDetect: true,
        activeComputer: 'Workstation',
        attachments: [
          {
            computer: 'Workstation',
            deviceId: 'example-workstation',
            monitorId: 'ddc:example-desk',
            input: 18,
            label: 'HDMI 2',
          },
          {
            computer: 'Studio',
            deviceId: 'example-studio',
            monitorId: 'betterdisplay:Shared display',
            input: 15,
            label: 'DisplayPort',
          },
          {
            computer: 'Notebook',
            deviceId: 'example-notebook',
            monitorId: 'ddc:example-desk',
            input: 17,
            label: 'HDMI 1',
          },
        ],
      },
    ],
    peers: [
      {
        id: 'example-studio',
        name: 'Studio',
        address: '192.0.2.20',
        port: 24862,
        certificate: 'example-certificate',
      },
      {
        id: 'example-notebook',
        name: 'Notebook',
        address: '192.0.2.30',
        port: 24862,
        certificate: 'example-certificate',
      },
    ],
    modules: [
      {
        manifest: { ...manifest, executable: 'provider.exe' } as ModuleManifest,
        path: 'C:\\Modules\\virtual-provider',
        executableHash: '0'.repeat(64),
      },
    ],
    manualMonitors: [],
  },
  activity: [
    {
      at: 1767260640,
      level: 'success',
      message: 'Shared desk display → Workstation. Input request sent.',
    },
    {
      at: 1767260580,
      level: 'success',
      message: 'Shared desk display → Studio. Input request sent.',
    },
    {
      at: 1767260520,
      level: 'success',
      message: 'Shared desk display → Notebook. Input request sent.',
    },
    {
      at: 1767260460,
      level: 'info',
      message: 'Deskflow log attached; historical transitions skipped.',
    },
    {
      at: 1767260400,
      level: 'info',
      message: 'Local agent listening. Two computers paired.',
    },
  ],
}

interface FixtureControl {
  calls: { command: string; args: Record<string, unknown> }[]
  failNext: boolean
  delayMs: number
}
const control: FixtureControl = { calls: [], failNext: false, delayMs: 0 }
Object.assign(window, { displayflowFixture: control })
const query = new URLSearchParams(location.search)
if (query.has('mac')) snapshot.platform = 'macos'
if (query.has('empty')) {
  snapshot.settings.routes = []
  snapshot.settings.peers = []
  snapshot.settings.modules = []
  snapshot.monitors = []
  snapshot.activity = []
}
if (query.has('large')) {
  const route = snapshot.settings.routes[0]!
  snapshot.settings.routes = Array.from({ length: 64 }, (_, i) => ({
    ...structuredClone(route),
    id: 'route-' + i,
    name:
      'Display ' +
      i +
      ' — 長い名前の共有ディスプレイ ' +
      'workspace '.repeat(6),
  }))
  snapshot.monitors = Array.from({ length: 1500 }, (_, i) => ({
    ...monitors[0]!,
    id: 'ddc:display-' + i,
    name: 'Display ' + String(i).padStart(4, '0'),
  }))
  snapshot.settings.peers = Array.from({ length: 150 }, (_, i) => ({
    ...snapshot.settings.peers[0]!,
    id: 'peer-' + i,
    name: 'Computer ' + i,
  }))
  snapshot.activity = Array.from({ length: 150 }, (_, i) => ({
    at: 1767260640 - i,
    level: i % 3 === 0 ? 'error' : 'info',
    message: 'Input event ' + i + ' — ' + '長いメッセージ '.repeat(20),
  }))
}

export function isTauri() {
  return !query.has('preview')
}

/** Documentation renderer: all operations use in-memory example data. */
export async function invoke<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  control.calls.push({ command, args: JSON.parse(JSON.stringify(args)) })
  if (control.failNext) {
    control.failNext = false
    throw new Error('The provider is unavailable. Check the local agent.')
  }
  if (control.delayMs && command !== 'snapshot')
    await new Promise((resolve) => setTimeout(resolve, control.delayMs))
  let result: unknown
  switch (command) {
    case 'snapshot':
      result = structuredClone(snapshot)
      break
    case 'select_destination': {
      const route = snapshot.settings.routes.find((r) => r.id === args.routeId)
      if (route) route.activeComputer = String(args.destination)
      break
    }
    case 'detect_input':
      result = { status: 'unknown', message: 'No usable input evidence.' }
      break
    case 'confirm_input': {
      const route = snapshot.settings.routes.find((r) => r.id === args.routeId)
      if (route) route.activeComputer = String(args.computer)
      break
    }
    case 'create_invitation':
      result = '{"version":1,"example":true}'
      break
    case 'pair_peer':
      break
    case 'remove_peer':
      snapshot.settings.peers = snapshot.settings.peers.filter(
        (p) => p.id !== args.id,
      )
      break
    case 'inspect_module':
      result = snapshot.settings.modules[0]
      break
    case 'mount_module':
      break
    case 'unmount_module':
      snapshot.settings.modules = snapshot.settings.modules.filter(
        (m) => m.manifest.id !== args.id,
      )
      break
    case 'configure_firewall':
      result = 'Local network rule configured'
      break
    case 'open_network_permissions':
      break
    case 'set_startup':
      result = { enabled: args.enabled, canEnable: true }
      break
    case 'save_settings':
      snapshot.settings = JSON.parse(JSON.stringify(args.settings)) as Settings
      break
    case 'startup_status':
      result = { enabled: true, canEnable: true }
      break
    case 'peer_inventory':
      result = [monitors[0]]
      break
    case 'module_action':
    case 'panel_action':
      result = {
        ok: true,
        recoveryState: 'notApplicable',
        message: 'Virtual display is ready.',
        fields: {
          Display: 'Virtual display',
          Input: 'HDMI 2',
          Transport: 'Simulated',
          'Request count': 3,
        },
      }
      break
    default:
      throw new Error(`Unsupported screenshot operation: ${command}`)
  }
  return result as T
}
