import { test, expect } from '@playwright/test'

test('one-shot detection stays available when automatic detection is off and confirmation uses a shared command', async ({
  page,
}) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  await page.getByRole('switch', { name: 'Automatic input detection' }).click()
  await page.getByRole('button', { name: 'Save route', exact: true }).click()
  await expect(page.getByRole('dialog')).not.toBeVisible()
  await page.getByRole('button', { name: 'Detect now', exact: true }).click()
  await page
    .getByRole('combobox', { name: 'Confirm visible computer', exact: true })
    .click()
  await page.getByRole('option', { name: 'Studio', exact: true }).click()
  await expect(
    page.getByRole('combobox', {
      name: 'Confirm visible computer',
      exact: true,
    }),
  ).toContainText('Studio')
  const commands = await page.evaluate(
    () =>
      (
        window as unknown as {
          displayflowFixture: {
            calls: { command: string; args: Record<string, unknown> }[]
          }
        }
      ).displayflowFixture.calls,
  )
  expect(commands.filter((c) => c.command === 'detect_input')).toHaveLength(1)
  expect(commands.filter((c) => c.command === 'confirm_input')).toEqual([
    {
      command: 'confirm_input',
      args: { routeId: 'example-desk', computer: 'Studio' },
    },
  ])
})
import type { Page } from '@playwright/test'

async function navigate(page: Page, name: string) {
  if ((page.viewportSize()?.width ?? 1280) < 1024) {
    await page
      .getByRole('button', { name: 'Open navigation', exact: true })
      .click()
    await page.getByRole('dialog').getByText(name, { exact: true }).click()
  } else await page.locator('aside').getByText(name, { exact: true }).click()
  await expect(
    page.getByRole('heading', { level: 1, name, exact: true }),
  ).toBeVisible()
}

test('unsaved settings and pairing text survive navigation and refresh', async ({
  page,
}) => {
  await page.goto('/')
  await navigate(page, 'Settings')
  await page
    .getByLabel('This computer’s Deskflow name')
    .fill('Unsaved workstation')
  await navigate(page, 'Computers')
  await page.getByLabel('This computer’s local IP address').fill('192.0.2.55')
  await page.getByLabel('Invitation to accept').fill('unsaved invitation')
  await navigate(page, 'Settings')
  await expect(page.getByLabel('This computer’s Deskflow name')).toHaveValue(
    'Unsaved workstation',
  )
  await page.waitForTimeout(2600)
  await expect(page.getByLabel('This computer’s Deskflow name')).toHaveValue(
    'Unsaved workstation',
  )
  await navigate(page, 'Computers')
  await expect(page.getByLabel('Invitation to accept')).toHaveValue(
    'unsaved invitation',
  )
})

test('route editor preserves custom hexadecimal validation and accessible controls', async ({
  page,
}) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  const dialog = page.getByRole('dialog')
  await expect(
    dialog.getByRole('switch', { name: 'Automatic input detection' }),
  ).toBeVisible()
  await dialog
    .getByRole('combobox', { name: 'Input preset', exact: true })
    .first()
    .click()
  await page.getByRole('option', { name: 'Custom value…', exact: true }).click()
  const input = dialog.getByRole('textbox', { name: 'Custom input value' })
  await input.fill('0x91')
  await expect(dialog.locator('#input-0-hint')).toContainText('145 decimal')
  await input.fill('65536')
  await expect(input).toHaveAttribute('aria-invalid', 'true')
  await dialog.getByRole('button', { name: 'Save route', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Each input')
  await expect(dialog).toBeVisible()
  await input.fill('17')
  await dialog.getByRole('button', { name: 'Save route', exact: true }).click()
  await expect(dialog).not.toBeVisible()
})

test('confirmation gates module actions and keeps results across navigation', async ({
  page,
}) => {
  await page.goto('/')
  await navigate(page, 'Sample tools')
  await page
    .getByRole('button', { name: 'Run Reset sample state', exact: true })
    .click()
  const dialog = page.getByRole('dialog')
  await expect(
    dialog.getByRole('button', { name: 'Run action', exact: true }),
  ).toBeDisabled()
  await dialog.getByRole('checkbox').check()
  await dialog.getByRole('button', { name: 'Run action', exact: true }).click()
  await expect(
    page.getByText('Virtual display is ready.', { exact: true }),
  ).toBeVisible()
  await navigate(page, 'Activity')
  await navigate(page, 'Sample tools')
  await expect(
    page.getByText('Virtual display is ready.', { exact: true }),
  ).toBeVisible()
  expect(
    await page.evaluate(() =>
      (
        window as unknown as {
          displayflowFixture: {
            calls: { command: string; args: { confirmed?: boolean } }[]
          }
        }
      ).displayflowFixture.calls.some(
        (c) => c.command === 'panel_action' && c.args.confirmed,
      ),
    ),
  ).toBe(true)
})

test('in-flight actions complete when the view is hidden', async ({ page }) => {
  await page.goto('/')
  await navigate(page, 'Sample tools')
  await page.evaluate(() => {
    ;(
      window as unknown as { displayflowFixture: { delayMs: number } }
    ).displayflowFixture.delayMs = 800
  })
  await page
    .getByRole('button', { name: 'Run Show sample status', exact: true })
    .click()
  await navigate(page, 'Activity')
  await expect(
    page.getByRole('button', { name: 'Pause', exact: true }),
  ).toBeDisabled()
  await expect(
    page.getByRole('button', { name: 'Pause', exact: true }),
  ).toBeEnabled()
  await navigate(page, 'Sample tools')
  await expect(
    page.getByText('Virtual display is ready.', { exact: true }),
  ).toBeVisible()
})

test('disclosure keyboard behavior and secondary actions are independent', async ({
  page,
}) => {
  await page.goto('/')
  const heading = page.getByRole('button', {
    name: /Shared desk display Last confirmed/,
  })
  await expect(heading).toHaveAttribute('aria-expanded', 'true')
  await heading.focus()
  await page.keyboard.press('Space')
  await expect(heading).toHaveAttribute('aria-expanded', 'false')
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  await page.getByRole('button', { name: 'Close dialog', exact: true }).click()
  await expect(heading).toHaveAttribute('aria-expanded', 'false')
  await heading.focus()
  await page.keyboard.press('Enter')
  await expect(heading).toHaveAttribute('aria-expanded', 'true')
})

test('large datasets render bounded rows and responsive navigation', async ({
  page,
}) => {
  await page.goto('/?large=1')
  await expect(page.locator('main article')).toHaveCount(8)
  await page.getByRole('button', { name: /Available on this computer/ }).click()
  await expect(page.getByText('Display 0049', { exact: true })).toBeVisible()
  await expect(page.getByText('Display 0050', { exact: true })).toHaveCount(0)
  await navigate(page, 'Activity')
  await expect(page.locator('main ol li')).toHaveCount(25)
  await navigate(page, 'Computers')
  await expect(page.locator('main article')).toHaveCount(10)
  await page.setViewportSize({ width: 390, height: 844 })
  await navigate(page, 'Monitor routes')
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true)
})

test('narrow dialogs, theme persistence, empty and error states', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/?empty=1')
  await expect(
    page.getByText('Add your shared monitor', { exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Use light appearance' }).click()
  await page.reload()
  await expect(page.locator('html')).not.toHaveClass(/dark/)
  await page.getByRole('button', { name: 'New route', exact: true }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true)
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).not.toBeVisible()
  await page.goto('/')
  await page.evaluate(() => {
    ;(
      window as unknown as { displayflowFixture: { failNext: boolean } }
    ).displayflowFixture.failNext = true
  })
  await page.getByRole('button', { name: 'Detect now', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('provider is unavailable')
  await page.getByRole('button', { name: 'Dismiss error' }).click()
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('hidden route drafts survive navigation; explicit cancel discards them', async ({
  page,
}) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  await page
    .getByRole('dialog')
    .getByLabel('Monitor name')
    .fill('Unsaved display name')
  await page.getByRole('button', { name: 'Close dialog', exact: true }).click()
  await navigate(page, 'Activity')
  await navigate(page, 'Monitor routes')
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  await expect(page.getByRole('dialog').getByLabel('Monitor name')).toHaveValue(
    'Unsaved display name',
  )
  await page
    .getByRole('dialog')
    .getByRole('button', { name: 'Cancel', exact: true })
    .click()
  await page.getByRole('button', { name: 'Edit route', exact: true }).click()
  await expect(page.getByRole('dialog').getByLabel('Monitor name')).toHaveValue(
    'Shared desk display',
  )
})

test('browser preview saves a route without cloning reactive proxies', async ({
  page,
}) => {
  await page.goto('/?preview=1')
  await page.getByRole('button', { name: 'New route', exact: true }).click()
  const dialog = page.getByRole('dialog')
  await dialog.getByLabel('Monitor name').fill('Preview display')
  const computers = dialog.getByRole('textbox', {
    name: 'Exact Deskflow name',
    exact: true,
  })
  await computers.nth(0).fill('First computer')
  await computers.nth(1).fill('Second computer')
  const monitors = dialog.getByPlaceholder('Exact provider monitor ID')
  await monitors.nth(0).fill('ddc:first')
  await monitors.nth(1).fill('ddc:second')
  await dialog.getByRole('button', { name: 'Save route', exact: true }).click()
  await expect(dialog).not.toBeVisible()
  await page.reload()
  await expect(
    page.getByRole('button', { name: /Preview display/ }),
  ).toBeVisible()
})

test('module trust review retains its confirmation gate', async ({ page }) => {
  await page.goto('/')
  await navigate(page, 'Modules')
  await page.getByLabel('Module directory').fill('/example/modules/virtual')
  await page.getByRole('button', { name: 'Review module', exact: true }).click()
  const dialog = page.getByRole('dialog')
  await expect(
    dialog.getByRole('button', { name: 'Trust and add', exact: true }),
  ).toBeDisabled()
  await dialog.getByRole('checkbox').check()
  await dialog
    .getByRole('button', { name: 'Trust and add', exact: true })
    .click()
  await expect(dialog).not.toBeVisible()
})

test('input controls send the selected route and destination once', async ({
  page,
}) => {
  await page.goto('/')
  await page
    .getByRole('combobox', { name: 'Switch destination', exact: true })
    .click()
  await page.getByRole('option', { name: 'Studio', exact: true }).click()
  await page
    .getByRole('button', { name: 'Request selected input', exact: true })
    .click()
  const calls = await page.evaluate(() =>
    (
      window as unknown as {
        displayflowFixture: {
          calls: {
            command: string
            args: { routeId?: string; destination?: string }
          }[]
        }
      }
    ).displayflowFixture.calls.filter(
      (c) => c.command === 'select_destination',
    ),
  )
  expect(calls).toHaveLength(1)
  expect(calls[0]!.args).toEqual({
    routeId: 'example-desk',
    destination: 'Studio',
  })
  await expect(
    page.getByRole('combobox', {
      name: 'Confirm visible computer',
      exact: true,
    }),
  ).toContainText('Studio')
})

test('production builds reject the UI fixture transport', async () => {
  const { spawnSync } = await import('node:child_process')
  const { resolve } = await import('node:path')
  const result = spawnSync(
    process.execPath,
    [resolve('apps/ui/node_modules/vite/bin/vite.js'), 'build'],
    {
      cwd: resolve('apps/ui'),
      env: { ...process.env, DISPLAYFLOW_UI_FIXTURE: '1' },
      encoding: 'utf8',
    },
  )
  expect(result.status).not.toBe(0)
  expect(result.stderr).toContain(
    'UI fixtures cannot be included in a production build',
  )
})
