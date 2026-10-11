<script setup lang="ts">
import { Activity, CheckCircle2, CircleAlert, Info } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
const { snapshot } = useApp()
function time(at: number) {
  return new Date(at * 1000).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}
</script>
<template>
  <div class="panel overflow-hidden">
    <div
      class="flex items-center justify-between border-b border-stone-100 px-6 py-4"
    >
      <h2 class="section-title">Agent activity</h2>
      <span class="text-xs text-stone-400"
        >Latest 150 events · this session</span
      >
    </div>
    <div v-if="!snapshot?.activity.length" class="px-6 py-14 text-center">
      <Activity :size="30" class="mx-auto mb-3 text-stone-300" />
      <p class="text-sm text-stone-500">
        Events will appear here as the agent runs.
      </p>
    </div>
    <ol class="divide-y divide-stone-100">
      <li
        v-for="(event, index) in snapshot?.activity"
        :key="`${event.at}-${index}`"
        class="flex items-start gap-4 px-6 py-4"
      >
        <CircleAlert
          v-if="event.level === 'error'"
          :size="17"
          class="mt-0.5 shrink-0 text-red-500"
        /><CheckCircle2
          v-else-if="event.level === 'success'"
          :size="17"
          class="mt-0.5 shrink-0 text-emerald-500"
        /><Info v-else :size="17" class="mt-0.5 shrink-0 text-stone-400" />
        <p class="flex-1 text-sm leading-relaxed text-stone-600">
          {{ event.message }}
        </p>
        <time class="mt-0.5 shrink-0 font-mono text-xs text-stone-400">{{
          time(event.at)
        }}</time>
      </li>
    </ol>
  </div>
  <p class="hint mt-4">
    “Request sent” records command acceptance, not a verified picture change.
    Recovery responses are reported exactly as provided by the external module.
  </p>
</template>
