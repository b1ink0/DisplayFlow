<script setup lang="ts">
import UInput from '@nuxt/ui/components/Input.vue'
import { watch, ref, onMounted } from 'vue'
import { Save, Plus, Trash2 } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import Button from '@/components/ui/Button.vue'
import Toggle from '@/components/ui/Toggle.vue'
import PageSection from '@/components/product/PageSection.vue'
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
  <div class="flex max-w-4xl min-w-0 flex-col gap-8">
    <PageSection
      title="Agent & Deskflow"
      description="Use the computer name from Deskflow. Configure its log only on the server computer."
    >
      <div class="flex flex-col gap-4 sm:flex-row">
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <label for="computer-name" class="label"
            >This computer’s Deskflow name</label
          ><UInput id="computer-name" v-model="name" />
          <p class="hint">This can differ from its OS hostname.</p>
        </div>
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <label for="deskflow-log" class="label"
            >Deskflow server log path</label
          ><UInput
            id="deskflow-log"
            v-model="log"
            :ui="{ base: 'font-mono' }"
            placeholder="Absolute path to the current server log"
          />
          <p class="hint">Leave empty on client computers.</p>
        </div>
      </div>
      <div
        class="flex items-center justify-between gap-4 border-t border-default py-4"
      >
        <div class="flex flex-col gap-1">
          <h3 class="text-sm font-medium">Peer agent</h3>
          <p class="hint">
            Allow paired computers to request assigned monitor inputs.
          </p>
        </div>
        <Toggle v-model="network" label="Enable peer agent" />
      </div>
      <div v-if="network" class="flex max-w-xs flex-col gap-2">
        <label for="listen-port" class="label">TCP listen port</label
        ><UInput
          id="listen-port"
          v-model.number="port"
          type="number"
          min="1024"
          max="65535"
        />
        <p class="hint">Default 24862. Use your private local network.</p>
      </div>
      <div class="flex justify-end">
        <Button
          variant="primary"
          :disabled="busy || !settings"
          @click="run(save, 'Settings saved')"
          ><Save :size="14" />Save settings</Button
        >
      </div>
    </PageSection>
    <PageSection
      title="Local network access"
      description="DisplayFlow runs as your normal user. No SSH service or router forwarding is needed."
      class="border-t border-default pt-6"
    >
      <template v-if="snapshot?.platform === 'windows'"
        ><p class="hint">
          Save the listen port first. Firewall setup requests administrator
          approval for the installed app on Private and Domain local networks.
          Update the rule after changing the port or installation path.
        </p>
        <div class="flex flex-wrap gap-2">
          <Button
            :disabled="busy || !desktop"
            @click="run(() => configureFirewall(true))"
            >Configure firewall access</Button
          ><Button
            :disabled="busy || !desktop"
            variant="ghost"
            @click="run(() => configureFirewall(false))"
            >Remove firewall rule</Button
          >
        </div>
        <p v-if="networkSetupResult" class="hint" role="status">
          {{ networkSetupResult }}
        </p></template
      >
      <template v-else-if="snapshot?.platform === 'macos'"
        ><p class="hint">
          Allow Local Network access when pairing. DisplayFlow also needs its
          selected log and module files; Full Disk Access is unnecessary.
        </p>
        <Button
          class="self-start"
          :disabled="busy || !desktop"
          @click="run(() => call('open_network_permissions'))"
          >Open network privacy settings</Button
        ></template
      >
      <p v-else class="hint">
        Configure network access in the installed desktop app.
      </p>
    </PageSection>
    <PageSection
      title="Startup & background operation"
      class="border-t border-default pt-6"
    >
      <div class="flex items-start justify-between gap-4">
        <div class="flex flex-col gap-2">
          <h3 class="text-sm font-medium">Start at login</h3>
          <p class="hint">
            Launch in the menu bar or notification area with the window hidden.
            Changes apply immediately.
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
      <p v-if="!desktop || (startupReady && !startupCanEnable)" class="hint">
        Install a release build to enable login startup.
      </p>
      <p v-if="startupError" class="text-xs text-error" role="alert">
        {{ startupError }}
      </p>
      <p class="hint">
        Keep the app in its permanent location. Restarting clears the assumed
        input; confirm the visible computer or use a reliable detection provider
        before switching.
      </p>
    </PageSection>
    <PageSection
      title="BetterDisplay provider"
      description="On macOS, keep BetterDisplay running with CLI access enabled. Register its exact display name for standard MCCS input selection."
      class="border-t border-default pt-6"
    >
      <div class="flex flex-col gap-2">
        <label for="betterdisplay-name" class="label">Exact display name</label>
        <div class="flex flex-wrap gap-3">
          <UInput
            id="betterdisplay-name"
            v-model="target"
            class="min-w-0 flex-1 basis-64"
            placeholder="Display name shown in BetterDisplay"
          /><Button
            :disabled="busy || !target"
            @click="run(register, 'Display registered')"
            ><Plus :size="14" />Register</Button
          >
        </div>
      </div>
      <div class="flex flex-col divide-y divide-default">
        <div
          v-for="m in settings?.manualMonitors"
          :key="m.id"
          class="flex min-w-0 items-center justify-between gap-3 py-3"
        >
          <span class="text-sm break-words">{{ m.name }}</span
          ><Button
            variant="ghost"
            aria-label="Remove registered monitor"
            @click="
              run(async () => {
                await mutate((s) => {
                  s.manualMonitors = s.manualMonitors.filter(
                    (x) => x.id !== m.id,
                  )
                })
                await refresh(true)
              })
            "
            ><Trash2 :size="14"
          /></Button>
        </div>
      </div>
    </PageSection>
    <p class="hint">
      Closing the window keeps the agent running. Use the tray or menu bar to
      pause switching or quit.
    </p>
  </div>
</template>
