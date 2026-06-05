<template>
  <div class="btn-group">
    <button
      v-for="opt in options"
      :key="String(opt.value)"
      class="btn-group__item"
      :class="{ 'btn-group__item--active': modelValue === opt.value }"
      @click="$emit('update:modelValue', opt.value)"
    >
      <AppIcon v-if="opt.icon" :name="opt.icon" :size="14" />
      <span>{{ opt.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts" generic="T extends string | number">
import AppIcon from "@/components/common/AppIcon.vue";

defineProps<{
  modelValue: T;
  options: { value: T; label: string; icon?: string }[];
}>();

defineEmits<{
  'update:modelValue': [value: T];
}>();
</script>

<style scoped>
.btn-group {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.btn-group__item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border: 1px solid var(--color-border-strong, rgba(255,255,255,0.08));
  border-radius: var(--radius-sm, 4px);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}
.btn-group__item:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
}
.btn-group__item--active {
  border-color: var(--color-accent, #0078d4);
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
  color: var(--color-accent, #0078d4);
}
</style>
