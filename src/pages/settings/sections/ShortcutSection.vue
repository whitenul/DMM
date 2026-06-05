<template>
  <div class="shortcut-section">
    <h3 class="section-title">{{ t('shortcut.title') }}</h3>

    <SettingRow :title="t('shortcut.globalSearch')" :description="t('shortcut.globalSearch.desc')">
      <ShortcutRecorder
        :model-value="globalSearchShortcut"
        @update:model-value="onShortcutChange"
      />
    </SettingRow>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "@/composables/useI18n";
import SettingRow from "@/components/settings/SettingRow.vue";
import ShortcutRecorder from "@/components/settings/ShortcutRecorder.vue";

const settingsStore = useSettingsStore();
const { t } = useI18n();

const globalSearchShortcut = computed(
  () => settingsStore.config?.shortcut?.global_search ?? "Ctrl+Shift+Space"
);

async function onShortcutChange(shortcut: string) {
  if (!settingsStore.config) return;
  await settingsStore.updateSettings({
    ...settingsStore.config,
    shortcut: { global_search: shortcut },
  });
}
</script>

<style scoped>
.shortcut-section {
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
</style>
