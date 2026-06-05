<template>
  <div class="setting-slider">
    <input
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      class="setting-slider__input"
      @input="$emit('update:modelValue', parseFloat(($event.target as HTMLInputElement).value))"
    />
    <span class="setting-slider__value">
      {{ unit === '%' ? Math.round(modelValue * 100) + '%' : String(modelValue) }}
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: number;
  min: number;
  max: number;
  step: number;
  unit?: string;
}>();

defineEmits<{ 'update:modelValue': [value: number] }>();
</script>

<style scoped>
.setting-slider {
  display: flex;
  align-items: center;
  gap: 10px;
}
.setting-slider__input {
  width: 140px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--color-bg-active, rgba(255,255,255,0.1));
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}
.setting-slider__input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-accent, #0078d4);
  cursor: pointer;
  transition: transform 0.1s ease;
}
.setting-slider__input::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}
.setting-slider__value {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  min-width: 36px;
  text-align: right;
}
</style>
