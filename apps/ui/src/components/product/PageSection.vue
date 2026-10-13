<script setup lang="ts">
defineProps<{ title?: string; description?: string }>()
</script>
<template>
  <section class="flex min-w-0 flex-col gap-4">
    <header
      v-if="title"
      class="flex flex-wrap items-start justify-between gap-3"
    >
      <div class="flex min-w-0 flex-col gap-1">
        <h2 class="section-title break-words">{{ title }}</h2>
        <p v-if="description" class="hint">{{ description }}</p>
      </div>
      <slot name="actions" />
    </header>
    <slot />
  </section>
</template>
