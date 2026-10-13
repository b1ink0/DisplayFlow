<script setup lang="ts">
import UCheckbox from '@nuxt/ui/components/Checkbox.vue'
import { computed, ref, watch, toRefs } from 'vue'
import type { ModuleResponse, PanelAction } from '@displayflow/module-sdk'
import { useApp } from '@/composables/context'
import { call } from '@/services/bridge'
import Select from '@/components/ui/Select.vue'
import Button from '@/components/ui/Button.vue'
import Modal from '@/components/ui/Modal.vue'
import ModuleResult from '@/components/ModuleResult.vue'
import PageSection from '@/components/product/PageSection.vue'
const props = defineProps<{ moduleId: string; panelId: string }>()
const { settings, snapshot, busy, run, panelState } = useApp()
const module = computed(() =>
  settings.value?.modules.find((m) => m.manifest.id === props.moduleId),
)
const panel = computed(() =>
  module.value?.manifest.ui?.panels.find((p) => p.id === props.panelId),
)
const { monitorId, result } = toRefs(
  panelState(`${props.moduleId}:${props.panelId}`),
)
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
  <div class="flex max-w-4xl min-w-0 flex-col gap-6">
    <PageSection v-if="panel" :title="module?.manifest.name">
      <div class="flex max-w-md flex-col gap-2">
        <label class="label" for="panel-monitor">Monitor</label
        ><Select
          id="panel-monitor"
          v-model="monitorId"
          :options="monitors"
          label="Monitor"
        />
        <p v-if="!monitors.length" class="hint">
          Refresh monitors to find this module’s displays.
        </p>
      </div>
      <div
        class="flex flex-col divide-y divide-default border-y border-default"
      >
        <div
          v-for="action in panel.actions"
          :key="action.id"
          class="flex items-start justify-between gap-4 py-4"
        >
          <div class="flex min-w-0 flex-1 flex-col gap-1">
            <h3 class="text-sm font-medium break-words">{{ action.label }}</h3>
            <p class="hint break-words">{{ action.description }}</p>
          </div>
          <Button
            :disabled="busy || !monitorId"
            :aria-label="'Run ' + action.label"
            @click="selectAction(action)"
            >Run</Button
          >
        </div>
      </div>
      <ModuleResult v-if="result" :result="result" />
    </PageSection>
    <Modal
      v-model="confirmation"
      :title="pending?.label ?? 'Confirm action'"
      :description="pending?.description ?? ''"
      ><UCheckbox
        v-model="accepted"
        label="Run this action on the selected monitor."
      />
      <div class="flex justify-end gap-2">
        <Button @click="confirmation = false">Cancel</Button
        ><Button
          variant="primary"
          :disabled="busy || !accepted || !pending"
          @click="pending && run(() => execute(pending!, true))"
          >Run action</Button
        >
      </div></Modal
    >
  </div>
</template>
