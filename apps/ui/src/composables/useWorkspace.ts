import { computed, onMounted, onUnmounted, ref } from 'vue'
import { call } from '@/services/bridge'
import type { Settings, Snapshot } from '@/types'

export function useWorkspace() {
  const snapshot = ref<Snapshot | null>(null)
  const busy = ref(false)
  const error = ref('')
  const notice = ref('')
  const settings = computed(() => snapshot.value?.settings)
  let timer: ReturnType<typeof setInterval> | undefined
  let refreshing = false
  async function refresh(inventory = false) {
    if (refreshing) return
    refreshing = true
    try {
      snapshot.value = await call<Snapshot>('snapshot', { refresh: inventory })
    } finally {
      refreshing = false
    }
  }
  async function run<T>(
    action: () => Promise<T>,
    success = '',
  ): Promise<T | undefined> {
    busy.value = true
    error.value = ''
    notice.value = ''
    try {
      const result = await action()
      notice.value = success
      await refresh()
      return result
    } catch (e) {
      error.value = String(e instanceof Error ? e.message : e)
      return undefined
    } finally {
      busy.value = false
    }
  }
  async function save(next: Settings) {
    await call('save_settings', { settings: next })
    await refresh()
  }
  async function mutate(fn: (next: Settings) => void) {
    await refresh()
    if (!settings.value) return
    const next = JSON.parse(JSON.stringify(settings.value)) as Settings
    fn(next)
    await save(next)
  }
  onMounted(async () => {
    // Configuration must remain accessible while an external provider waits
    // for an OS file-access prompt. Discovery runs separately from first load.
    await run(() => refresh())
    void refresh(true).catch((e: unknown) => {
      error.value = String(e)
    })
    timer = setInterval(() => {
      if (!busy.value)
        void refresh().catch((e: unknown) => {
          error.value = String(e)
        })
    }, 2500)
  })
  onUnmounted(() => clearInterval(timer))
  return { snapshot, settings, busy, error, notice, run, refresh, save, mutate }
}
