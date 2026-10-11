<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'reka-ui'
defineProps<{ label: string; disabled?: boolean }>()
const model = defineModel<boolean>({ default: false })
</script>
<template>
  <SwitchRoot
    v-model="model"
    :aria-label="label"
    :disabled="disabled"
    class="relative h-6 w-11 shrink-0 rounded-full bg-stone-200 transition outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-violet-500 data-[state=checked]:bg-violet-600"
    ><SwitchThumb
      class="block size-5 translate-x-0.5 rounded-full bg-white shadow-sm transition data-[state=checked]:translate-x-5.5"
  /></SwitchRoot>
</template>
