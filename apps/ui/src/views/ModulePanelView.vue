<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { ModuleResponse, PanelAction } from '@displayflow/module-sdk'
import { useApp } from '@/composables/context'
import { call } from '@/services/bridge'
import Select from '@/components/ui/Select.vue'
import Button from '@/components/ui/Button.vue'
import Modal from '@/components/ui/Modal.vue'
import ModuleResult from '@/components/ModuleResult.vue'
const props = defineProps<{ moduleId: string; panelId: string }>()
const { settings, snapshot, busy, run } = useApp()
const module = computed(() =>
  settings.value?.modules.find((m) => m.manifest.id === props.moduleId),
)
const panel = computed(() =>
  module.value?.manifest.ui?.panels.find((p) => p.id === props.panelId),
)
const monitorId = ref('')
const result = ref<ModuleResponse | null>(null)
const confirmation = ref(false)
const accepted = ref(false)
const pending = ref<PanelAction | null>(null)
const monitors = computed(() =>
  (snapshot.value?.monitors ?? [])
    .filter((m) => m.provider === props.moduleId)
    .map((m) => ({
      value: m.id.slice(`module:${props.moduleId}:`.length),
      label: m.name,
    })),
)
async function execute(action: PanelAction, confirmed = false) {
  result.value = await call<ModuleResponse>('panel_action', {
    moduleId: props.moduleId,
    panelId: props.panelId,
    actionId: action.id,
    monitorId: monitorId.value,
    confirmed,
  })
  confirmation.value = false
}
watch(
  monitors,
  (options) => {
    if (!options.some((o) => o.value === monitorId.value))
      monitorId.value = options.length === 1 ? options[0]!.value : ''
  },
  { immediate: true },
)
function selectAction(action: PanelAction) {
  if (action.requiresConfirmation) {
    pending.value = action
    accepted.value = false
    confirmation.value = true
  } else void run(() => execute(action))
}
</script>
<template>
  <section v-if="panel" class="panel max-w-3xl p-6">
    <p class="mb-5 text-xs text-stone-400">
      {{ module?.manifest.name }}
    </p>
    <label class="label">Monitor</label
    ><Select v-model="monitorId" :options="monitors" label="Monitor" />
    <p v-if="!monitors.length" class="hint mt-2">
      Refresh monitors to find this module’s displays.
    </p>
    <div class="mt-6 space-y-3">
      <div
        v-for="action in panel.actions"
        :key="action.id"
        class="flex items-center justify-between gap-5 rounded-lg border border-stone-200 p-4"
      >
        <div>
          <h3 class="text-sm font-medium">{{ action.label }}</h3>
          <p class="hint mt-1">{{ action.description }}</p>
        </div>
        <Button :disabled="busy || !monitorId" @click="selectAction(action)"
          >Run</Button
        >
      </div>
    </div>
    <ModuleResult v-if="result" :result="result" />
  </section>
  <Modal
    v-model="confirmation"
    :title="pending?.label ?? 'Confirm action'"
    :description="pending?.description ?? ''"
    ><label class="flex gap-3 text-sm"
      ><input v-model="accepted" type="checkbox" class="accent-violet-600" />Run
      this action on the selected monitor.</label
    >
    <div class="mt-6 flex justify-end gap-2">
      <Button @click="confirmation = false">Cancel</Button
      ><Button
        variant="primary"
        :disabled="busy || !accepted || !pending"
        @click="pending && run(() => execute(pending!, true))"
        >Run action</Button
      >
    </div></Modal
  >
</template>
