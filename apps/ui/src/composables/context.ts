import { inject, type InjectionKey } from 'vue'
import type { useWorkspace } from './useWorkspace'
export const workspaceKey: InjectionKey<ReturnType<typeof useWorkspace>> =
  Symbol('workspace')
export function useApp() {
  const value = inject(workspaceKey)
  if (!value) throw new Error('Workspace provider missing')
  return value
}
