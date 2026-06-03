import { defineStore } from "pinia";
import { ref } from "vue";
import { useTauriCommand } from "@/composables/useTauriCommand";

export interface AppConfig {
  window: { x: number; y: number; width: number; height: number; edge_snap: boolean };
  shortcut: { global_search: string };
  appearance: { theme: string; effect: string; language: string };
  scan: { auto_scan_on_start: boolean; scan_start_menu: boolean; scan_uwp: boolean };
  close_behavior: "ask" | "minimize_to_tray" | "quit";
}

const DEFAULT_CONFIG: AppConfig = {
  window: { x: 100, y: 100, width: 800, height: 600, edge_snap: true },
  shortcut: { global_search: "Ctrl+Shift+Space" },
  appearance: { theme: "system", effect: "auto", language: "zh-CN" },
  scan: { auto_scan_on_start: true, scan_start_menu: true, scan_uwp: true },
  close_behavior: "ask",
};

/**
 * 应用配置 store —— 只管数据，不碰 DOM
 *
 * 主题应用、language 切换等副作用由 useTheme / useI18n 等 composable 负责
 */
export const useSettingsStore = defineStore("settings", () => {
  const { call } = useTauriCommand();
  const config = ref<AppConfig | null>(null);
  const loading = ref(false);

  async function loadSettings() {
    loading.value = true;
    try {
      const loaded = await call<AppConfig>("load_settings");
      config.value = loaded;
    } catch (e) {
      console.warn("[Settings] Failed to load, using defaults:", e);
      config.value = { ...DEFAULT_CONFIG };
    } finally {
      loading.value = false;
    }
  }

  async function updateSettings(newConfig: AppConfig) {
    await call("update_settings", { settings: newConfig });
    config.value = newConfig;
  }

  async function patchAppearance(patch: Partial<AppConfig["appearance"]>) {
    if (!config.value) {
      config.value = { ...DEFAULT_CONFIG };
    }
    const next: AppConfig = {
      ...config.value,
      appearance: { ...config.value.appearance, ...patch },
    };
    await updateSettings(next);
  }

  async function patchCloseBehavior(behavior: AppConfig["close_behavior"]) {
    if (!config.value) {
      config.value = { ...DEFAULT_CONFIG };
    }
    const next: AppConfig = {
      ...config.value,
      close_behavior: behavior,
    };
    await updateSettings(next);
  }

  return { config, loading, loadSettings, updateSettings, patchAppearance, patchCloseBehavior };
});
