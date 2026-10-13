<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { CheckCircle2, CircleAlert, Info } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import Select from '@/components/ui/Select.vue'
import EmptyState from '@/components/product/EmptyState.vue'
import ListToolbar from '@/components/product/ListToolbar.vue'
import UPagination from '@nuxt/ui/components/Pagination.vue'
const { snapshot } = useApp()
const level = ref('all')
const page = ref(1)
const events = computed(() =>
  (snapshot.value?.activity ?? []).filter(
    (e) => level.value === 'all' || e.level === level.value,
  ),
)
const visible = computed(() =>
  events.value.slice((page.value - 1) * 25, page.value * 25),
)
watch(level, () => {
  page.value = 1
})
watch(
  () => events.value.length,
  (count) => {
    page.value = Math.max(1, Math.min(page.value, Math.ceil(count / 25)))
  },
)
function time(at: number) {
  return new Date(at * 1000).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}
</script>
<template>
  <div class="flex min-w-0 flex-col gap-4">
    <ListToolbar title="Agent events" :count="events.length"
      ><Select
        v-model="level"
        label="Filter event level"
        :options="[
          { value: 'all', label: 'All events' },
          { value: 'error', label: 'Errors' },
          { value: 'warning', label: 'Warnings' },
          { value: 'success', label: 'Success' },
          { value: 'info', label: 'Information' },
        ]"
        class="w-40"
    /></ListToolbar>
    <EmptyState
      v-if="!events.length"
      title="No events to show"
      description="Agent activity appears here as it runs. Try another filter if events are hidden."
      icon="i-lucide-activity"
    />
    <ol
      v-else
      class="flex min-w-0 flex-col divide-y divide-default border-y border-default"
    >
      <li
        v-for="(event, index) in visible"
        :key="`${event.at}-${index}`"
        class="flex min-w-0 items-start gap-3 py-4"
      >
        <CircleAlert
          v-if="event.level === 'error' || event.level === 'warning'"
          :size="16"
          :class="[
            'shrink-0',
            event.level === 'error' ? 'text-error' : 'text-warning',
          ]"
        /><CheckCircle2
          v-else-if="event.level === 'success'"
          :size="16"
          class="shrink-0 text-success"
        /><Info v-else :size="16" class="shrink-0 text-muted" />
        <div
          class="flex min-w-0 flex-1 flex-col gap-2 sm:flex-row sm:justify-between"
        >
          <p class="min-w-0 text-base leading-relaxed break-words">
            {{ event.message }}
          </p>
          <time
            :datetime="new Date(event.at * 1000).toISOString()"
            class="shrink-0 text-xs text-muted tabular-nums"
            :title="new Date(event.at * 1000).toLocaleString()"
            >{{ time(event.at) }}</time
          >
        </div>
      </li>
    </ol>
    <div class="flex flex-wrap items-center justify-between gap-3">
      <p class="hint">
        Up to 150 events from this session. “Request sent” confirms command
        acceptance, not the visible picture.
      </p>
      <UPagination
        v-if="events.length > 25"
        v-model:page="page"
        :total="events.length"
        :items-per-page="25"
      />
    </div>
  </div>
</template>
