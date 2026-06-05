<template>
  <div class="theme-selector">
    <div class="theme-selector__grid">
      <button
        v-for="theme in themeOptions"
        :key="`${theme.id}-${theme.mode}`"
        class="theme-selector__item"
        :class="{ 'theme-selector__item--active': currentThemeId === theme.id }"
        :style="{ '--theme-accent-preview': theme.accent }"
        @click="selectTheme(theme.id)"
      >
        <div class="theme-selector__swatch" />
        <span class="theme-selector__name">{{ theme.name }}</span>
        <svg v-if="currentThemeId === theme.id" class="theme-selector__check" viewBox="0 0 16 16">
          <path d="M6.5 12L2 7.5l1.4-1.4L6.5 9.2l6.1-6.1L14 4.5z" fill="currentColor" />
        </svg>
      </button>
      <button class="theme-selector__item theme-selector__item--add" @click="$emit('openEditor')">
        <div class="theme-selector__swatch theme-selector__swatch--add">+</div>
        <span class="theme-selector__name">自定义</span>
      </button>
    </div>
    <div class="theme-selector__actions">
      <button class="theme-selector__action-btn" @click="handleImport">导入主题</button>
      <button
        v-if="currentThemeId && !isBuiltin(currentThemeId)"
        class="theme-selector__action-btn"
        @click="$emit('exportTheme', currentThemeId)"
      >
        导出主题
      </button>
    </div>
    <!-- 隐藏的文件输入，用于导入 -->
    <input
      ref="fileInputRef"
      type="file"
      accept=".json"
      class="theme-selector__file-input"
      @change="onFileSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { getBuiltinThemeIds } from "@/themes/builtin";
import type { ThemeDefinition } from "@/types/theme";

const props = defineProps<{
  currentThemeId: string;
  availableThemes: ThemeDefinition[];
}>();

const emit = defineEmits<{
  selectTheme: [id: string];
  openEditor: [];
  importTheme: [json: unknown];
  exportTheme: [id: string];
}>();

const builtinIds = getBuiltinThemeIds();
const fileInputRef = ref<HTMLInputElement | null>(null);

/**
 * 按 id + mode 组合去重，只保留当前 resolvedTheme 对应 mode 的主题。
 * 这样同一个主题 ID 的 light/dark 变体不会重复显示。
 */
const themeOptions = computed(() => {
  const seen = new Set<string>();
  const result: { id: string; name: string; accent: string; mode: string }[] = [];
  for (const t of props.availableThemes) {
    const key = `${t.id}__${t.mode}`;
    if (!seen.has(key)) {
      seen.add(key);
      result.push({ id: t.id, name: t.name, accent: t.colors.accent, mode: t.mode });
    }
  }
  return result;
});

function isBuiltin(id: string) {
  return builtinIds.includes(id);
}

function selectTheme(id: string) {
  emit("selectTheme", id);
}

function handleImport() {
  fileInputRef.value?.click();
}

function onFileSelected(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = () => {
    try {
      const json = JSON.parse(reader.result as string);
      emit("importTheme", json);
    } catch {
      // 无效 JSON，静默忽略
    }
  };
  reader.readAsText(file);

  // 重置 input 以便同一文件可以再次选择
  input.value = "";
}
</script>

<style scoped>
.theme-selector {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
  max-width: 100%;
}

.theme-selector__grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.theme-selector__item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px;
  border: 2px solid transparent;
  border-radius: var(--radius-md, 8px);
  background: var(--color-bg-card, #ffffff);
  cursor: pointer;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.theme-selector__item:hover {
  border-color: var(--color-border-strong, rgba(0, 0, 0, 0.12));
}

.theme-selector__item--active {
  border-color: var(--color-accent, #0078d4);
  box-shadow: 0 0 0 1px var(--color-accent, #0078d4);
}

.theme-selector__swatch {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md, 8px);
  background: var(--theme-accent-preview, #0078d4);
  transition: transform 0.15s ease;
}

.theme-selector__item:hover .theme-selector__swatch {
  transform: scale(1.05);
}

.theme-selector__swatch--add {
  background: var(--color-bg-subtle, #fafafa);
  color: var(--color-text-tertiary, rgba(0, 0, 0, 0.4));
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 600;
  border: 1px dashed var(--color-border-strong, rgba(0, 0, 0, 0.12));
}

.theme-selector__name {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
  max-width: 52px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-selector__check {
  position: absolute;
  bottom: 6px;
  right: 6px;
  width: 12px;
  height: 12px;
  color: var(--color-accent, #0078d4);
}

.theme-selector__actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.theme-selector__action-btn {
  padding: 4px 10px;
  font-size: var(--font-size-xs, 11px);
  border: 1px solid var(--color-border, rgba(0, 0, 0, 0.06));
  border-radius: var(--radius-sm, 4px);
  background: var(--color-bg-card, #ffffff);
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.theme-selector__action-btn:hover {
  background: var(--color-bg-hover, rgba(0, 0, 0, 0.04));
  border-color: var(--color-border-strong, rgba(0, 0, 0, 0.12));
}

.theme-selector__file-input {
  display: none;
}
</style>
