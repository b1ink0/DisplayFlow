import { spawnSync } from 'node:child_process'
import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const env = { ...process.env }
if (process.platform === 'darwin' && !env.APPLE_SIGNING_IDENTITY) {
  let saved: { identity: string } | undefined
  try {
    saved = JSON.parse(
      await readFile(join(root, 'src-tauri/signing.local.json'), 'utf8'),
    )
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error
  }
  if (saved) {
    if (!/^[0-9A-F]{40}$/i.test(saved.identity))
      throw new Error('Invalid local signing identity')
    env.APPLE_SIGNING_IDENTITY = saved.identity
  }
}
const result = spawnSync(
  process.execPath,
  [
    join(root, 'node_modules/@tauri-apps/cli/tauri.js'),
    'build',
    ...process.argv.slice(2),
  ],
  { cwd: root, env, stdio: 'inherit' },
)
if (result.error) throw result.error
process.exit(result.status ?? 1)
