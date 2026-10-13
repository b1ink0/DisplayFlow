<script setup lang="ts">
import USelect from '@nuxt/ui/components/Select.vue'
import USelectMenu from '@nuxt/ui/components/SelectMenu.vue'
defineProps<{
  options: { value: string; label: string }[]
  placeholder?: string
  label?: string
}>()
const model = defineModel<string>({ default: '' })
</script>
<template>
  <USelectMenu
    v-if="options.length > 100"
    v-model="model"
    :items="options"
    value-key="value"
    :placeholder="placeholder ?? 'Choose…'"
    :aria-label="label ?? placeholder"
    virtualize
    class="w-full"
  /><USelect
    v-else
    v-model="model"
    :items="options"
    value-key="value"
    :placeholder="placeholder ?? 'Choose…'"
    :aria-label="label ?? placeholder"
    class="w-full"
    :ui="{ content: 'max-h-72' }"
  />
</template>
