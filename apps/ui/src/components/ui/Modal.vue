<script setup lang="ts">
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from 'reka-ui'
import { X } from 'lucide-vue-next'
defineProps<{ title: string; description: string }>()
const open = defineModel<boolean>({ default: false })
</script>
<template>
  <DialogRoot v-model:open="open"
    ><DialogPortal
      ><DialogOverlay
        class="fixed inset-0 z-40 bg-stone-950/30 backdrop-blur-xs" /><DialogContent
        class="fixed top-1/2 left-1/2 z-50 max-h-[88vh] w-[min(680px,90vw)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-2xl border border-stone-200 bg-white p-7 shadow-2xl focus:outline-none"
        ><DialogTitle
          class="text-xl font-semibold tracking-tight text-stone-900"
          >{{ title }}</DialogTitle
        ><DialogDescription
          class="mt-2 pr-6 text-sm leading-relaxed text-stone-500"
          >{{ description }}</DialogDescription
        ><DialogClose
          aria-label="Close dialog"
          class="absolute top-6 right-6 rounded-md p-1 text-stone-400 hover:bg-stone-100 focus-visible:outline-2 focus-visible:outline-violet-500"
          ><X :size="18"
        /></DialogClose>
        <div class="mt-6"><slot /></div></DialogContent></DialogPortal
  ></DialogRoot>
</template>
