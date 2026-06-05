<template>
  <div class="scan-section">
    <h3 class="section-title">{{ t('scan.title') }}</h3>

    <SettingRow :title="t('scan.autoOnStart')" :description="t('scan.autoOnStart.desc')">
      <SettingToggle
        :model-value="autoScanOnStart"
        @update:model-value="onAutoScanChange"
      />
    </SettingRow>

    <SettingRow :title="t('scan.startMenu')" :description="t('scan.startMenu.desc')">
      <SettingToggle
        :model-value="scanStartMenu"
        @update:model-value="onScanStartMenuChange"
      />
    </SettingRow>

    <SettingRow :title="t('scan.uwp')" :description="t('scan.uwp.desc')">
      <SettingToggle
        :model-value="scanUwp"
        @update:model-value="onScanUwpChange"
      />
    </SettingRow>

    <SettingRow :title="t('scan.targetCategory')" :description="t('scan.targetCategory.desc')">
      <SettingButtonGroup
        :model-value="selectedCategoryId"
        :options="categoryOptions"
        @update:model-value="onCategoryChange"
      />
    </SettingRow>

    <SettingRow :title="t('scan.trigger')" :description="t('scan.trigger')">
      <SettingButtonGroup
        :model-value="scanTrigger"
        :options="scanTriggerOptions"
        @update:model-value="onScanTriggerChange"
      />
    </SettingRow>

    <SettingRow :title="t('scan.runNow')" :description="t('scan.runNow')">
      <button
        class="scan-now-btn"
        :disabled="scanning"
        @click="onScanNow"
      >
        {{ scanning ? t('scan.running') : t('scan.runNow') }}
      </button>
    </SettingRow>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useCategoryStore } from "@/stores/category";
import { useI18n } from "@/composables/useI18n";
import { useTauriCommand } from "@/composables/useTauriCommand";
import SettingRow from "@/components/settings/SettingRow.vue";
import SettingToggle from "@/components/settings/SettingToggle.vue";
import SettingButtonGroup from "@/components/settings/SettingButtonGroup.vue";

const settingsStore = useSettingsStore();
const categoryStore = useCategoryStore();
const { call } = useTauriCommand();
const { t } = useI18n();

const scanning = ref(false);
const scanTrigger = ref<string>("all");
const selectedCategoryId = ref<number>(0);

const scanTriggerOptions = computed(() => [
  { value: "start_menu", label: t("scan.trigger.startMenu") },
  { value: "uwp", label: t("scan.trigger.uwp") },
  { value: "all", label: t("scan.trigger.all") },
]);

const autoScanOnStart = computed(
  () => settingsStore.config?.scan?.auto_scan_on_start ?? true
);

const scanStartMenu = computed(
  () => settingsStore.config?.scan?.scan_start_menu ?? true
);

const scanUwp = computed(
  () => settingsStore.config?.scan?.scan_uwp ?? true
);

const categoryOptions = computed(() => {
  const cats = categoryStore.categories.map((c) => ({
    value: c.id,
    label: c.name,
  }));
  return [{ value: 0, label: t("scan.targetCategory") === "scan.targetCategory" ? "Default" : "默认分类" }, ...cats];
});

async function patchScan(patch: Record<string, unknown>) {
  if (!settingsStore.config) return;
  await settingsStore.updateSettings({
    ...settingsStore.config,
    scan: { ...settingsStore.config.scan, ...patch },
  });
}

async function onAutoScanChange(val: boolean) {
  await patchScan({ auto_scan_on_start: val });
}

async function onScanStartMenuChange(val: boolean) {
  await patchScan({ scan_start_menu: val });
}

async function onScanUwpChange(val: boolean) {
  await patchScan({ scan_uwp: val });
}

function onCategoryChange(id: number) {
  selectedCategoryId.value = id;
}

function onScanTriggerChange(trigger: string) {
  scanTrigger.value = trigger;
}

async function onScanNow() {
  if (scanning.value) return;
  scanning.value = true;
  try {
    if (scanTrigger.value === "start_menu" || scanTrigger.value === "all") {
      await call("scan_start_menu");
    }
    if (scanTrigger.value === "uwp" || scanTrigger.value === "all") {
      await call("scan_uwp_apps");
    }
  } catch (e) {
    console.error("[ScanSection] Scan failed:", e);
  } finally {
    scanning.value = false;
  }
}

onMounted(async () => {
  if (categoryStore.categories.length === 0) {
    await categoryStore.fetchCategories();
  }
});
</script>

<style scoped>
.scan-section {
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

.scan-now-btn {
  padding: 6px 16px;
  border: 1px solid var(--color-accent, #0078d4);
  border-radius: var(--radius-sm, 4px);
  background: transparent;
  color: var(--color-accent, #0078d4);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.scan-now-btn:hover:not(:disabled) {
  background: var(--color-accent-subtle, rgba(0, 120, 212, 0.1));
}

.scan-now-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
