<script setup lang="ts">
import type { ModuleResponse } from '@displayflow/module-sdk'
import Badge from '@/components/ui/Badge.vue'
defineProps<{ result: ModuleResponse }>()
const labels = {
  unknown: 'Recovery status unknown',
  required: 'Recovery required',
  applied: 'Recovery verified',
  notApplicable: 'Recovery not applicable',
}
</script>
<template>
  <div
    class="mt-4 space-y-4 rounded-xl border border-stone-200 bg-stone-50/60 p-4"
  >
    <div class="flex flex-wrap gap-2">
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
      ><Badge v-if="result.lastRecoveryAt"
        >Last recovery completed
        {{ new Date(result.lastRecoveryAt * 1000).toLocaleString() }}</Badge
      >
    </div>
    <p v-if="result.message" class="text-sm leading-relaxed text-stone-600">
      {{ result.message }}
    </p>
    <dl v-if="result.fields" class="grid grid-cols-2 gap-x-6 gap-y-3">
      <div v-for="(value, key) in result.fields" :key="key">
        <dt class="label">{{ key }}</dt>
        <dd class="text-xs leading-relaxed break-words text-stone-700">
          {{ value === null ? 'Unknown' : value }}
        </dd>
      </div>
    </dl>
    <details>
      <summary class="cursor-pointer text-xs text-stone-400">
        Response details
      </summary>
      <pre class="mt-2 overflow-x-auto text-xs text-stone-500">{{
        JSON.stringify(result, null, 2)
      }}</pre>
    </details>
  </div>
</template>
