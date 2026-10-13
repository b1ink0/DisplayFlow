<script setup lang="ts">
import UModal from '@nuxt/ui/components/Modal.vue'
import UAlert from '@nuxt/ui/components/Alert.vue'
import { useApp } from '@/composables/context'
defineProps<{ title: string; description: string }>()
const open = defineModel<boolean>({ default: false })
const { error } = useApp()
</script>
<template>
  <UModal
    v-model:open="open"
    :title="title"
    :description="description"
    :close="{
      'aria-label': 'Close dialog',
      color: 'neutral',
      variant: 'ghost',
    }"
    ><template #body
      ><UAlert
        v-if="error"
        role="alert"
        color="error"
        variant="subtle"
        :title="error"
        :close="{ 'aria-label': 'Dismiss dialog error' }"
        @update:open="error = ''" /><slot /></template
  ></UModal>
</template>
