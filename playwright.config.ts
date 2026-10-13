import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests/ui',
  testMatch: '**/*.spec.ts',
  fullyParallel: true,
  workers: 2,
  timeout: 30_000,
  reporter: 'list',
  use: {
    baseURL: 'http://127.0.0.1:1438',
    viewport: { width: 1280, height: 900 },
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'bun run --cwd apps/ui dev --port 1438',
    url: 'http://127.0.0.1:1438',
    env: { DISPLAYFLOW_UI_FIXTURE: '1' },
    reuseExistingServer: false,
  },
})
