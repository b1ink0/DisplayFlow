<script setup lang="ts">
defineProps<{ title: string; count?: number }>()
</script>
<template>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex min-w-0 items-center gap-2">
      <h2 class="section-title">{{ title }}</h2>
      <span
        v-if="count !== undefined"
        class="text-xs text-muted tabular-nums"
        >{{ count }}</span
      >
    </div>
    <div class="flex flex-wrap items-center gap-2"><slot /></div>
  </div>
</template>
