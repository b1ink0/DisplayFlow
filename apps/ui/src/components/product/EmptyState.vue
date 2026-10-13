<script setup lang="ts">
import UEmpty from '@nuxt/ui/components/Empty.vue'
defineProps<{ title: string; description: string; icon?: string }>()
</script>
<template>
  <UEmpty
    :title="title"
    :description="description"
    :icon="icon"
    :ui="{
      root: 'border-0 py-8',
      title: 'text-sm font-medium',
      description: 'text-sm max-w-md',
    }"
    ><template #actions><slot /></template
  ></UEmpty>
</template>
