<script setup lang="ts">
import UTextarea from '@nuxt/ui/components/Textarea.vue'
import UInput from '@nuxt/ui/components/Input.vue'
import { computed, ref, watch } from 'vue'
import {
  ArrowRight,
  Monitor,
  Plus,
  Pencil,
  Trash2,
  Laptop,
} from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import type { Monitor as MonitorType, Route, Attachment } from '@/types'
import { standardInputs } from '@/lib/inputs'
import InputSelect from '@/components/InputSelect.vue'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Modal from '@/components/ui/Modal.vue'
import Select from '@/components/ui/Select.vue'
import Toggle from '@/components/ui/Toggle.vue'
import ListToolbar from '@/components/product/ListToolbar.vue'
import EmptyState from '@/components/product/EmptyState.vue'
import Disclosure from '@/components/product/Disclosure.vue'
import UPagination from '@nuxt/ui/components/Pagination.vue'
const { snapshot, settings, busy, run, mutate } = useApp()
const open = ref(false)
const reusableDraft = ref(false)
const draft = ref<Route>({
  id: '',
  name: '',
  attachments: [],
  activeComputer: null,
  enabled: true,
  autoDetect: false,
})
const destinations = ref<Record<string, string>>({})
const inventory = ref<Record<string, MonitorType[]>>({})
const transferOpen = ref(false)
const transferText = ref('')
const query = ref('')
const displayLimit = ref(50)
const page = ref(1)
const expanded = ref<Record<string, boolean>>({})
const filtered = computed(() =>
  (settings.value?.routes ?? []).filter(
    (r) =>
      r.name.toLocaleLowerCase().includes(query.value.toLocaleLowerCase()) ||
      r.attachments.some((a) =>
        a.computer
          .toLocaleLowerCase()
          .includes(query.value.toLocaleLowerCase()),
      ),
  ),
)
const visibleRoutes = computed(() =>
  filtered.value.slice((page.value - 1) * 8, page.value * 8),
)
watch(query, () => {
  page.value = 1
})
watch(
  () => filtered.value.length,
  (count) => {
    page.value = Math.max(1, Math.min(page.value, Math.ceil(count / 8)))
  },
)
function beginTransfer() {
  transferText.value = ''
  transferOpen.value = true
}
async function exportRoutes() {
  transferText.value = JSON.stringify(settings.value?.routes ?? [], null, 2)
  await navigator.clipboard.writeText(transferText.value)
}
async function importRoutes() {
  const parsed: unknown = JSON.parse(transferText.value)
  if (
    !Array.isArray(parsed) ||
    parsed.length > 64 ||
    parsed.some(
      (r) =>
        !r ||
        typeof r !== 'object' ||
        !('id' in r) ||
        !('attachments' in r) ||
        !Array.isArray(r.attachments),
    )
  )
    throw new Error('Paste a DisplayFlow route array.')
  const routes = parsed as Route[]
  for (const route of routes) {
    if (
      typeof route.id !== 'string' ||
      typeof route.name !== 'string' ||
      !route.name.trim() ||
      typeof route.enabled !== 'boolean' ||
      (route.autoDetect !== undefined &&
        typeof route.autoDetect !== 'boolean') ||
      route.attachments.length < 2 ||
      route.attachments.length > 16
    )
      throw new Error('Invalid route metadata.')
    for (const a of route.attachments) {
      if (
        !a ||
        typeof a.computer !== 'string' ||
        !a.computer.trim() ||
        typeof a.deviceId !== 'string' ||
        !a.deviceId ||
        typeof a.monitorId !== 'string' ||
        !a.monitorId ||
        typeof a.label !== 'string' ||
        !Number.isInteger(a.input) ||
        a.input < 0 ||
        a.input > 65535
      )
        throw new Error('Invalid attachment or input value.')
    }
    if (
      new Set(route.attachments.map((a) => a.computer)).size !==
      route.attachments.length
    )
      throw new Error('Duplicate computer assignment.')
  }
  await mutate((s) => {
    for (const route of routes) {
      const clean = { ...route, activeComputer: null }
      const index = s.routes.findIndex((r) => r.id === route.id)
      if (index < 0) s.routes.push(clean)
      else s.routes[index] = clean
    }
  })
  transferOpen.value = false
}
const devices = computed(() =>
  settings.value
    ? [
        {
          value: settings.value.deviceId,
          label: `${settings.value.computerName} (this computer)`,
        },
        ...settings.value.peers.map((p) => ({ value: p.id, label: p.name })),
      ]
    : [],
)
function attachment(): Attachment {
  return {
    computer: '',
    deviceId: settings.value?.deviceId ?? '',
    monitorId: '',
    input: 17,
    label: 'HDMI 1',
  }
}
function edit(route?: Route) {
  if (
    reusableDraft.value &&
    (route
      ? route.id === draft.value.id
      : !settings.value?.routes.some((r) => r.id === draft.value.id))
  ) {
    open.value = true
    return
  }
  draft.value = route
    ? (JSON.parse(JSON.stringify(route)) as Route)
    : {
        id: crypto.randomUUID(),
        name: '',
        attachments: [attachment(), attachment()],
        activeComputer: null,
        enabled: true,
        autoDetect: false,
        coordinatorDeviceId: settings.value?.deviceId,
      }
  open.value = true
  reusableDraft.value = true
}
function discardDraft() {
  reusableDraft.value = false
  open.value = false
}
async function save() {
  if (!draft.value.name.trim())
    throw new Error('Give this physical monitor a name.')
  if (
    draft.value.attachments.some(
      (a) => !Number.isInteger(a.input) || a.input < 0 || a.input > 65535,
    )
  )
    throw new Error(
      'Each input must be a decimal or hexadecimal value from 0 to 65535.',
    )
  await mutate((s) => {
    const index = s.routes.findIndex((r) => r.id === draft.value.id)
    const saved = {
      ...(JSON.parse(JSON.stringify(draft.value)) as Route),
      activeComputer: null,
    }
    if (index >= 0) s.routes[index] = saved
    else s.routes.push(saved)
  })
  open.value = false
  reusableDraft.value = false
}
function options(device: string) {
  return (
    device === settings.value?.deviceId
      ? (snapshot.value?.monitors ?? [])
      : (inventory.value[device] ?? [])
  ).map((m) => ({ value: m.id, label: `${m.name} · ${m.provider}` }))
}
function inputPresets(a: Attachment) {
  const monitors =
    a.deviceId === settings.value?.deviceId
      ? (snapshot.value?.monitors ?? [])
      : (inventory.value[a.deviceId] ?? [])
  const presets = monitors.find((m) => m.id === a.monitorId)?.inputPresets
  if (presets?.length) return presets
  return a.monitorId.startsWith('ddc:') ||
    a.monitorId.startsWith('betterdisplay:')
    ? standardInputs
    : []
}
async function loadPeer(id: string) {
  inventory.value[id] = await call<MonitorType[]>('peer_inventory', { id })
}
function monitorName(id: string) {
  return snapshot.value?.monitors.find((m) => m.id === id)?.name ?? id
}
</script>
<template>
  <div class="flex min-w-0 flex-col gap-6">
    <div
      class="flex flex-wrap items-center gap-x-6 gap-y-2 border-y border-default py-3 text-xs text-muted"
    >
      <span
        ><strong class="font-medium text-default">{{
          settings?.routes.length ?? 0
        }}</strong>
        monitor routes</span
      ><span
        ><strong class="font-medium text-default">{{
          snapshot?.monitors.length ?? 0
        }}</strong>
        local displays</span
      ><span class="min-w-0 truncate" :title="snapshot?.logStatus">{{
        snapshot?.logStatus ?? 'Starting…'
      }}</span>
    </div>
    <div class="flex flex-col gap-4">
      <ListToolbar title="Input assignments" :count="filtered.length"
        ><Button :disabled="!settings || busy" @click="beginTransfer"
          >Transfer</Button
        ><Button variant="primary" :disabled="!settings || busy" @click="edit()"
          ><Plus :size="14" />New route</Button
        ></ListToolbar
      >
      <UInput
        v-if="(settings?.routes.length ?? 0) > 4"
        v-model="query"
        icon="i-lucide-search"
        placeholder="Find a display or computer"
        aria-label="Search monitor routes"
        class="max-w-sm"
      />
      <EmptyState
        v-if="!settings?.routes.length"
        title="Add your shared monitor"
        description="Create a route for each physical display, then assign its computers and inputs."
        icon="i-lucide-monitor"
        ><Button variant="primary" :disabled="!settings || busy" @click="edit()"
          ><Plus :size="14" />Create a route</Button
        ></EmptyState
      >
      <EmptyState
        v-else-if="!filtered.length"
        title="No matching routes"
        description="Try another display or computer name."
        icon="i-lucide-search"
      />
      <div class="flex flex-col gap-3">
        <article
          v-for="route in visibleRoutes"
          :key="route.id"
          class="panel px-3 py-2"
        >
          <Disclosure
            :model-value="expanded[route.id] ?? settings?.routes.length === 1"
            :title="route.name"
            :description="
              route.activeComputer
                ? 'Last confirmed / requested: ' + route.activeComputer
                : 'Visible input needs confirmation'
            "
            @update:model-value="expanded[route.id] = $event"
          >
            <template #actions
              ><Badge :tone="route.activeComputer ? 'neutral' : 'warning'">{{
                route.enabled ? 'Enabled' : 'Disabled'
              }}</Badge
              ><Button
                variant="ghost"
                :disabled="busy"
                aria-label="Edit route"
                @click="edit(route)"
                ><Pencil :size="14" /></Button
            ></template>
            <div class="flex flex-col divide-y divide-default">
              <div
                v-for="a in route.attachments"
                :key="a.computer"
                :class="[
                  'flex min-w-0 flex-wrap items-center gap-3 px-3 py-3',
                  route.activeComputer === a.computer
                    ? 'rounded-md bg-selected'
                    : '',
                ]"
              >
                <Laptop :size="16" class="shrink-0 text-muted" /><span
                  class="min-w-0 flex-1 text-sm font-medium break-words"
                  >{{ a.computer }}</span
                ><span class="text-xs text-muted"
                  >{{ a.label || 'Input' }} · 0x{{
                    a.input.toString(16).toUpperCase()
                  }}</span
                ><Badge v-if="route.activeComputer === a.computer"
                  >Current source</Badge
                >
              </div>
            </div>
            <div
              class="flex flex-wrap items-end gap-4 border-t border-default px-3 pt-4"
            >
              <div class="flex min-w-40 flex-1 flex-col gap-2">
                <label class="label" :for="'visible-' + route.id"
                  >Which picture is visible?</label
                ><Select
                  :id="'visible-' + route.id"
                  :model-value="route.activeComputer ?? ''"
                  :options="
                    route.attachments.map((a) => ({
                      value: a.computer,
                      label: a.computer,
                    }))
                  "
                  placeholder="Confirm visible computer"
                  label="Confirm visible computer"
                  @update:model-value="
                    (value) =>
                      run(() =>
                        call('confirm_input', {
                          routeId: route.id,
                          computer: value,
                        }),
                      )
                  "
                />
              </div>
              <div class="flex min-w-40 flex-1 flex-col gap-2">
                <label class="label" :for="'destination-' + route.id"
                  >Destination</label
                ><Select
                  :id="'destination-' + route.id"
                  v-model="destinations[route.id]"
                  :options="
                    route.attachments.map((a) => ({
                      value: a.computer,
                      label: a.computer,
                    }))
                  "
                  placeholder="Choose computer"
                  label="Switch destination"
                />
              </div>
              <Button
                :disabled="
                  busy ||
                  !desktop ||
                  !destinations[route.id] ||
                  !route.activeComputer ||
                  settings?.paused
                "
                aria-label="Request selected input"
                @click="
                  run(() =>
                    call('select_destination', {
                      routeId: route.id,
                      destination: destinations[route.id],
                    }),
                  )
                "
                ><ArrowRight :size="15"
              /></Button>
            </div>
            <div
              class="flex flex-wrap items-center justify-between gap-3 border-t border-default px-3 pt-4"
            >
              <p class="hint min-w-0 flex-1 basis-48">
                {{
                  snapshot?.routeDetection?.[route.id]?.message ??
                  (route.autoDetect
                    ? 'Automatic detection is shared across this route.'
                    : 'Check all assigned computers with one read-only request.')
                }}
              </p>
              <Button
                :disabled="busy || !desktop"
                @click="run(() => call('detect_input', { routeId: route.id }))"
                >Detect now</Button
              >
              <p v-if="snapshot?.routeSync?.[route.id]" class="hint basis-full">
                {{ snapshot.routeSync[route.id]?.message }}
              </p>
            </div>
          </Disclosure>
        </article>
      </div>
      <UPagination
        v-if="filtered.length > 8"
        v-model:page="page"
        :total="filtered.length"
        :items-per-page="8"
        class="self-end"
      />
    </div>
    <Disclosure
      v-if="snapshot?.monitors.length"
      title="Available on this computer"
      :description="
        snapshot.monitors.length + ' discovered or registered displays'
      "
      ><div class="flex flex-col divide-y divide-default">
        <div
          v-for="monitor in snapshot.monitors.slice(0, displayLimit)"
          :key="monitor.id"
          class="flex min-w-0 items-center gap-3 py-3"
        >
          <Monitor :size="16" class="shrink-0 text-muted" />
          <div class="flex min-w-0 flex-1 flex-col gap-1">
            <p class="text-sm font-medium break-words">{{ monitor.name }}</p>
            <p class="hint truncate" :title="monitor.id">{{ monitor.id }}</p>
          </div>
          <Badge>{{ monitor.provider }}</Badge>
        </div>
      </div>
      <Button
        v-if="snapshot.monitors.length > displayLimit"
        variant="ghost"
        class="self-start"
        @click="displayLimit += 50"
        >Show more displays</Button
      ></Disclosure
    >
    <p class="hint">
      Commands come from the computer driving the current input. Accepted
      requests update the assumed source; reconfirm the picture after a manual
      change when detection is unavailable.
    </p>
    <Modal
      v-model="transferOpen"
      title="Transfer assignments"
      description="Copy route mappings between paired computers. This contains computer names and monitor IDs, but no pairing credentials. Imported routes need a fresh visible-input confirmation."
      ><label for="route-transfer" class="label">Route JSON</label
      ><UTextarea
        id="route-transfer"
        v-model="transferText"
        class="field h-64 font-mono text-xs"
        placeholder="Paste exported route array"
      />
      <div class="flex flex-wrap justify-between gap-3">
        <Button
          :disabled="busy"
          @click="run(exportRoutes, 'Assignments copied')"
          >Copy current assignments</Button
        ><Button
          variant="primary"
          :disabled="busy || !transferText"
          @click="
            run(
              importRoutes,
              'Assignments imported; verify them before resuming',
            )
          "
          >Import assignments</Button
        >
      </div></Modal
    >
    <Modal
      v-model="open"
      title="Physical monitor route"
      description="These attachments belong to one physical monitor. Input numbers are monitor-specific; confirm them using the monitor menu."
      ><div class="flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <label class="label" for="route-name">Monitor name</label
          ><UInput
            id="route-name"
            v-model="draft.name"
            class="field"
            placeholder="Shared desk display"
          />
        </div>
        <div class="flex items-center justify-between gap-3">
          <span class="label shrink-0">Coordinator</span>
          <span
            v-if="
              settings?.routes.some((r) => r.id === draft.id) &&
              draft.coordinatorDeviceId
            "
            class="text-[13px] text-toned"
          >
            {{
              devices.find((d) => d.value === draft.coordinatorDeviceId)
                ?.label ?? 'Paired coordinator'
            }}
          </span>
          <Select
            v-else
            :model-value="draft.coordinatorDeviceId ?? 'auto'"
            :options="[
              { value: 'auto', label: 'Discover Deskflow server' },
              ...devices,
            ]"
            label="Route coordinator"
            placeholder="Discover Deskflow server"
            @update:model-value="
              draft.coordinatorDeviceId = $event === 'auto' ? null : $event
            "
          />
        </div>
        <div class="flex items-center justify-between">
          <span class="label">Allow automatic switching</span
          ><Toggle v-model="draft.enabled" label="Enable route" />
        </div>
        <div class="flex items-start justify-between gap-4">
          <div>
            <span class="label">Detect the visible input automatically</span>
            <p class="hint">
              Shared with all assigned computers through the coordinator. Uses
              read-only replies; confirm manually when your monitor cannot
              report a reliable result.
            </p>
          </div>
          <Toggle
            :model-value="draft.autoDetect ?? false"
            label="Automatic input detection"
            @update:model-value="draft.autoDetect = $event"
          />
        </div>
        <div
          v-for="(a, index) in draft.attachments"
          :key="index"
          class="flex flex-col gap-4 border-t border-default pt-4"
        >
          <div class="flex items-center justify-between">
            <p class="text-xs font-semibold text-muted">
              Computer {{ index + 1 }}
            </p>
            <Button
              variant="ghost"
              :disabled="draft.attachments.length <= 2"
              aria-label="Remove attachment"
              @click="draft.attachments.splice(index, 1)"
              ><Trash2 :size="13"
            /></Button>
          </div>
          <div class="flex flex-wrap gap-4">
            <div class="flex min-w-0 flex-1 basis-48 flex-col gap-2">
              <label :for="`computer-${index}`" class="label"
                >Exact Deskflow name</label
              ><UInput
                :id="`computer-${index}`"
                v-model="a.computer"
                class="field"
                placeholder="Computer name"
              />
            </div>
            <div class="min-w-0 flex-1 basis-48">
              <label class="label">Agent computer</label
              ><Select
                v-model="a.deviceId"
                :options="devices"
                label="Agent computer"
              />
            </div>
            <div class="flex w-full flex-col gap-2">
              <div class="flex items-center justify-between">
                <label class="label mb-0">Monitor on that agent</label
                ><Button
                  v-if="a.deviceId !== settings?.deviceId"
                  variant="ghost"
                  class="text-xs text-toned"
                  @click="run(() => loadPeer(a.deviceId))"
                >
                  Fetch inventory
                </Button>
              </div>
              <Select
                v-if="options(a.deviceId).length"
                v-model="a.monitorId"
                :options="options(a.deviceId)"
                label="Monitor identity"
              /><UInput
                v-else
                v-model="a.monitorId"
                class="field font-mono"
                placeholder="Exact provider monitor ID"
              />
            </div>
            <div class="min-w-0 flex-1 basis-48">
              <label class="label">Input</label>
              <InputSelect
                :id="`input-${index}`"
                v-model="a.input"
                :presets="inputPresets(a)"
                @select-label="a.label = $event"
              />
            </div>
            <div class="min-w-0 flex-1 basis-48">
              <label :for="`label-${index}`" class="label">Input label</label
              ><UInput
                :id="`label-${index}`"
                v-model="a.label"
                class="field"
                placeholder="HDMI 1 / DisplayPort"
              />
            </div>
          </div>
          <p v-if="a.monitorId" class="hint truncate">
            {{ monitorName(a.monitorId) }}
          </p>
        </div>
        <Button class="w-full" @click="draft.attachments.push(attachment())"
          ><Plus :size="14" />Add another computer</Button
        >
        <p class="hint">
          Standard presets are suggestions; verify your monitor's input values.
          Modules can supply their own presets. Custom values accept decimal or
          hex. The selected monitor provider controls the command protocol. Peer
          agents must also have a local route allowing this monitor and its
          input values.
        </p>
        <div
          class="flex flex-wrap gap-3 border-t border-default pt-4"
          :class="
            settings?.routes.some((r) => r.id === draft.id)
              ? 'justify-between'
              : 'justify-end'
          "
        >
          <Button
            v-if="settings?.routes.some((r) => r.id === draft.id)"
            variant="danger"
            :disabled="busy"
            @click="
              run(async () => {
                await mutate((s) => {
                  s.routes = s.routes.filter((r) => r.id !== draft.id)
                })
                open = false
              })
            "
            >Delete route</Button
          >
          <div class="flex gap-2">
            <Button @click="discardDraft">Cancel</Button
            ><Button
              variant="primary"
              :disabled="busy"
              @click="run(save, 'Route saved')"
              >Save route</Button
            >
          </div>
        </div>
      </div></Modal
    >
  </div>
</template>
