<script setup lang="ts">
import { computed, useAttrs } from 'vue'
import UButton from '@nuxt/ui/components/Button.vue'
import UTooltip from '@nuxt/ui/components/Tooltip.vue'
import { cn } from '@/lib/utils'
import { tooltipContent } from '@/lib/theme'
defineOptions({ inheritAttrs: false })
const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
    class?: string
    disabled?: boolean
    type?: 'button' | 'submit' | 'reset'
  }>(),
  { variant: 'secondary', type: 'button', class: '' },
)
const attrs = useAttrs()
function tooltip() {
  return typeof attrs['aria-label'] === 'string'
    ? attrs['aria-label']
    : undefined
}
const color = computed(() => (props.variant === 'danger' ? 'error' : 'neutral'))
const variant = computed(() =>
  props.variant === 'primary'
    ? 'solid'
    : props.variant === 'ghost'
      ? 'ghost'
      : props.variant === 'danger'
        ? 'soft'
        : 'outline',
)
</script>
<template>
  <UTooltip :text="tooltip()" :disabled="!tooltip()" :content="tooltipContent"
    ><UButton
      v-bind="attrs"
      :type="type"
      :disabled="disabled"
      :color="color"
      :variant="variant"
      :class="cn('shrink-0', $props.class)"
      ><slot /></UButton
  ></UTooltip>
</template>
