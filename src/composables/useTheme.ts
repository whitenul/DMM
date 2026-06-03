import { ref, watch, readonly, onBeforeUnmount } from "vue";
import { useSettingsStore } from "@/stores/settings";

/**
 * Theme 类型
 * - "light" | "dark" 强制指定
 * - "system" 跟随系统 prefers-color-scheme
 */
export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

/**
 * 主题系统 —— 唯一权威源
 *
 * 职责：
 * 1. 维护 reactive themeMode (用户意图)
 * 2. 维护 reactive resolvedTheme (实际应用的主题)
 * 3. 应用到 document.documentElement[data-theme]
 * 4. 监听系统主题变化（仅在 system 模式下生效）
 * 5. 同步到 settings store 持久化
 *
 * 调用方式：
 * ```ts
 * // 应用启动时
 * useTheme().init()
 *
 * // UI 切换
 * useTheme().setMode("dark")
 * ```
 */
const resolvedTheme = ref<ResolvedTheme>("dark");
let mediaQuery: MediaQueryList | null = null;
let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;
let initCount = 0;

function resolve(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

function applyToDom(theme: ResolvedTheme) {
  if (typeof document === "undefined") return;
  if (theme === "dark") {
    document.documentElement.setAttribute("data-theme", "dark");
  } else {
    document.documentElement.removeAttribute("data-theme");
  }
  resolvedTheme.value = theme;
}

function startSystemListener() {
  stopSystemListener();
  if (typeof window === "undefined") return;
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQueryListener = (e) => {
    const settingsStore = useSettingsStore();
    if (settingsStore.config?.appearance.theme === "system") {
      applyToDom(e.matches ? "dark" : "light");
    }
  };
  mediaQuery.addEventListener("change", mediaQueryListener);
}

function stopSystemListener() {
  if (mediaQuery && mediaQueryListener) {
    mediaQuery.removeEventListener("change", mediaQueryListener);
  }
  mediaQuery = null;
  mediaQueryListener = null;
}

export function useTheme() {
  initCount++;
  const settingsStore = useSettingsStore();

  /**
   * 初始化主题（应用启动时调用一次）
   * 从 settings 读取用户偏好并应用，启动 system 模式监听
   */
  function init() {
    const mode = (settingsStore.config?.appearance.theme as ThemeMode) ?? "system";
    const resolved = resolve(mode);
    applyToDom(resolved);
    startSystemListener();
  }

  /**
   * 用户切换主题
   * 1. 立即更新 DOM（避免等待 settings 持久化）
   * 2. 持久化到 settings
   */
  async function setMode(mode: ThemeMode) {
    applyToDom(resolve(mode));
    await settingsStore.patchAppearance({ theme: mode });
  }

  /**
   * 监听 settings 变化（外部更新时同步 DOM）
   * 例如：直接改配置文件后重启，或者 IPC 推送新设置
   */
  const stopWatch = watch(
    () => settingsStore.config?.appearance.theme,
    (newMode) => {
      if (!newMode) return;
      const mode = newMode as ThemeMode;
      if (resolve(mode) !== resolvedTheme.value) {
        applyToDom(resolve(mode));
      }
    }
  );

  /**
   * composable 实例销毁时清理
   * （防止热更新或测试时多个 useTheme 实例泄漏监听器）
   */
  onBeforeUnmount(() => {
    initCount--;
    if (initCount <= 0) {
      stopSystemListener();
      stopWatch();
    }
  });

  const isDark = readonly(ref(resolvedTheme.value === "dark"));

  return {
    resolvedTheme: readonly(resolvedTheme),
    isDark,
    init,
    setMode,
  };
}
