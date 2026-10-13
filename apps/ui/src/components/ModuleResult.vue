<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ModuleResponse } from '@displayflow/module-sdk'
import Badge from '@/components/ui/Badge.vue'
import Disclosure from '@/components/product/Disclosure.vue'
const props = defineProps<{ result: ModuleResponse }>()
const expanded = ref(false)
const details = computed(() =>
  expanded.value ? JSON.stringify(props.result, null, 2) : '',
)
const labels = {
  unknown: 'Recovery status unknown',
  required: 'Recovery required',
  applied: 'Recovery verified',
  notApplicable: 'Recovery not applicable',
}
</script>
<template>
  <div class="flex min-w-0 flex-col gap-4 border-s-2 border-strong ps-4">
    <div
      v-if="result.recoveryState || result.lastRecoveryAt"
      class="flex flex-wrap gap-2"
    >
      <Badge
        v-if="result.recoveryState"
        :tone="
          result.recoveryState === 'applied'
            ? 'success'
            : result.recoveryState === 'unknown'
              ? 'warning'
              : 'neutral'
        "
        >{{ labels[result.recoveryState] }}</Badge
      ><span v-if="result.lastRecoveryAt" class="text-xs text-muted"
        >Last recovery completed
        {{ new Date(result.lastRecoveryAt * 1000).toLocaleString() }}</span
      >
    </div>
    <p
      v-if="result.message"
      :class="[
        'text-base leading-relaxed break-words',
        result.ok ? 'text-toned' : 'text-error',
      ]"
    >
      {{ result.message }}
    </p>
    <dl v-if="result.fields" class="flex flex-col divide-y divide-default">
      <div
        v-for="(value, key) in result.fields"
        :key="key"
        class="flex flex-col gap-1 py-2 sm:flex-row sm:gap-4"
      >
        <dt class="text-xs break-words text-muted sm:w-44 sm:shrink-0">
          {{ key }}
        </dt>
        <dd class="min-w-0 text-sm break-words">
          {{ value === null ? 'Unknown' : value }}
        </dd>
      </div>
    </dl>
    <Disclosure v-model="expanded" title="Response details">
      <pre
        class="max-h-72 overflow-auto font-mono text-xs break-all whitespace-pre-wrap text-toned"
        >{{ details }}</pre>
    </Disclosure>
  </div>
</template>
