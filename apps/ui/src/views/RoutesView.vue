<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  ArrowRight,
  Cable,
  Check,
  CircleHelp,
  Monitor,
  Plus,
  Pencil,
  Trash2,
  Laptop,
  Waypoints,
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
const { snapshot, settings, busy, run, mutate } = useApp()
const open = ref(false)
const draft = ref<Route>({
  id: '',
  name: '',
  attachments: [],
  activeComputer: null,
  enabled: true,
})
const destinations = ref<Record<string, string>>({})
const inventory = ref<Record<string, MonitorType[]>>({})
const transferOpen = ref(false)
const transferText = ref('')
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
  draft.value = route
    ? (JSON.parse(JSON.stringify(route)) as Route)
    : {
        id: crypto.randomUUID(),
        name: '',
        attachments: [attachment(), attachment()],
        activeComputer: null,
        enabled: true,
      }
  open.value = true
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
    const saved = { ...draft.value, activeComputer: null }
    if (index >= 0) s.routes[index] = saved
    else s.routes.push(saved)
  })
  open.value = false
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
  <div class="mb-6 grid grid-cols-3 gap-4">
    <div class="panel p-5">
      <p class="text-xs text-stone-500">Physical monitor routes</p>
      <p class="mt-2 text-2xl font-semibold">
        {{ settings?.routes.length ?? 0
        }}<span class="ml-2 text-xs font-normal text-stone-400"
          >configured</span
        >
      </p>
    </div>
    <div class="panel p-5">
      <p class="text-xs text-stone-500">Local displays</p>
      <p class="mt-2 text-2xl font-semibold">
        {{ snapshot?.monitors.length ?? 0
        }}<span class="ml-2 text-xs font-normal text-stone-400"
          >discovered / registered</span
        >
      </p>
    </div>
    <div class="panel p-5">
      <p class="text-xs text-stone-500">Deskflow connection</p>
      <p class="mt-3 truncate text-sm font-medium" :title="snapshot?.logStatus">
        {{ snapshot?.logStatus ?? 'Starting…' }}
      </p>
    </div>
  </div>
  <div class="mb-4 flex items-center justify-between">
    <h2 class="section-title">Input assignments</h2>
    <div class="flex gap-2">
      <Button :disabled="!settings || busy" @click="beginTransfer"
        >Transfer assignments</Button
      ><Button variant="primary" :disabled="!settings || busy" @click="edit()"
        ><Plus :size="14" />New route</Button
      >
    </div>
  </div>
  <div
    v-if="!settings?.routes.length"
    class="panel flex flex-col items-center px-8 py-14 text-center"
  >
    <div
      class="relative mb-6 flex h-20 w-32 items-center justify-center rounded-2xl bg-violet-50 text-violet-500"
    >
      <Monitor :size="48" :stroke-width="1.2" /><span
        class="absolute -right-2 bottom-0 flex size-8 items-center justify-center rounded-full border-4 border-white bg-violet-600 text-white"
        ><Cable :size="14"
      /></span>
    </div>
    <h3 class="text-lg font-semibold tracking-tight">
      Add your shared monitor
    </h3>
    <p class="mt-3 max-w-md text-sm leading-relaxed text-stone-500">
      Create a route for each shared physical monitor. Assign every connected
      computer to its input, then let Deskflow transitions do the switching.
    </p>
    <Button class="mt-6" variant="primary" :disabled="!settings" @click="edit()"
      ><Plus :size="14" />Create your first route</Button
    >
    <p class="mt-4 text-xs text-stone-400">
      Works with standard DDC monitors and external providers.
    </p>
  </div>
  <div class="space-y-4">
    <article
      v-for="route in settings?.routes"
      :key="route.id"
      class="panel overflow-hidden"
    >
      <div
        class="flex items-start justify-between border-b border-stone-100 p-5"
      >
        <div class="flex items-center gap-3">
          <div class="rounded-lg bg-stone-100 p-2.5">
            <Monitor :size="20" />
          </div>
          <div>
            <h3 class="font-semibold">{{ route.name }}</h3>
            <p class="mt-1 text-xs text-stone-500">
              {{ route.attachments.length }} computers ·
              {{
                route.activeComputer
                  ? `Last confirmed / requested: ${route.activeComputer}`
                  : 'Active input unknown'
              }}
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Badge :tone="route.activeComputer ? 'violet' : 'warning'">{{
            route.activeComputer ? 'Assigned' : 'Confirmation needed'
          }}</Badge
          ><Button
            variant="ghost"
            :disabled="busy"
            aria-label="Edit route"
            @click="edit(route)"
            ><Pencil :size="14"
          /></Button>
        </div>
      </div>
      <div
        class="grid gap-3 p-5"
        :style="{
          gridTemplateColumns: `repeat(${Math.min(route.attachments.length, 3)}, minmax(0, 1fr))`,
        }"
      >
        <div
          v-for="a in route.attachments"
          :key="a.computer"
          :class="[
            'rounded-lg border p-3.5',
            route.activeComputer === a.computer
              ? 'border-violet-200 bg-violet-50/40'
              : 'border-stone-200',
          ]"
        >
          <div class="flex items-center gap-2 text-sm font-medium">
            <Laptop :size="15" />{{ a.computer }}
          </div>
          <div class="mt-3 flex items-center justify-between">
            <span class="text-xs text-stone-500"
              >{{ a.label || 'Input' }} · 0x{{
                a.input.toString(16).toUpperCase()
              }}</span
            ><Check
              v-if="route.activeComputer === a.computer"
              :size="13"
              class="text-violet-600"
            />
          </div>
        </div>
      </div>
      <div
        class="flex flex-wrap items-center gap-3 border-t border-stone-100 bg-stone-50/60 px-5 py-3"
      >
        <div class="w-56">
          <Select
            :model-value="route.activeComputer ?? ''"
            :options="
              route.attachments.map((a) => ({
                value: a.computer,
                label: a.computer,
              }))
            "
            placeholder="Which picture is visible?"
            label="Confirm visible computer"
            @update:model-value="
              (value) =>
                run(() =>
                  mutate((s) => {
                    const r = s.routes.find((x) => x.id === route.id)
                    if (r) r.activeComputer = value
                  }),
                )
            "
          />
        </div>
        <CircleHelp :size="14" class="text-stone-400" /><span
          class="ml-auto text-xs text-stone-400"
          >Request input</span
        >
        <div class="w-44">
          <Select
            v-model="destinations[route.id]"
            :options="
              route.attachments.map((a) => ({
                value: a.computer,
                label: a.computer,
              }))
            "
            placeholder="Destination"
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
    </article>
  </div>
  <section v-if="snapshot?.monitors.length" class="mt-7">
    <h2 class="section-title mb-3">Available on this computer</h2>
    <div class="panel divide-y divide-stone-100">
      <div
        v-for="monitor in snapshot.monitors"
        :key="monitor.id"
        class="flex items-center gap-3 px-5 py-4"
      >
        <Monitor :size="18" class="text-stone-400" />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium">{{ monitor.name }}</p>
          <p class="hint mt-1 truncate" :title="monitor.id">
            {{ monitor.detail }}
          </p>
        </div>
        <Badge>{{ monitor.provider }}</Badge>
      </div>
    </div>
  </section>
  <div class="mt-6 flex gap-3 rounded-lg bg-stone-100/70 p-4">
    <Waypoints :size="17" class="mt-0.5 shrink-0 text-stone-400" />
    <p class="hint">
      The computer driving the current input sends each command. A successful
      request is recorded as an assumption; manually confirm the visible picture
      after a power cycle, cable change, or joystick switch.
    </p>
  </div>
  <Modal
    v-model="transferOpen"
    title="Transfer assignments"
    description="Copy route mappings between paired computers. This contains computer names and monitor IDs, but no pairing credentials. Imported routes need a fresh visible-input confirmation."
    ><label for="route-transfer" class="label">Route JSON</label
    ><textarea
      id="route-transfer"
      v-model="transferText"
      class="field h-64 font-mono text-xs"
      placeholder="Paste exported route array"
    />
    <div class="mt-4 flex justify-between">
      <Button :disabled="busy" @click="run(exportRoutes, 'Assignments copied')"
        >Copy current assignments</Button
      ><Button
        variant="primary"
        :disabled="busy || !transferText"
        @click="
          run(importRoutes, 'Assignments imported; verify them before resuming')
        "
        >Import assignments</Button
      >
    </div></Modal
  >
  <Modal
    v-model="open"
    title="Physical monitor route"
    description="These attachments belong to one physical monitor. Input numbers are monitor-specific; confirm them using the monitor menu."
    ><div class="space-y-5">
      <div>
        <label class="label" for="route-name">Monitor name</label
        ><input
          id="route-name"
          v-model="draft.name"
          class="field"
          placeholder="Shared desk display"
        />
      </div>
      <div class="flex items-center justify-between">
        <span class="label mb-0">Allow automatic switching</span
        ><Toggle v-model="draft.enabled" label="Enable route" />
      </div>
      <div
        v-for="(a, index) in draft.attachments"
        :key="index"
        class="rounded-xl border border-stone-200 bg-stone-50/50 p-4"
      >
        <div class="mb-3 flex items-center justify-between">
          <p class="text-xs font-semibold text-stone-500">
            ATTACHMENT {{ index + 1 }}
          </p>
          <Button
            variant="ghost"
            :disabled="draft.attachments.length <= 2"
            aria-label="Remove attachment"
            @click="draft.attachments.splice(index, 1)"
            ><Trash2 :size="13"
          /></Button>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label :for="`computer-${index}`" class="label"
              >Exact Deskflow name</label
            ><input
              :id="`computer-${index}`"
              v-model="a.computer"
              class="field"
              placeholder="Computer name"
            />
          </div>
          <div>
            <label class="label">Agent computer</label
            ><Select
              v-model="a.deviceId"
              :options="devices"
              label="Agent computer"
            />
          </div>
          <div class="col-span-2">
            <div class="mb-1 flex items-center justify-between">
              <label class="label mb-0">Monitor on that agent</label
              ><button
                v-if="a.deviceId !== settings?.deviceId"
                class="text-xs text-violet-600"
                @click="run(() => loadPeer(a.deviceId))"
              >
                Fetch inventory
              </button>
            </div>
            <Select
              v-if="options(a.deviceId).length"
              v-model="a.monitorId"
              :options="options(a.deviceId)"
              label="Monitor identity"
            /><input
              v-else
              v-model="a.monitorId"
              class="field font-mono"
              placeholder="Exact provider monitor ID"
            />
          </div>
          <div>
            <label class="label">Input</label>
            <InputSelect
              :id="`input-${index}`"
              v-model="a.input"
              :presets="inputPresets(a)"
              @select-label="a.label = $event"
            />
          </div>
          <div>
            <label :for="`label-${index}`" class="label">Input label</label
            ><input
              :id="`label-${index}`"
              v-model="a.label"
              class="field"
              placeholder="HDMI 1 / DisplayPort"
            />
          </div>
        </div>
        <p v-if="a.monitorId" class="hint mt-2 truncate">
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
        agents must also have a local route allowing this monitor and its input
        values.
      </p>
      <div class="flex justify-between border-t border-stone-100 pt-4">
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
        <div class="ml-auto flex gap-2">
          <Button @click="open = false">Cancel</Button
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
</template>
