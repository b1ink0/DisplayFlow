<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  FolderInput,
  Puzzle,
  ShieldCheck,
  Trash2,
  Stethoscope,
  Wrench,
} from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import type { MountedModule } from '@/types'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Modal from '@/components/ui/Modal.vue'
import ModuleResult from '@/components/ModuleResult.vue'
import type { ModuleResponse } from '@displayflow/module-sdk'
const { settings, snapshot, busy, run, refresh } = useApp()
const path = ref('')
const reviewed = ref<MountedModule | null>(null)
const reviewOpen = ref(false)
const trusted = ref(false)
const recoveryOpen = ref(false)
const recoveryMonitor = ref('')
const visible = ref(false)
const recoveryUi = computed(
  () =>
    settings.value?.modules.find((m) =>
      recoveryMonitor.value.startsWith(`module:${m.manifest.id}:`),
    )?.manifest.ui?.recovery,
)
const results = ref<Record<string, ModuleResponse>>({})
function beginRecovery(id: string) {
  recoveryMonitor.value = id
  visible.value = false
  recoveryOpen.value = true
}
async function inspect() {
  reviewed.value = await call<MountedModule>('inspect_module', {
    path: path.value,
  })
  trusted.value = false
  reviewOpen.value = true
}
async function mount() {
  await call('mount_module', {
    reviewed: reviewed.value,
    trusted: trusted.value,
  })
  reviewOpen.value = false
  await refresh(true)
}
async function action(monitorId: string, operation: string) {
  const result = await call<ModuleResponse>('module_action', {
    monitorId,
    operation,
    confirmed: visible.value,
  })
  results.value[monitorId] = result
  recoveryOpen.value = false
}
</script>
<template>
  <section class="panel mb-6 p-6">
    <div class="flex gap-4">
      <div
        class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-violet-50 text-violet-600"
      >
        <Puzzle :size="24" />
      </div>
      <div>
        <h2 class="section-title">Monitor modules</h2>
        <p class="mt-2 max-w-2xl text-sm leading-relaxed text-stone-500">
          Add input providers, diagnostics, and tools for your monitors.
        </p>
      </div>
    </div>
  </section>
  <h2 class="section-title mb-4">Added modules</h2>
  <div
    v-if="!settings?.modules.length"
    class="panel mb-6 px-6 py-9 text-center"
  >
    <Puzzle :size="28" class="mx-auto mb-3 text-stone-300" />
    <p class="text-sm font-medium">No modules added</p>
    <p class="hint mt-2">
      Add a module below to extend the built-in input providers.
    </p>
  </div>
  <div class="mb-6 space-y-4">
    <section
      v-for="module in settings?.modules"
      :key="module.manifest.id"
      class="panel p-5"
    >
      <div class="flex items-start justify-between">
        <div>
          <h3 class="font-semibold">
            {{ module.manifest.name }}
            <span class="ml-1 text-xs font-normal text-stone-400"
              >v{{ module.manifest.version }}</span
            >
          </h3>
          <p class="hint mt-2">{{ module.manifest.description }}</p>
          <p class="mt-2 font-mono text-[10px] break-all text-stone-400">
            {{ module.path }}
          </p>
        </div>
        <Button
          variant="ghost"
          aria-label="Remove module"
          :disabled="busy"
          @click="
            run(async () => {
              await call('unmount_module', { id: module.manifest.id })
              await refresh(true)
            }, 'Module removed')
          "
          ><Trash2 :size="14"
        /></Button>
      </div>
      <div class="mt-4 flex gap-2">
        <Badge
          v-for="capability in module.manifest.capabilities"
          :key="capability"
          >{{ capability }}</Badge
        >
      </div>
      <div
        v-for="monitor in snapshot?.monitors.filter(
          (m) => m.provider === module.manifest.id,
        )"
        :key="monitor.id"
        class="mt-4 border-t border-stone-100 pt-4"
      >
        <div class="flex items-center justify-between">
          <p class="text-sm font-medium">{{ monitor.name }}</p>
          <div class="flex gap-2">
            <Button
              v-if="module.manifest.capabilities.includes('inspect')"
              :disabled="busy"
              @click="run(() => action(monitor.id, 'inspect'))"
              ><Stethoscope :size="13" />Check status</Button
            ><Button
              v-if="module.manifest.capabilities.includes('recover')"
              :disabled="busy"
              @click="beginRecovery(monitor.id)"
              ><Wrench :size="13" />{{
                module.manifest.ui?.recovery?.label ?? 'Apply recovery'
              }}</Button
            >
          </div>
        </div>
        <ModuleResult
          v-if="results[monitor.id]"
          :result="results[monitor.id]!"
        />
      </div>
    </section>
  </div>
  <section class="panel p-6">
    <h2 class="section-title mb-2">Add a module</h2>
    <p class="hint mb-4">
      Choose a directory containing a
      <code>displayflow-module.json</code> manifest and its executable.
    </p>
    <label for="module-path" class="label">Module directory</label>
    <div class="flex gap-3">
      <input
        id="module-path"
        v-model="path"
        class="field"
        placeholder="Full path to the module folder"
      /><Button :disabled="busy || !desktop || !path" @click="run(inspect)"
        ><FolderInput :size="14" />Review</Button
      >
    </div>
  </section>
  <div class="mt-5 flex gap-3 rounded-lg bg-stone-100/70 p-4">
    <ShieldCheck :size="18" class="shrink-0 text-stone-400" />
    <p class="hint">
      Modules run with your account’s permissions. Add only modules you trust.
    </p>
  </div>
  <Modal
    v-model="reviewOpen"
    title="Add this module?"
    description="This executable will run with your account’s permissions."
    ><template v-if="reviewed"
      ><dl class="space-y-3 text-sm">
        <div>
          <dt class="label">Module</dt>
          <dd>
            {{ reviewed.manifest.name }} · {{ reviewed.manifest.version }}
          </dd>
        </div>
        <div>
          <dt class="label">Executable</dt>
          <dd class="font-mono text-xs break-all">
            {{ reviewed.path }} / {{ reviewed.manifest.executable }}
          </dd>
        </div>
        <div>
          <dt class="label">SHA-256</dt>
          <dd class="font-mono text-[11px] break-all">
            {{ reviewed.executableHash }}
          </dd>
        </div>
        <div>
          <dt class="label">Capabilities</dt>
          <dd>{{ reviewed.manifest.capabilities.join(', ') }}</dd>
        </div>
      </dl>
      <label class="mt-6 flex items-start gap-3 text-sm"
        ><input
          v-model="trusted"
          type="checkbox"
          class="mt-1 accent-violet-600"
        />I trust this module and its supporting files.</label
      >
      <div class="mt-6 flex justify-end gap-2">
        <Button @click="reviewOpen = false">Cancel</Button
        ><Button
          variant="primary"
          :disabled="busy || !trusted"
          @click="run(mount, 'Module added')"
          >Trust and add</Button
        >
      </div></template
    ></Modal
  >
  <Modal
    v-model="recoveryOpen"
    :title="recoveryUi?.label ?? 'Run local recovery'"
    :description="
      recoveryUi?.description ??
      'Run this module’s recovery action on the selected monitor.'
    "
    ><label class="flex items-start gap-3 text-sm leading-relaxed"
      ><input
        v-model="visible"
        type="checkbox"
        class="mt-1 accent-violet-600"
      />{{
        recoveryUi?.confirmation ??
        'This computer’s picture is visible on the selected monitor. Run the recovery action.'
      }}</label
    >
    <div class="mt-6 flex justify-end gap-2">
      <Button @click="recoveryOpen = false">Cancel</Button
      ><Button
        variant="primary"
        :disabled="busy || !visible"
        @click="run(() => action(recoveryMonitor, 'recover'))"
        >{{ recoveryUi?.label ?? 'Run recovery' }}</Button
      >
    </div></Modal
  >
</template>
