<script setup lang="ts">
import USwitch from '@nuxt/ui/components/Switch.vue'
defineProps<{ label: string; disabled?: boolean }>()
const model = defineModel<boolean>({ default: false })
</script>
<template>
  <USwitch
    v-model="model"
    :aria-label="label"
    :disabled="disabled"
    size="xl"
    :ui="{ base: 'relative' }"
    class="switch-target shrink-0"
  />
</template>
