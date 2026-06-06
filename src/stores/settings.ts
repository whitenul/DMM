import { defineStore } from "pinia";
import { ref } from "vue";
import { useTauriCommand } from "@/composables/useTauriCommand";

const VALID_CLOSE_BEHAVIORS = ["ask", "minimize_to_tray", "quit"] as const;
export type CloseBehavior = typeof VALID_CLOSE_BEHAVIORS[number];

export function validateCloseBehavior(v: string): CloseBehavior {
  return (VALID_CLOSE_BEHAVIORS as readonly string[]).includes(v) ? v as CloseBehavior : "ask";
}

export interface AppConfig {
  window: { x: number; y: number; width: number; height: number; edge_snap: boolean };
  shortcut: { global_search: string };
  appearance: {
    theme: string;
    effect: string;
    language: string;
    theme_id: string;
    accent_source: "system" | "theme" | "custom";
    custom_accent_color: string | null;
    background_image: string | null;
    bg_blur: number;
    app_opacity: number;
  };
  scan: { auto_scan_on_start: boolean; scan_start_menu: boolean; scan_uwp: boolean };
  close_behavior: "ask" | "minimize_to_tray" | "quit";
  autostart: boolean;
}

const DEFAULT_CONFIG: AppConfig = {
  window: { x: 100, y: 100, width: 800, height: 600, edge_snap: true },
  shortcut: { global_search: "Ctrl+Shift+Space" },
  appearance: { theme: "system", effect: "auto", language: "zh-CN", theme_id: "default", accent_source: "system" as const, custom_accent_color: null, background_image: null, bg_blur: 0, app_opacity: 0 },
  scan: { auto_scan_on_start: true, scan_start_menu: true, scan_uwp: true },
  close_behavior: "ask",
  autostart: false,
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
    // 校验 close_behavior
    if (loaded.close_behavior) {
      loaded.close_behavior = validateCloseBehavior(loaded.close_behavior);
    }
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

  async function toggleAutostart(enabled: boolean) {
    try {
      if (enabled) {
        await import('@tauri-apps/plugin-autostart').then(m => m.enable());
      } else {
        await import('@tauri-apps/plugin-autostart').then(m => m.disable());
      }
      if (config.value) {
        config.value.autostart = enabled;
      }
    } catch (e) {
      console.warn('[Settings] Autostart toggle failed:', e);
    }
  }

  return { config, loading, loadSettings, updateSettings, patchAppearance, patchCloseBehavior, toggleAutostart };
});
