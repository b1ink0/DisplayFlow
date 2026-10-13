import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import ui from '@nuxt/ui/vite'
import { productUi } from './src/lib/theme'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig(({ command }) => {
  const fixture = process.env.DISPLAYFLOW_UI_FIXTURE === '1'
  if (command === 'build' && fixture)
    throw new Error('UI fixtures cannot be included in a production build')
  return {
    plugins: [
      vue(),
      tailwindcss(),
      ui({
        router: false,
        colorMode: false,
        components: false,
        autoImport: false,
        ui: productUi,
        theme: { defaultVariants: { color: 'neutral', size: 'sm' } },
        icon: { clientBundle: { scan: true } },
      }),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
        ...(fixture
          ? {
              '@tauri-apps/api/core': fileURLToPath(
                new URL('../../tests/ui/bridge.ts', import.meta.url),
              ),
            }
          : {}),
      },
      dedupe: ['vue', 'reka-ui'],
    },
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_ENV_'],
    server: { watch: { ignored: ['**/src-tauri/**'] } },
    build: {
      target:
        process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15',
    },
  }
})
