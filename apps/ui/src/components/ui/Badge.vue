<script setup lang="ts">
import { cn } from '@/lib/utils'
withDefaults(
  defineProps<{ tone?: 'neutral' | 'success' | 'warning' | 'violet' }>(),
  { tone: 'neutral' },
)
</script>
<template>
  <span
    :class="
      cn(
        'inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium',
        {
          'bg-stone-100 text-stone-600': tone === 'neutral',
          'bg-emerald-50 text-emerald-700': tone === 'success',
          'bg-amber-50 text-amber-700': tone === 'warning',
          'bg-violet-50 text-violet-700': tone === 'violet',
        },
      )
    "
    ><slot
  /></span>
</template>
