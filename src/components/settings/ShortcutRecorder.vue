<template>
  <div
    class="shortcut-recorder"
    :class="{ 'shortcut-recorder--recording': recording }"
    tabindex="0"
    @keydown="onKeyDown"
    @click="startRecording"
  >
    <span v-if="recording" class="shortcut-recorder__hint">按下快捷键...</span>
    <span v-else-if="modelValue" class="shortcut-recorder__value">{{ formatShortcut(modelValue) }}</span>
    <span v-else class="shortcut-recorder__placeholder">点击录制</span>
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount } from "vue";

defineProps<{ modelValue: string }>();
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const recording = ref(false);

function formatShortcut(shortcut: string): string {
  return shortcut
    .replace(/Control/g, 'Ctrl')
    .replace(/Meta/g, 'Win')
    .replace(/ArrowUp/g, '↑').replace(/ArrowDown/g, '↓')
    .replace(/ArrowLeft/g, '←').replace(/ArrowRight/g, '→');
}

function startRecording() {
  recording.value = true;
}

function onKeyDown(e: KeyboardEvent) {
  if (!recording.value) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.key === 'Escape') {
    recording.value = false;
    return;
  }

  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');
  if (e.metaKey) parts.push('Win');

  let mainKey = e.key;
  if (mainKey === ' ') mainKey = 'Space';
  else if (mainKey.length === 1) mainKey = mainKey.toUpperCase();
  parts.push(mainKey);

  const shortcut = parts.join('+');
  emit('update:modelValue', shortcut);
  recording.value = false;
}

onBeforeUnmount(() => {
  recording.value = false;
});
</script>

<style scoped>
.shortcut-recorder {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 160px;
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--color-border-strong, rgba(255,255,255,0.08));
  border-radius: var(--radius-sm, 4px);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}
.shortcut-recorder:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
}
.shortcut-recorder--recording {
  border-color: var(--color-accent, #0078d4);
  box-shadow: 0 0 0 1px var(--color-accent, #0078d4);
}
.shortcut-recorder__placeholder,
.shortcut-recorder__hint {
  color: var(--color-text-tertiary);
}
.shortcut-recorder__hint {
  color: var(--color-accent, #0078d4);
}
</style>
