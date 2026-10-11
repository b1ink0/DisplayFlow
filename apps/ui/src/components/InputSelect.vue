<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { InputPreset } from '@displayflow/module-sdk'
import Select from '@/components/ui/Select.vue'

const props = defineProps<{
  modelValue: number
  presets: InputPreset[]
  id: string
}>()
const emit = defineEmits<{
  'update:modelValue': [value: number]
  'select-label': [label: string]
}>()
const custom = ref(false)
const text = ref(formatValue(props.modelValue))
const selected = computed(() =>
  !custom.value && props.presets.some((p) => p.value === props.modelValue)
    ? String(props.modelValue)
    : 'custom',
)
const options = computed(() => [
  ...props.presets.map((p) => ({
    value: String(p.value),
    label: `${p.label} · ${formatValue(p.value)}`,
  })),
  { value: 'custom', label: 'Custom value…' },
])
function formatValue(value: number) {
  return Number.isInteger(value)
    ? `0x${value.toString(16).toUpperCase().padStart(2, '0')}`
    : ''
}
function parseValue(value: string) {
  const trimmed = value.trim()
  if (!/^(?:0x[0-9a-f]+|[0-9]+)$/i.test(trimmed)) return Number.NaN
  const number = Number(trimmed)
  return Number.isInteger(number) && number >= 0 && number <= 65535
    ? number
    : Number.NaN
}
watch(
  () => props.modelValue,
  (value) => {
    if (Number.isInteger(value) && parseValue(text.value) !== value)
      text.value = formatValue(value)
  },
)
function choose(value: string) {
  custom.value = value === 'custom'
  if (custom.value) return
  const preset = props.presets.find((p) => String(p.value) === value)
  if (preset) {
    emit('update:modelValue', preset.value)
    emit('select-label', preset.label)
  }
}
function updateText(event: Event) {
  text.value = (event.target as HTMLInputElement).value
  emit('update:modelValue', parseValue(text.value))
}
</script>
<template>
  <div>
    <Select
      :model-value="selected"
      :options="options"
      label="Input preset"
      @update:model-value="choose"
    />
    <div v-if="selected === 'custom'" class="mt-2">
      <label :for="id" class="sr-only">Custom input value</label>
      <input
        :id="id"
        :value="text"
        class="field font-mono"
        placeholder="0x11 or 17"
        spellcheck="false"
        :aria-invalid="!Number.isInteger(modelValue)"
        :aria-describedby="`${id}-hint`"
        @input="updateText"
      />
    </div>
    <p
      :id="`${id}-hint`"
      :class="[
        'mt-1.5 text-xs',
        Number.isInteger(modelValue) ? 'text-stone-400' : 'text-red-600',
      ]"
    >
      {{
        Number.isInteger(modelValue)
          ? `${formatValue(modelValue)} · ${modelValue} decimal`
          : 'Enter 0–65535 or 0x0000–0xFFFF.'
      }}
    </p>
  </div>
</template>
