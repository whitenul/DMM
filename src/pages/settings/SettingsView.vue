<template>
  <div class="settings-view">
    <div class="settings-header">
      <button class="back-btn" @click="onBack" title="返回主界面">
        <AppIcon name="chevron-left" :size="14" />
        <span>返回</span>
      </button>
      <h2 class="settings-title">设置</h2>
    </div>

    <div v-if="settingsStore.loading" class="settings-loading">
      <span>加载中...</span>
    </div>
    <div v-else class="settings-body">
      <section class="settings-section">
        <h3 class="section-title">外观</h3>
        <div class="setting-row">
          <span class="setting-label">主题</span>
          <div class="theme-selector">
            <button
              v-for="t in themes"
              :key="t.value"
              class="theme-option"
              :class="{ active: currentTheme === t.value }"
              @click="onThemeChange(t.value)"
            >
              <AppIcon :name="t.icon" :size="14" />
              <span>{{ t.label }}</span>
            </button>
          </div>
        </div>
        <div class="setting-row">
          <span class="setting-label">窗口效果</span>
          <div class="theme-selector">
            <button
              v-for="e in effects"
              :key="e.value"
              class="theme-option"
              :class="{ active: currentEffect === e.value }"
              @click="updateAppearance('effect', e.value)"
            >
              <span>{{ e.label }}</span>
            </button>
          </div>
        </div>
        <div class="setting-row">
          <span class="setting-label">关闭行为</span>
          <div class="theme-selector">
            <button
              v-for="b in closeBehaviors"
              :key="b.value"
              class="theme-option"
              :class="{ active: currentCloseBehavior === b.value }"
              :title="b.hint"
              @click="updateCloseBehavior(b.value)"
            >
              <AppIcon :name="b.icon" :size="14" />
              <span>{{ b.label }}</span>
            </button>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">快捷键</h3>
        <div class="setting-row">
          <span class="setting-label">全局搜索</span>
          <span class="setting-value">{{ currentShortcut }}</span>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">扫描</h3>
        <div class="setting-row">
          <span class="setting-label">启动时自动扫描</span>
          <label class="toggle">
            <input
              :checked="currentScan.auto_scan_on_start"
              type="checkbox"
              class="toggle-input"
              @change="updateScan('auto_scan_on_start', ($event.target as HTMLInputElement).checked)"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="setting-row">
          <span class="setting-label">扫描开始菜单</span>
          <label class="toggle">
            <input
              :checked="currentScan.scan_start_menu"
              type="checkbox"
              class="toggle-input"
              @change="updateScan('scan_start_menu', ($event.target as HTMLInputElement).checked)"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="setting-row">
          <span class="setting-label">扫描 UWP 应用</span>
          <label class="toggle">
            <input
              :checked="currentScan.scan_uwp"
              type="checkbox"
              class="toggle-input"
              @change="updateScan('scan_uwp', ($event.target as HTMLInputElement).checked)"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="setting-row">
          <span class="setting-label">扫描目标分类</span>
          <div class="theme-selector">
            <button
              v-for="cat in categoryStore.categories"
              :key="cat.id"
              class="theme-option"
              :class="{ active: scanTargetCategoryId === cat.id }"
              :title="cat.name"
              @click="scanTargetCategoryId = cat.id"
            >
              <AppIcon :name="cat.icon || 'Folder'" :size="14" />
              <span>{{ cat.name }}</span>
            </button>
          </div>
        </div>
        <div class="setting-row">
          <span class="setting-label">触发方式</span>
          <div class="theme-selector">
            <button
              v-for="opt in scanSourceOptions"
              :key="opt.value"
              class="theme-option"
              :class="{ active: scanSource === opt.value }"
              :title="opt.hint"
              @click="scanSource = opt.value"
            >
              <AppIcon :name="opt.icon" :size="14" />
              <span>{{ opt.label }}</span>
            </button>
          </div>
        </div>
        <div class="setting-row">
          <button class="btn btn-primary" :disabled="scanning || !scanTargetCategoryId" @click="onScanNow">
            {{ scanning ? "扫描中..." : "立即扫描" }}
          </button>
          <span class="scan-status">{{ scanStatusText }}</span>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">关于</h3>
        <div class="setting-row">
          <span class="setting-label">版本</span>
          <span class="setting-value">0.1.0</span>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useSettingsStore } from "@/stores/settings";
import { useCategoryStore } from "@/stores/category";
import { useScanStore } from "@/stores/scan";
import { useToastStore } from "@/stores/toast";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import type { CloseBehavior } from "@/components/common/CloseConfirmDialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";

const settingsStore = useSettingsStore();
const categoryStore = useCategoryStore();
const scanStore = useScanStore();
const toast = useToastStore();
const router = useRouter();
const theme = useTheme();

type ScanSource = "start_menu" | "uwp" | "both";

const scanTargetCategoryId = ref<number | null>(null);
const scanSource = ref<ScanSource>("both");
const scanning = ref(false);
const lastScanResult = ref<{ imported: number; total: number } | null>(null);

const scanSourceOptions: { value: ScanSource; label: string; icon: string; hint: string }[] = [
  { value: "start_menu", label: "开始菜单", icon: "search", hint: "扫描开始菜单快捷方式" },
  { value: "uwp", label: "UWP 应用", icon: "search", hint: "扫描已安装的 UWP 应用" },
  { value: "both", label: "全部", icon: "search", hint: "同时扫描开始菜单和 UWP 应用" },
];

const scanStatusText = computed(() => {
  if (scanning.value) return "正在扫描，请稍候...";
  if (lastScanResult.value) {
    const r = lastScanResult.value;
    if (r.total === 0) return "未发现任何新应用";
    return `上次扫描：发现 ${r.total} 个应用，新增 ${r.imported} 个`;
  }
  return "";
});

const themes: { value: ThemeMode; label: string; icon: string }[] = [
  { value: "light", label: "浅色", icon: "settings" },
  { value: "dark", label: "深色", icon: "settings" },
  { value: "system", label: "跟随系统", icon: "settings" },
];

const effects = [
  { value: "auto", label: "自动" },
  { value: "mica", label: "Mica" },
  { value: "acrylic", label: "亚克力" },
  { value: "none", label: "无" },
];

const closeBehaviors: {
  value: CloseBehavior;
  label: string;
  icon: string;
  hint: string;
}[] = [
  {
    value: "ask",
    label: "每次询问",
    icon: "settings",
    hint: "点 X 时弹出选择对话框",
  },
  {
    value: "minimize_to_tray",
    label: "最小化到托盘",
    icon: "chevron-down",
    hint: "关闭窗口，应用继续在托盘运行",
  },
  {
    value: "quit",
    label: "退出应用",
    icon: "close",
    hint: "关闭窗口即完全退出",
  },
];

const currentTheme = computed(() => settingsStore.config?.appearance.theme ?? "system");
const currentEffect = computed(() => settingsStore.config?.appearance.effect ?? "auto");
const currentCloseBehavior = computed(
  () => settingsStore.config?.close_behavior ?? "ask"
);
const currentShortcut = computed(
  () => settingsStore.config?.shortcut.global_search ?? "Ctrl+Shift+Space"
);
const currentScan = computed(
  () =>
    settingsStore.config?.scan ?? {
      auto_scan_on_start: true,
      scan_start_menu: true,
      scan_uwp: true,
    }
);

onMounted(async () => {
  if (!settingsStore.config) {
    await settingsStore.loadSettings();
  }
  if (categoryStore.categories.length === 0) {
    await categoryStore.fetchCategories();
  }
  if (scanTargetCategoryId.value === null && categoryStore.categories.length > 0) {
    scanTargetCategoryId.value = categoryStore.categories[0].id;
  }
});

function onThemeChange(mode: ThemeMode) {
  theme.setMode(mode);
}

async function updateAppearance(key: string, value: string) {
  if (!settingsStore.config) return;
  try {
    await settingsStore.updateSettings({
      ...settingsStore.config,
      appearance: { ...settingsStore.config.appearance, [key]: value },
    });
  } catch (e) {
    toast.error("保存设置失败");
  }
}

async function updateCloseBehavior(behavior: CloseBehavior) {
  try {
    await settingsStore.patchCloseBehavior(behavior);
  } catch (e) {
    toast.error("保存设置失败");
  }
}

async function updateScan(key: string, value: boolean) {
  if (!settingsStore.config) return;
  try {
    await settingsStore.updateSettings({
      ...settingsStore.config,
      scan: { ...settingsStore.config.scan, [key]: value },
    });
  } catch (e) {
    toast.error("保存设置失败");
  }
}

function onBack() {
  router.push("/");
}

async function onScanNow() {
  if (scanTargetCategoryId.value === null) {
    toast.warning("请先选择一个目标分类");
    return;
  }
  if (scanning.value) return;

  scanning.value = true;
  lastScanResult.value = null;
  const targetId = scanTargetCategoryId.value;

  try {
    let totalScanned = 0;
    let totalImported = 0;

    if (scanSource.value === "start_menu" || scanSource.value === "both") {
      await scanStore.scanStartMenu();
      const apps = scanStore.scannedApps;
      totalScanned += apps.length;
      if (apps.length > 0) {
        const imported = await scanStore.importApps(targetId, apps);
        totalImported += imported;
      }
    }

    if (scanSource.value === "uwp" || scanSource.value === "both") {
      await scanStore.scanUwpApps();
      const apps = scanStore.scannedApps;
      totalScanned += apps.length;
      if (apps.length > 0) {
        const imported = await scanStore.importApps(targetId, apps);
        totalImported += imported;
      }
    }

    lastScanResult.value = { imported: totalImported, total: totalScanned };
    if (totalImported > 0) {
      toast.success(`扫描完成：新增 ${totalImported} 个应用`);
    } else if (totalScanned > 0) {
      toast.info(`扫描完成：发现 ${totalScanned} 个应用（全部已存在）`);
    } else {
      toast.info("扫描完成：未发现新应用");
    }
  } catch (e) {
    const msg = e && typeof e === "object" && "message" in e
      ? (e as { message: string }).message
      : "扫描失败";
    toast.error(msg);
  } finally {
    scanning.value = false;
    scanStore.clearResults();
  }
}
</script>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-solid);
  color: var(--color-text-primary);
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-bg-hover);
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.back-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.settings-title {
  font-size: var(--font-size-2xl);
  font-weight: 600;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 24px 24px;
}

.settings-loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-accent);
  margin-bottom: 12px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--color-bg-active);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-label {
  font-size: var(--font-size-md);
}

.setting-value {
  font-size: var(--font-size-md);
  color: var(--color-text-secondary);
}

.theme-selector {
  display: flex;
  gap: 6px;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border: 1px solid var(--color-bg-active);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.theme-option:hover {
  background: var(--color-bg-hover);
}

.theme-option.active {
  border-color: var(--color-accent);
  background: var(--color-bg-hover);
  color: var(--color-accent);
}

.toggle {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.toggle-input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: var(--color-bg-active);
  border-radius: 11px;
  transition: background var(--transition-fast);
}

.toggle-slider::before {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 3px;
  bottom: 3px;
  background: var(--color-bg-card);
  border-radius: 50%;
  transition: transform var(--transition-fast);
}

.toggle-input:checked + .toggle-slider {
  background: var(--color-accent);
}

.toggle-input:checked + .toggle-slider::before {
  transform: translateX(18px);
}

.btn {
  padding: 6px 20px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-bg-active);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: transparent;
  color: var(--color-text-primary);
}

.btn-primary {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-text-on-accent);
}

.btn-primary:hover {
  background: var(--color-accent-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.scan-status {
  margin-left: 12px;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
