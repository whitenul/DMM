<template>
  <div class="about-section">
    <h3 class="section-title">{{ t('about.title') }}</h3>

    <div class="about-info">
      <div class="about-info__logo">
        <AppIcon name="app-icon" :size="48" />
      </div>
      <div class="about-info__text">
        <span class="about-info__name">{{ t('about.appName') }}</span>
        <span class="about-info__version">v{{ version || '...' }}</span>
      </div>
    </div>

    <SettingRow :title="t('about.version')" :description="t('about.version')">
      <span class="about-version-value">{{ version || '...' }}</span>
    </SettingRow>

    <SettingRow :title="t('about.autostart')" :description="t('about.autostart.desc')">
      <SettingToggle
        :model-value="autostart"
        @update:model-value="onAutostartChange"
      />
    </SettingRow>

    <SettingRow :title="t('about.closeBehavior')" :description="t('about.closeBehavior.desc')">
      <SettingButtonGroup
        :model-value="closeBehavior"
        :options="closeBehaviorOptions"
        @update:model-value="onCloseBehaviorChange"
      />
    </SettingRow>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "@/composables/useI18n";
import SettingRow from "@/components/settings/SettingRow.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import SettingButtonGroup from "@/components/settings/SettingButtonGroup.vue";
import AppIcon from "@/components/common/AppIcon.vue";

const settingsStore = useSettingsStore();
const { t } = useI18n();

const version = ref("");

const autostart = computed(() => settingsStore.config?.autostart ?? false);
const closeBehavior = computed(
  () => settingsStore.config?.close_behavior ?? "ask"
);

const closeBehaviorOptions = computed(() => [
  { value: "ask", label: t("about.closeBehavior.ask") },
  { value: "minimize_to_tray", label: t("about.closeBehavior.minimize") },
  { value: "quit", label: t("about.closeBehavior.quit") },
]);

async function onAutostartChange(enabled: boolean) {
  await settingsStore.toggleAutostart(enabled);
}

async function onCloseBehaviorChange(behavior: string) {
  await settingsStore.patchCloseBehavior(
    behavior as "ask" | "minimize_to_tray" | "quit"
  );
}

onMounted(async () => {
  try {
    version.value = await getVersion();
  } catch {
    version.value = "?";
  }
});
</script>

<style scoped>
.about-section {
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

.about-info {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 0;
  margin-bottom: 8px;
}

.about-info__logo {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg, 12px);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border, rgba(255, 255, 255, 0.06));
}

.about-info__text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.about-info__name {
  font-size: var(--font-size-xl, 16px);
  font-weight: 600;
  color: var(--color-text-primary);
}

.about-info__version {
  font-size: var(--font-size-sm);
  color: var(--color-text-tertiary);
}

.about-version-value {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-family: var(--font-family-mono, monospace);
}
</style>
