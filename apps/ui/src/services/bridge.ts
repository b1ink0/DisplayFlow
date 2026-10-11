import { invoke, isTauri } from '@tauri-apps/api/core'
import type { Settings } from '@/types'
export const desktop = isTauri()
function previewSettings(): Settings {
  try {
    const saved = localStorage.getItem('displayflow-preview')
    if (saved) return JSON.parse(saved) as Settings
  } catch {
    /* Start with an empty browser workspace. */
  }
  return {
    schemaVersion: 1,
    revision: 0,
    deviceId: 'browser-preview',
    computerName: 'This computer',
    paused: true,
    deskflowLog: '',
    networkEnabled: false,
    listenPort: 24862,
    routes: [],
    peers: [],
    modules: [],
    manualMonitors: [],
  }
}
let preview = previewSettings()
export async function call<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (desktop) return invoke<T>(command, args)
  if (command === 'snapshot')
    return {
      settings: structuredClone(preview),
      monitors: [],
      activity: [],
      platform: 'browser',
      agentStatus: 'Desktop app required',
      logStatus: 'Desktop app required',
    } as T
  if (command === 'save_settings') {
    preview = structuredClone(args.settings as Settings)
    localStorage.setItem('displayflow-preview', JSON.stringify(preview))
    return undefined as T
  }
  throw new Error(
    'Open the Tauri desktop app to use monitors, pairing, or external modules.',
  )
}
