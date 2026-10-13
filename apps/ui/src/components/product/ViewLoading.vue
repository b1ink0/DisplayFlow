<script setup lang="ts">
import USkeleton from '@nuxt/ui/components/Skeleton.vue'
</script>
<template>
  <div role="status" aria-label="Loading view" class="flex flex-col gap-4">
    <USkeleton class="h-10" /><USkeleton class="h-36" />
  </div>
</template>
