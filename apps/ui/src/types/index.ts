import type { Monitor, ModuleManifest } from '@displayflow/module-sdk'
export type { Monitor }
export interface Attachment {
  computer: string
  deviceId: string
  monitorId: string
  input: number
  label: string
}
export interface Route {
  id: string
  name: string
  attachments: Attachment[]
  activeComputer: string | null
  enabled: boolean
}
export interface Peer {
  id: string
  name: string
  address: string
  port: number
  certificate: string
}
export interface MountedModule {
  manifest: ModuleManifest
  path: string
  executableHash: string
}
export interface Settings {
  revision: number
  schemaVersion: number
  deviceId: string
  computerName: string
  paused: boolean
  deskflowLog: string
  networkEnabled: boolean
  listenPort: number
  routes: Route[]
  peers: Peer[]
  modules: MountedModule[]
  manualMonitors: Monitor[]
}
export interface Activity {
  at: number
  level: string
  message: string
}
export interface Snapshot {
  settings: Settings
  monitors: Monitor[]
  activity: Activity[]
  platform: string
  agentStatus: string
  logStatus: string
}
