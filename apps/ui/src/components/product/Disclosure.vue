<script setup lang="ts">
import {
  CollapsibleRoot,
  CollapsibleTrigger,
  CollapsibleContent,
} from 'reka-ui'
import UButton from '@nuxt/ui/components/Button.vue'
import { ChevronRight } from 'lucide-vue-next'
defineProps<{ title: string; description?: string }>()
const open = defineModel<boolean>({ default: false })
</script>
<template>
  <CollapsibleRoot v-model:open="open" class="flex min-w-0 flex-col"
    ><div class="flex items-center gap-2">
      <CollapsibleTrigger as-child
        ><UButton
          variant="ghost"
          color="neutral"
          class="min-w-0 flex-1 justify-start text-start"
          ><ChevronRight
            :size="16"
            :class="[
              'shrink-0 transition-transform duration-150 motion-reduce:transition-none',
              { 'rotate-90': open },
            ]"
          /><span class="flex min-w-0 flex-col gap-1"
            ><span class="truncate" :title="title">{{ title }}</span
            ><span
              v-if="description"
              class="truncate text-xs font-normal text-muted"
              >{{ description }}</span
            ></span
          ></UButton
        ></CollapsibleTrigger
      ><slot name="actions" />
    </div>
    <CollapsibleContent class="disclosure-content overflow-hidden"
      ><div class="flex min-w-0 flex-col gap-4 py-4">
        <slot /></div></CollapsibleContent
  ></CollapsibleRoot>
</template>
