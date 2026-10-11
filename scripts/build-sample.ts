import { spawnSync } from 'node:child_process'
import { mkdir, copyFile, readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, resolve, join } from 'node:path'
const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  'samples',
  'virtual-provider',
)
const build = spawnSync(
  'cargo',
  ['build', '--release', '--manifest-path', join(root, 'Cargo.toml')],
  { stdio: 'inherit' },
)
if (build.status !== 0) process.exit(build.status ?? 1)
const extension = process.platform === 'win32' ? '.exe' : ''
const output = join(root, 'dist')
await mkdir(output, { recursive: true })
await copyFile(
  join(root, 'target', 'release', `displayflow-sample-provider${extension}`),
  join(output, `provider${extension}`),
)
const manifest = JSON.parse(
  await readFile(join(root, 'manifest.json'), 'utf8'),
) as { executable: string }
manifest.executable = `provider${extension}`
await writeFile(
  join(output, 'displayflow-module.json'),
  `${JSON.stringify(manifest, null, 2)}\n`,
)
console.log(`Mount this sample directory in DisplayFlow: ${output}`)
