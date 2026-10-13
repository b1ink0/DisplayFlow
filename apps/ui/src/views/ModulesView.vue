<script setup lang="ts">
import UInput from '@nuxt/ui/components/Input.vue'
import UCheckbox from '@nuxt/ui/components/Checkbox.vue'
import { ref, computed, watch } from 'vue'
import { FolderInput, Trash2, Stethoscope, Wrench } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import type { MountedModule } from '@/types'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Modal from '@/components/ui/Modal.vue'
import ModuleResult from '@/components/ModuleResult.vue'
import PageSection from '@/components/product/PageSection.vue'
import ListToolbar from '@/components/product/ListToolbar.vue'
import EmptyState from '@/components/product/EmptyState.vue'
import Disclosure from '@/components/product/Disclosure.vue'
import UPagination from '@nuxt/ui/components/Pagination.vue'
import type { ModuleResponse } from '@displayflow/module-sdk'
const { settings, snapshot, busy, run, refresh } = useApp()
const page = ref(1)
const modules = computed(() =>
  (settings.value?.modules ?? []).slice((page.value - 1) * 8, page.value * 8),
)
watch(
  () => settings.value?.modules.length ?? 0,
  (count) => {
    page.value = Math.max(1, Math.min(page.value, Math.ceil(count / 8)))
  },
)
const monitorLimits = ref<Record<string, number>>({})
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
  <div class="flex min-w-0 flex-col gap-6">
    <PageSection>
      <ListToolbar
        title="Added modules"
        :count="settings?.modules.length ?? 0"
      />
      <EmptyState
        v-if="!settings?.modules.length"
        title="No modules added"
        description="Built-in providers are ready to use. Add a module below when your monitor needs another provider or tool."
        icon="i-lucide-puzzle"
      />
      <div class="flex flex-col divide-y divide-default">
        <section
          v-for="module in modules"
          :key="module.manifest.id"
          class="py-3"
        >
          <Disclosure
            :title="module.manifest.name"
            :description="
              'v' +
              module.manifest.version +
              ' · ' +
              module.manifest.description
            "
          >
            <template #actions
              ><Button
                variant="ghost"
                aria-label="Remove module"
                :disabled="busy"
                @click="
                  run(async () => {
                    await call('unmount_module', { id: module.manifest.id })
                    await refresh(true)
                  }, 'Module removed')
                "
                ><Trash2 :size="14" /></Button
            ></template>
            <p class="font-mono text-xs break-all text-muted">
              {{ module.path }}
            </p>
            <div class="flex flex-wrap gap-2">
              <Badge
                v-for="capability in module.manifest.capabilities"
                :key="capability"
                >{{ capability }}</Badge
              >
            </div>
            <div
              v-for="monitor in snapshot?.monitors
                .filter((m) => m.provider === module.manifest.id)
                .slice(0, monitorLimits[module.manifest.id] ?? 50)"
              :key="monitor.id"
              class="flex flex-col gap-4 border-t border-default pt-4"
            >
              <div class="flex flex-wrap items-center justify-between gap-3">
                <p class="min-w-0 flex-1 text-sm font-medium break-words">
                  {{ monitor.name }}
                </p>
                <div class="flex flex-wrap gap-2">
                  <Button
                    v-if="module.manifest.capabilities.includes('inspect')"
                    :disabled="busy"
                    @click="run(() => action(monitor.id, 'inspect'))"
                    ><Stethoscope :size="14" />Check status</Button
                  ><Button
                    v-if="module.manifest.capabilities.includes('recover')"
                    :disabled="busy"
                    @click="beginRecovery(monitor.id)"
                    ><Wrench :size="14" />{{
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
            <Button
              v-if="
                (snapshot?.monitors.filter(
                  (m) => m.provider === module.manifest.id,
                ).length ?? 0) > (monitorLimits[module.manifest.id] ?? 50)
              "
              variant="ghost"
              class="self-start"
              @click="
                monitorLimits[module.manifest.id] =
                  (monitorLimits[module.manifest.id] ?? 50) + 50
              "
              >Show more displays</Button
            >
          </Disclosure>
        </section>
      </div>
      <UPagination
        v-if="(settings?.modules.length ?? 0) > 8"
        v-model:page="page"
        :total="settings?.modules.length ?? 0"
        :items-per-page="8"
        class="self-end"
      />
    </PageSection>
    <PageSection
      title="Add a module"
      description="Choose the directory containing its displayflow-module.json manifest and executable."
    >
      <div class="flex flex-col gap-2">
        <label for="module-path" class="label">Module directory</label>
        <div class="flex flex-wrap gap-3">
          <UInput
            id="module-path"
            v-model="path"
            class="min-w-0 flex-1 basis-64"
            placeholder="Full path to the module folder"
          /><Button :disabled="busy || !desktop || !path" @click="run(inspect)"
            ><FolderInput :size="14" />Review module</Button
          >
        </div>
      </div>
      <p class="hint">
        Modules run with your account’s permissions. Review the executable and
        add only modules you trust.
      </p>
    </PageSection>
    <Modal
      v-model="reviewOpen"
      title="Add this module?"
      description="This executable will run with your account’s permissions."
      ><template v-if="reviewed"
        ><dl class="flex flex-col gap-3 text-sm">
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
            <dd class="font-mono text-xs break-all">
              {{ reviewed.executableHash }}
            </dd>
          </div>
          <div>
            <dt class="label">Capabilities</dt>
            <dd>{{ reviewed.manifest.capabilities.join(', ') }}</dd>
          </div>
        </dl>
        <UCheckbox
          v-model="trusted"
          label="I trust this module and its supporting files."
        />
        <div class="flex justify-end gap-2">
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
      ><UCheckbox
        v-model="visible"
        :label="
          recoveryUi?.confirmation ??
          'This computer’s picture is visible on the selected monitor. Run the recovery action.'
        "
      />
      <div class="mt-4 flex justify-end gap-2">
        <Button @click="recoveryOpen = false">Cancel</Button
        ><Button
          variant="primary"
          :disabled="busy || !visible"
          @click="run(() => action(recoveryMonitor, 'recover'))"
          >{{ recoveryUi?.label ?? 'Run recovery' }}</Button
        >
      </div></Modal
    >
  </div>
</template>
