import { spawnSync } from 'node:child_process'
import { mkdtemp, readFile, writeFile, rm, chmod } from 'node:fs/promises'
import { tmpdir, homedir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { randomBytes } from 'node:crypto'

if (process.platform !== 'darwin')
  throw new Error('Local macOS signing setup runs on a Mac.')
const root = dirname(dirname(fileURLToPath(import.meta.url)))
const label = 'DisplayFlow Local Development'
function run(command: string, args: string[]) {
  const result = spawnSync(command, args, { encoding: 'utf8' })
  if (result.error || result.status !== 0)
    throw new Error(
      `${command} failed: ${result.stderr.trim() || result.error?.message || `exit ${result.status}`}`,
    )
  return result.stdout
}
function find() {
  const output = run('/usr/bin/security', [
    'find-identity',
    '-p',
    'codesigning',
  ])
  const identities = [
    ...new Set(
      [
        ...output.matchAll(
          /\b([A-F0-9]{40})\s+"DisplayFlow Local Development"/gi,
        ),
      ].map((m) => m[1]),
    ),
  ]
  if (identities.length > 1)
    throw new Error(
      'Several local signing identities exist. Choose one with APPLE_SIGNING_IDENTITY.',
    )
  return identities[0]
}
let identity = find()
if (!identity) {
  const temporary = await mkdtemp(join(tmpdir(), 'displayflow-signing-'))
  await chmod(temporary, 0o700)
  try {
    const config = join(temporary, 'certificate.cnf')
    await writeFile(
      config,
      `[req]\nprompt=no\ndistinguished_name=name\nx509_extensions=signing\n[name]\nCN=${label}\n[signing]\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=critical,codeSigning\n`,
      { mode: 0o600 },
    )
    run('/usr/bin/openssl', [
      'req',
      '-new',
      '-newkey',
      'rsa:3072',
      '-x509',
      '-sha256',
      '-nodes',
      '-days',
      '3650',
      '-config',
      config,
      '-keyout',
      join(temporary, 'key.pem'),
      '-out',
      join(temporary, 'certificate.pem'),
    ])
    await chmod(join(temporary, 'key.pem'), 0o600)
    const password = randomBytes(32).toString('hex')
    await writeFile(join(temporary, 'password'), password, { mode: 0o600 })
    run('/usr/bin/openssl', [
      'pkcs12',
      '-export',
      '-inkey',
      join(temporary, 'key.pem'),
      '-in',
      join(temporary, 'certificate.pem'),
      '-out',
      join(temporary, 'identity.p12'),
      '-name',
      label,
      '-passout',
      `file:${join(temporary, 'password')}`,
    ])
    run('/usr/bin/security', [
      'import',
      join(temporary, 'identity.p12'),
      '-k',
      join(homedir(), 'Library/Keychains/login.keychain-db'),
      '-P',
      password,
      '-T',
      '/usr/bin/codesign',
    ])
    identity = find()
    if (!identity)
      throw new Error('The imported signing identity could not be found.')
  } finally {
    // Only this process's newly created temporary directory is removed.
    await rm(temporary, { recursive: true, force: true })
  }
}
const probe = await mkdtemp(join(tmpdir(), 'displayflow-signing-check-'))
await chmod(probe, 0o700)
try {
  const executable = join(probe, 'signing-check')
  await writeFile(executable, await readFile('/usr/bin/true'), { mode: 0o700 })
  run('/usr/bin/codesign', [
    '--force',
    '--sign',
    identity,
    '--identifier',
    'dev.displayflow.signing-check',
    executable,
  ])
  run('/usr/bin/codesign', ['--verify', '--strict', executable])
  await writeFile(
    join(root, 'src-tauri/signing.local.json'),
    `${JSON.stringify({ identity }, null, 2)}\n`,
    { mode: 0o600 },
  )
} finally {
  await rm(probe, { recursive: true, force: true })
}
console.log(
  'Local signing is configured. Future bun run build commands reuse this identity. The certificate and private key remain in your login Keychain.',
)
