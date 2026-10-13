/** Versioned manifest for a DisplayFlow module. */
export interface ModuleManifest {
  apiVersion: 1
  id: string
  name: string
  version: string
  description: string
  executable: string
  arguments: string[]
  capabilities: (
    'enumerate' | 'getInput' | 'setInput' | 'inspect' | 'recover' | 'action'
  )[]
  timeoutSeconds?: number
  recoveryTimeoutSeconds?: number
  ui?: {
    panels: ModulePanel[]
    recovery?: { label: string; description: string; confirmation: string }
  }
}
export interface ModulePanel {
  id: string
  title: string
  description: string
  actions: PanelAction[]
}
export interface PanelAction {
  id: string
  label: string
  description: string
  requiresConfirmation: boolean
}
export interface Monitor {
  id: string
  name: string
  provider: string
  detail: string
  inputPresets?: InputPreset[]
}
export interface InputPreset {
  value: number
  label: string
}
export type ModuleRequest =
  | { apiVersion: 1; operation: 'enumerate' }
  | { apiVersion: 1; operation: 'inspect'; monitorId: string }
  | { apiVersion: 1; operation: 'getInput'; monitorId: string }
  | { apiVersion: 1; operation: 'setInput'; monitorId: string; input: number }
  | { apiVersion: 1; operation: 'recover'; monitorId: string; confirmed: true }
  | {
      apiVersion: 1
      operation: 'action'
      monitorId: string
      panelId: string
      actionId: string
      confirmed: boolean
    }
export interface ModuleResponse {
  ok: boolean
  message?: string
  monitors?: Monitor[]
  input?: number
  activeConnection?: boolean
  /** Use unknown when the current recovery state cannot be verified. */
  recoveryState?: 'unknown' | 'required' | 'applied' | 'notApplicable'
  fields?: Record<string, string | number | boolean | null>
  lastRecoveryAt?: number
}
