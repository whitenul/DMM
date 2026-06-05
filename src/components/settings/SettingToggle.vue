<template>
  <label class="toggle">
    <input
      type="checkbox"
      :checked="modelValue"
      class="toggle__input"
      @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span class="toggle__slider" />
  </label>
</template>

<script setup lang="ts">
defineProps<{ modelValue: boolean }>();
defineEmits<{ 'update:modelValue': [value: boolean] }>();
</script>

<style scoped>
.toggle {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}
.toggle__input {
  opacity: 0;
  width: 0;
  height: 0;
}
.toggle__slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: var(--color-bg-active, rgba(255,255,255,0.1));
  border-radius: 11px;
  transition: background 0.15s ease;
}
.toggle__slider::before {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 3px;
  bottom: 3px;
  background: var(--color-bg-card, #fff);
  border-radius: 50%;
  transition: transform 0.15s ease;
}
.toggle__input:checked + .toggle__slider {
  background: var(--color-accent, #0078d4);
}
.toggle__input:checked + .toggle__slider::before {
  transform: translateX(18px);
}
</style>
