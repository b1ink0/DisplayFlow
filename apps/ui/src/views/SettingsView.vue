<script setup lang="ts">
import { watch, ref, onMounted } from 'vue'
import { Save, SlidersHorizontal, Plus, Trash2 } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import Button from '@/components/ui/Button.vue'
import Toggle from '@/components/ui/Toggle.vue'
const { settings, snapshot, busy, run, mutate, refresh } = useApp()
const name = ref('')
const log = ref('')
const network = ref(false)
const port = ref(24862)
const target = ref('')
const startupEnabled = ref(false)
const startupReady = ref(false)
const startupCanEnable = ref(false)
const startupError = ref('')
const networkSetupResult = ref('')
async function configureFirewall(enabled: boolean) {
  networkSetupResult.value = ''
  networkSetupResult.value = await call<string>('configure_firewall', {
    enabled,
  })
}
interface StartupStatus {
  enabled: boolean
  canEnable: boolean
}
function applyStartupStatus(status: StartupStatus) {
  startupEnabled.value = status.enabled
  startupCanEnable.value = status.canEnable
  startupReady.value = true
}
onMounted(async () => {
  if (!desktop) return
  try {
    applyStartupStatus(await call<StartupStatus>('startup_status'))
  } catch (error) {
    startupError.value = String(error)
  }
})
async function setStartup(enabled: boolean) {
  applyStartupStatus(await call<StartupStatus>('set_startup', { enabled }))
}
let initialized = false
watch(
  settings,
  () => {
    if (settings.value && !initialized) {
      initialized = true
      name.value = settings.value.computerName
      log.value = settings.value.deskflowLog
      network.value = settings.value.networkEnabled
      port.value = settings.value.listenPort
    }
  },
  { immediate: true },
)
async function save() {
  await mutate((s) => {
    s.computerName = name.value
    s.deskflowLog = log.value
    s.networkEnabled = network.value
    s.listenPort = port.value
  })
}
async function register() {
  const value = target.value.trim()
  if (!value) return
  await mutate((s) => {
    const id = `betterdisplay:${value}`
    if (!s.manualMonitors.some((m) => m.id === id))
      s.manualMonitors.push({
        id,
        name: value,
        provider: 'BetterDisplay',
        detail: 'Exact BetterDisplay name • standard MCCS input selection',
      })
  })
  target.value = ''
  await refresh(true)
}
</script>
<template>
  <div class="panel max-w-3xl">
    <div class="flex items-center gap-3 border-b border-stone-100 p-6">
      <SlidersHorizontal :size="19" class="text-violet-600" />
      <h2 class="section-title">Agent & Deskflow</h2>
    </div>
    <div class="space-y-6 p-6">
      <div>
        <label for="computer-name" class="label"
          >This computer’s Deskflow name</label
        ><input id="computer-name" v-model="name" class="field" />
        <p class="hint mt-2">
          Use the exact name in Deskflow’s server configuration. Hostname and
          Deskflow name can differ.
        </p>
      </div>
      <div>
        <label for="deskflow-log" class="label">Deskflow server log path</label
        ><input
          id="deskflow-log"
          v-model="log"
          class="field font-mono"
          placeholder="Absolute path to the current server log"
        />
        <p class="hint mt-2">
          Set this on the Deskflow server computer only. Enable logging in
          Deskflow at a level that records
          <code>switch from &quot;name&quot; to &quot;name&quot;</code>
          transitions. DisplayFlow watches new lines and leaves unassigned
          destinations alone.
        </p>
      </div>
      <div
        class="flex items-center justify-between rounded-lg border border-stone-200 p-4"
      >
        <div>
          <h3 class="text-sm font-medium">
            Accept paired computer connections
          </h3>
          <p class="hint mt-1">
            Off by default. Needed on every computer participating in peer
            routing.
          </p>
        </div>
        <Toggle v-model="network" label="Enable peer agent" />
      </div>
      <div v-if="network">
        <label for="listen-port" class="label">TCP listen port</label
        ><input
          id="listen-port"
          v-model.number="port"
          type="number"
          min="1024"
          max="65535"
          class="field max-w-40"
        />
        <p class="hint mt-2">
          Default 24862. Allow only your private local network in the OS
          firewall. No router port forwarding.
        </p>
      </div>
      <div class="flex justify-end border-t border-stone-100 pt-4">
        <Button
          variant="primary"
          :disabled="busy || !settings"
          @click="run(save, 'Settings saved')"
          ><Save :size="14" />Save settings</Button
        >
      </div>
    </div>
  </div>
  <section class="panel mt-6 max-w-3xl p-6">
    <h2 class="section-title">Local network access</h2>
    <p class="hint mt-2">
      DisplayFlow runs as your normal user. Pairing and remote input requests
      need local network access. No router forwarding or SSH service is needed.
    </p>
    <template v-if="snapshot?.platform === 'windows'">
      <p class="hint mt-3">
        Save the listen port first. This one-time firewall setup asks Windows
        for administrator approval and allows the installed app on the local
        subnet of Private and Domain networks. Changing the port or installation
        path requires updating the rule. Public networks remain blocked by this
        rule.
      </p>
      <div class="mt-4 flex flex-wrap gap-3">
        <Button
          :disabled="busy || !desktop"
          @click="run(() => configureFirewall(true))"
          >Configure firewall access</Button
        >
        <Button
          :disabled="busy || !desktop"
          variant="ghost"
          @click="run(() => configureFirewall(false))"
          >Remove firewall rule</Button
        >
      </div>
      <p v-if="networkSetupResult" class="hint mt-3" role="status">
        {{ networkSetupResult }}
      </p>
    </template>
    <template v-else-if="snapshot?.platform === 'macos'">
      <p class="hint mt-3">
        Allow DisplayFlow's Local Network prompt when pairing. If access was
        declined, enable it in System Settings → Privacy &amp; Security → Local
        Network. File access is limited to your chosen log and external module
        files; pairing credentials use the OS keychain. Full Disk Access is not
        required.
      </p>
      <Button
        class="mt-4"
        :disabled="busy || !desktop"
        @click="run(() => call('open_network_permissions'))"
        >Open network privacy settings</Button
      >
    </template>
    <p v-else class="hint mt-3">
      Configure networking in the installed desktop application.
    </p>
  </section>
  <section class="panel mt-6 max-w-3xl p-6">
    <h2 class="section-title">Startup & background operation</h2>
    <div class="mt-4 flex items-center justify-between gap-6">
      <div>
        <h3 class="text-sm font-medium">Start at login</h3>
        <p class="hint mt-1">
          Launch in the menu bar or notification area without opening the
          window. Changes apply immediately.
        </p>
      </div>
      <Toggle
        :model-value="startupEnabled"
        :disabled="
          busy || !startupReady || (!startupCanEnable && !startupEnabled)
        "
        label="Start DisplayFlow at login"
        @update:model-value="
          (enabled) =>
            run(
              () => setStartup(enabled),
              enabled ? 'Start at login enabled' : 'Start at login disabled',
            )
        "
      />
    </div>
    <p v-if="!desktop || (startupReady && !startupCanEnable)" class="hint mt-3">
      Install a release build to enable login startup.
    </p>
    <p v-if="startupError" class="hint mt-3 text-red-600">{{ startupError }}</p>
    <p class="hint mt-3">
      Install the app in its permanent location before enabling this option.
      Startup preserves your pause setting; confirm the currently visible
      computer after restarting before routing can switch an input.
    </p>
  </section>
  <section class="panel mt-6 max-w-3xl p-6">
    <h2 class="section-title">macOS · BetterDisplay provider</h2>
    <p class="hint mt-2">
      BetterDisplay must be installed, running, and accept CLI commands.
      Register its exact display name here. Standard input selection uses MCCS
      0x60; vendor-specific behavior belongs in an external provider.
    </p>
    <label for="betterdisplay-name" class="label mt-4"
      >Exact display name</label
    >
    <div class="flex gap-3">
      <input
        id="betterdisplay-name"
        v-model="target"
        class="field"
        placeholder="Display name shown in BetterDisplay"
      /><Button
        :disabled="busy || !target"
        @click="run(register, 'Display registered')"
        ><Plus :size="14" />Register</Button
      >
    </div>
    <div
      v-for="m in settings?.manualMonitors"
      :key="m.id"
      class="mt-3 flex items-center justify-between rounded-lg bg-stone-50 px-3 py-2"
    >
      <span class="text-sm">{{ m.name }}</span
      ><Button
        variant="ghost"
        aria-label="Remove registered monitor"
        @click="
          run(async () => {
            await mutate((s) => {
              s.manualMonitors = s.manualMonitors.filter((x) => x.id !== m.id)
            })
            await refresh(true)
          })
        "
        ><Trash2 :size="13"
      /></Button>
    </div>
  </section>
  <p class="hint mt-5 max-w-3xl">
    Closing the window keeps DisplayFlow running. Use its tray or menu bar menu
    to pause switching or quit.
  </p>
</template>
