<script setup lang="ts">
defineProps<{ title: string; description?: string }>()
</script>
<template>
  <header class="flex flex-wrap items-start justify-between gap-4">
    <div class="flex min-w-0 flex-1 basis-64 flex-col gap-2">
      <h1 class="text-xl font-semibold break-words text-highlighted">
        {{ title }}
      </h1>
      <p v-if="description" class="text-sm leading-relaxed text-muted">
        {{ description }}
      </p>
    </div>
    <div
      v-if="$slots.actions"
      class="flex shrink-0 flex-wrap items-center gap-2"
    >
      <slot name="actions" />
    </div>
  </header>
</template>
