<script setup lang="ts">
import { cn } from '@/lib/utils'
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
    class?: string
    disabled?: boolean
    type?: 'button' | 'submit' | 'reset'
  }>(),
  { variant: 'secondary', type: 'button', class: '' },
)
</script>
<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="
      cn(
        'inline-flex items-center justify-center gap-2 rounded-lg border px-3.5 py-2 text-sm font-medium transition focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-violet-500 disabled:pointer-events-none disabled:opacity-45',
        {
          'border-violet-600 bg-violet-600 text-white shadow-sm hover:bg-violet-700':
            variant === 'primary',
          'border-stone-200 bg-white text-stone-700 shadow-xs hover:bg-stone-50':
            variant === 'secondary',
          'border-transparent text-stone-500 hover:bg-stone-100 hover:text-stone-900':
            variant === 'ghost',
          'border-red-200 bg-red-50 text-red-700 hover:bg-red-100':
            variant === 'danger',
        },
        $props.class,
      )
    "
  >
    <slot />
  </button>
</template>
