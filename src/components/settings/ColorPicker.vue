<template>
  <div class="color-picker">
    <label v-if="label" class="color-picker__label">{{ label }}</label>
    <div class="color-picker__control">
      <div class="color-picker__swatch" :style="{ backgroundColor: modelValue }" @click="openPicker" />
      <input
        ref="nativeInputRef"
        type="color"
        class="color-picker__native"
        :value="hexOnly"
        @input="onNativeInput"
      />
      <input
        type="text"
        class="color-picker__hex"
        :value="modelValue"
        @change="onHexChange"
        placeholder="#RRGGBB"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";

const props = withDefaults(defineProps<{
  modelValue: string;
  label?: string;
}>(), {
  label: undefined,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const nativeInputRef = ref<HTMLInputElement | null>(null);

/** 提取 hex 部分 */
const hexOnly = computed(() => {
  const v = props.modelValue;
  if (v.startsWith("#") && v.length >= 7) return v.substring(0, 7);
  return "#000000";
});

function openPicker() {
  nativeInputRef.value?.click();
}

function onNativeInput(e: Event) {
  const target = e.target as HTMLInputElement;
  emit("update:modelValue", target.value);
}

function onHexChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = target.value.trim();
  if (/^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(val) || /^rgba?\(/.test(val)) {
    emit("update:modelValue", val);
  }
}
</script>

<style scoped>
.color-picker {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.color-picker__label {
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-tertiary, rgba(0, 0, 0, 0.4));
  flex-shrink: 0;
  min-width: 48px;
}

.color-picker__control {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.color-picker__swatch {
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm, 4px);
  border: 1px solid var(--color-border, rgba(0, 0, 0, 0.06));
  cursor: pointer;
  flex-shrink: 0;
  transition: border-color 0.15s ease;
}

.color-picker__swatch:hover {
  border-color: var(--color-border-strong, rgba(0, 0, 0, 0.12));
}

.color-picker__native {
  position: absolute;
  width: 0;
  height: 0;
  opacity: 0;
  pointer-events: none;
}

.color-picker__hex {
  width: 80px;
  padding: 2px 6px;
  font-size: var(--font-size-xs, 11px);
  font-family: var(--font-family-mono, monospace);
  border: 1px solid var(--color-border, rgba(0, 0, 0, 0.06));
  border-radius: var(--radius-sm, 4px);
  background: var(--color-bg-card, #ffffff);
  color: var(--color-text-primary, rgba(0, 0, 0, 0.9));
  outline: none;
  transition: border-color 0.15s ease;
}

.color-picker__hex:focus {
  border-color: var(--color-accent, #0078d4);
}
</style>
