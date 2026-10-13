<script setup lang="ts">
import UBadge from '@nuxt/ui/components/Badge.vue'
withDefaults(
  defineProps<{ tone?: 'neutral' | 'success' | 'warning' | 'violet' }>(),
  { tone: 'neutral' },
)
</script>
<template>
  <UBadge
    :color="tone === 'violet' ? 'neutral' : tone"
    variant="subtle"
    size="sm"
    class="font-normal"
    ><slot
  /></UBadge>
</template>
