import { ref } from 'vue'

type Mode = 'dark' | 'light'
const mode = ref<Mode>('dark')
const key = 'displayflow.appearance'
export function initializeAppearance() {
  try {
    if (localStorage.getItem(key) === 'light') mode.value = 'light'
  } catch {
    /* OS storage may be unavailable in a restricted webview. */
  }
  document.documentElement.classList.toggle('dark', mode.value === 'dark')
}
export function useAppearance() {
  function toggle() {
    mode.value = mode.value === 'dark' ? 'light' : 'dark'
    document.documentElement.classList.toggle('dark', mode.value === 'dark')
    try {
      localStorage.setItem(key, mode.value)
    } catch {
      /* The current session still retains its appearance. */
    }
  }
  return { mode, toggle }
}
