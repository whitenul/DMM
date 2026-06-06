<template>
  <div class="appearance-section">
    <h3 class="section-title">{{ t('appearance.title') }}</h3>

    <!-- 明暗模式 -->
    <SettingRow :title="t('appearance.mode')" :description="t('appearance.mode.desc')">
      <SettingButtonGroup
        :model-value="currentMode"
        :options="modeOptions"
        @update:model-value="(v) => themeStore.setMode(v as ThemeMode)"
      />
    </SettingRow>

    <!-- 主题色 -->
    <SettingRow :title="t('appearance.theme')" :description="t('appearance.theme.desc')">
      <ThemeSelector
        :current-theme-id="themeStore.currentThemeId"
        :available-themes="themeStore.availableThemes"
        @select-theme="(id) => themeStore.setTheme(id)"
        @open-editor="showEditor = true"
        @import-theme="onImportTheme"
        @export-theme="onExportTheme"
      />
    </SettingRow>

    <!-- 强调色来源 -->
    <SettingRow :title="t('appearance.accent')" :description="t('appearance.accent.desc')">
      <SettingButtonGroup
        :model-value="themeStore.currentAccentSource"
        :options="accentSourceOptions"
        @update:model-value="(v) => themeStore.setAccentSource(v as 'system' | 'theme' | 'custom')"
      />
    </SettingRow>

    <!-- 自定义强调色 -->
    <SettingRow
      v-if="themeStore.currentAccentSource === 'custom'"
      :title="t('appearance.accent.customLabel')"
      :description="t('appearance.accent.customDesc')"
    >
      <ColorPicker
        :model-value="themeStore.accentColor"
        @update:model-value="(hex) => themeStore.setAccentColor(hex)"
      />
    </SettingRow>

    <!-- 窗口效果 -->
    <SettingRow :title="t('appearance.effect')" :description="t('appearance.effect.desc')">
      <SettingButtonGroup
        :model-value="themeStore.currentWindowEffect"
        :options="windowEffectOptions"
        @update:model-value="(v) => themeStore.setWindowEffect(v)"
      />
    </SettingRow>

    <!-- 背景图片 -->
    <SettingRow :title="t('appearance.bg')" :description="t('appearance.bg.desc')">
      <div class="bg-image-controls">
        <button class="bg-image-btn" @click="onSelectBackgroundImage">
          {{ themeStore.backgroundImage ? t('appearance.bg.change') : t('appearance.bg.select') }}
        </button>
        <button
          v-if="themeStore.backgroundImage"
          class="bg-image-btn bg-image-btn--danger"
          @click="onClearBackgroundImage"
        >
          {{ t('appearance.bg.clear') }}
        </button>
      </div>
    </SettingRow>

    <!-- 图片模糊度（仅在有背景图片时显示） -->
    <SettingRow
      v-if="themeStore.backgroundImage"
      :title="t('appearance.bg.blur')"
      :description="t('appearance.bg.blur.desc')"
    >
      <SettingSlider
        :model-value="currentBgBlur"
        :min="0"
        :max="20"
        :step="1"
        unit="px"
        @update:model-value="(v) => themeStore.setBgBlur(v)"
      />
    </SettingRow>

    <!-- 应用透明度：值越高越透明，范围 5% ~ 95% -->
    <SettingRow
      :title="t('appearance.appOpacity')"
      :description="t('appearance.appOpacity.desc')"
    >
      <SettingSlider
        :model-value="currentAppOpacity"
        :min="0.05"
        :max="0.95"
        :step="0.05"
        unit="%"
        @update:model-value="(v) => themeStore.setAppOpacity(v)"
      />
    </SettingRow>

    <!-- 语言 -->
    <SettingRow :title="t('appearance.lang')" :description="t('appearance.lang.desc')">
      <SettingButtonGroup
        :model-value="currentLanguage"
        :options="languageOptions"
        @update:model-value="onLanguageChange"
      />
    </SettingRow>

    <!-- 主题编辑器弹窗 -->
    <ThemeEditor
      v-model:visible="showEditor"
      :theme="editingTheme"
      @save="onSaveTheme"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useThemeStore } from "@/stores/theme";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "@/composables/useI18n";
import type { ThemeMode } from "@/stores/theme";
import SettingRow from "@/components/settings/SettingRow.vue";
import SettingButtonGroup from "@/components/settings/SettingButtonGroup.vue";
import SettingSlider from "@/components/settings/SettingSlider.vue";
import ThemeSelector from "@/components/settings/ThemeSelector.vue";
import ThemeEditor from "@/components/settings/ThemeEditor.vue";
import ColorPicker from "@/components/settings/ColorPicker.vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ThemeDefinition } from "@/types/theme";

const themeStore = useThemeStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();

const showEditor = ref(false);

/** 当前编辑的主题 */
const editingTheme = computed<ThemeDefinition | null>(() => {
  const id = themeStore.currentThemeId;
  const mode = themeStore.resolvedTheme;
  return themeStore.availableThemes.find((th: ThemeDefinition) => th.id === id && th.mode === mode) ?? null;
});

// --- 选项配置 ---

const modeOptions = computed(() => [
  { value: "light", label: t("appearance.mode.light") },
  { value: "dark", label: t("appearance.mode.dark") },
  { value: "system", label: t("appearance.mode.system") },
]);

const accentSourceOptions = computed(() => [
  { value: "system", label: t("appearance.accent.system") },
  { value: "theme", label: t("appearance.accent.theme") },
  { value: "custom", label: t("appearance.accent.custom") },
]);

const windowEffectOptions = computed(() => [
  { value: "auto", label: t("appearance.effect.auto") },
  { value: "mica", label: t("appearance.effect.mica") },
  { value: "acrylic", label: t("appearance.effect.acrylic") },
  { value: "none", label: t("appearance.effect.none") },
]);

const languageOptions = computed(() => [
  { value: "zh-CN", label: t("appearance.lang.zh") },
  { value: "en", label: t("appearance.lang.en") },
]);

// --- 计算属性 ---

const currentMode = computed<ThemeMode>(
  () => (settingsStore.config?.appearance?.theme as ThemeMode) ?? "system"
);

const currentBgBlur = computed(
  () => settingsStore.config?.appearance?.bg_blur ?? 0
);

const currentAppOpacity = computed(
  () => settingsStore.config?.appearance?.app_opacity ?? 0
);

const currentLanguage = computed(
  () => settingsStore.config?.appearance?.language ?? "zh-CN"
);

// --- 事件处理 ---

async function onSelectBackgroundImage() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({
    filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png", "webp", "bmp"] }],
    multiple: false,
  });
  if (!path) return;
  const filePath = typeof path === "string" ? path : (path as any).path;
  if (!filePath) return;
  const imageUrl = convertFileSrc(filePath);
  await themeStore.setBackgroundImage(imageUrl);
}

async function onClearBackgroundImage() {
  await themeStore.setBackgroundImage(null);
}

async function onLanguageChange(lang: string) {
  await settingsStore.patchAppearance({ language: lang });
}

// --- 主题编辑器 ---

async function onSaveTheme(theme: ThemeDefinition) {
  const result = await themeStore.importTheme(theme);
  if (result.success) {
    showEditor.value = false;
  } else {
    console.error("[AppearanceSection] Failed to save theme:", result.error);
  }
}

async function onImportTheme() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({
    filters: [{ name: "Theme", extensions: ["json"] }],
    multiple: false,
  });
  if (!path) return;
  const filePath = typeof path === "string" ? path : (path as any).path;
  if (!filePath) return;

  try {
    const { readTextFile } = await import("@tauri-apps/plugin-fs");
    const content = await readTextFile(filePath);
    const json = JSON.parse(content);
    const result = await themeStore.importTheme(json);
    if (!result.success) {
      console.error("[AppearanceSection] Failed to import theme:", result.error);
    }
  } catch (e) {
    console.error("[AppearanceSection] Failed to read theme file:", e);
  }
}

async function onExportTheme(id: string) {
  const theme = await themeStore.exportTheme(id);
  if (!theme) return;

  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      filters: [{ name: "Theme", extensions: ["json"] }],
      defaultPath: `${theme.name}.json`,
    });
    if (!path) return;

    const { writeTextFile } = await import("@tauri-apps/plugin-fs");
    await writeTextFile(path, JSON.stringify(theme, null, 2));
  } catch (e) {
    console.error("[AppearanceSection] Failed to export theme:", e);
  }
}
</script>

<style scoped>
.appearance-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.section-title {
  font-size: var(--font-size-xl, 16px);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 8px;
}

.bg-image-controls {
  display: flex;
  gap: 8px;
}

.bg-image-btn {
  padding: 4px 12px;
  border: 1px solid var(--color-border-strong, rgba(255, 255, 255, 0.08));
  border-radius: var(--radius-sm, 4px);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.bg-image-btn:hover {
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.06));
}

.bg-image-btn--danger {
  color: var(--color-danger, #e81123);
  border-color: var(--color-danger, #e81123);
}

.bg-image-btn--danger:hover {
  background: var(--color-danger-subtle, rgba(232, 17, 35, 0.1));
}
</style>
