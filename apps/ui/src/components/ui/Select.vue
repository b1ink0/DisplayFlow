<script setup lang="ts">
import {
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectIcon,
  SelectPortal,
  SelectContent,
  SelectViewport,
  SelectItem,
  SelectItemText,
  SelectItemIndicator,
} from 'reka-ui'
import { Check, ChevronDown } from 'lucide-vue-next'
defineProps<{
  options: { value: string; label: string }[]
  placeholder?: string
  label?: string
}>()
const model = defineModel<string>({ default: '' })
</script>
<template>
  <SelectRoot v-model="model">
    <SelectTrigger
      class="field flex items-center justify-between gap-3"
      :aria-label="label ?? placeholder"
      ><SelectValue :placeholder="placeholder ?? 'Choose…'" /><SelectIcon
        ><ChevronDown :size="14" class="text-stone-400" /></SelectIcon
    ></SelectTrigger>
    <SelectPortal
      ><SelectContent
        position="popper"
        :side-offset="5"
        class="z-50 max-h-72 min-w-(--reka-select-trigger-width) overflow-hidden rounded-lg border border-stone-200 bg-white p-1 shadow-xl"
        ><SelectViewport
          ><SelectItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
            class="relative flex cursor-default items-center rounded-md py-2 pr-9 pl-3 text-sm text-stone-700 outline-none data-highlighted:bg-violet-50 data-highlighted:text-violet-700"
            ><SelectItemText>{{ option.label }}</SelectItemText
            ><SelectItemIndicator class="absolute right-3"
              ><Check
                :size="
                  14
                " /></SelectItemIndicator></SelectItem></SelectViewport></SelectContent
    ></SelectPortal>
  </SelectRoot>
</template>
