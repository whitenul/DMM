import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "@/stores/settings";
import { useTauriCommand } from "@/composables/useTauriCommand";
import { getBuiltinTheme, getBuiltinThemeIds } from "@/themes/builtin";
import { deriveAccentColors, resolveAlphaColor } from "@/utils/color";
import type { ThemeDefinition, ResolvedThemeColors } from "@/types/theme";
import { THEME_COLOR_MAP, ThemeDefinitionSchema } from "@/types/theme";

export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

// --- 工具函数 ---

/** 将本地文件路径转为 Tauri 资源 URL，http/https/asset:// 原样返回 */
function toAssetUrl(path: string | null): string | null {
  if (!path) return null;
  if (path.startsWith("http://") || path.startsWith("https://") || path.startsWith("asset://")) return path;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

/** 将 ThemeMode 解析为具体的亮/暗主题 */
function resolve(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

/** 从 ThemeDefinition 派生完整色集，填充可选字段并解析透明度颜色 */
function deriveThemeColors(theme: ThemeDefinition): ResolvedThemeColors {
  const mode = theme.mode;
  const c = theme.colors;

  const accentDerived = deriveAccentColors(c.accent, mode);
  const dangerDerived = deriveAccentColors(c.danger, mode);
  const warningDerived = deriveAccentColors(c.warning, mode);
  const successDerived = deriveAccentColors(c.success, mode);

  return {
    accent: c.accent,
    accentHover: c.accentHover ?? accentDerived.accentHover,
    accentPressed: c.accentPressed ?? accentDerived.accentPressed,
    accentSubtle: c.accentSubtle ?? accentDerived.accentSubtle,
    accentText: c.accentText ?? accentDerived.accentText,
    bgSolid: c.bgSolid,
    bgCard: c.bgCard,
    bgHover: resolveAlphaColor(c.bgHover, mode, "bgHover"),
    bgActive: resolveAlphaColor(c.bgActive, mode, "bgActive"),
    bgSubtle: c.bgSubtle,
    bgInset: c.bgInset,
    textPrimary: resolveAlphaColor(c.textPrimary, mode, "textPrimary"),
    textSecondary: resolveAlphaColor(c.textSecondary, mode, "textSecondary"),
    textTertiary: resolveAlphaColor(c.textTertiary, mode, "textTertiary"),
    textDisabled: resolveAlphaColor(c.textDisabled, mode, "textDisabled"),
    textOnAccent: c.textOnAccent,
    textOnDanger: c.textOnDanger,
    danger: c.danger,
    dangerHover: c.dangerHover ?? dangerDerived.accentHover,
    dangerPressed: c.dangerPressed ?? dangerDerived.accentPressed,
    dangerSubtle: c.dangerSubtle ?? dangerDerived.accentSubtle,
    warning: c.warning,
    warningSubtle: c.warningSubtle ?? warningDerived.accentSubtle,
    success: c.success,
    successSubtle: c.successSubtle ?? successDerived.accentSubtle,
    border: resolveAlphaColor(c.border, mode, "border"),
    borderStrong: resolveAlphaColor(c.borderStrong, mode, "borderStrong"),
    borderAccent: c.borderAccent,
    divider: resolveAlphaColor(c.divider, mode, "divider"),
    overlay: resolveAlphaColor(c.overlay, mode, "overlay"),
    closeHover: c.closeHover,
  };
}

// --- Pinia Store ---

export const useThemeStore = defineStore("theme", () => {
  const { call } = useTauriCommand();

  // --- 响应式状态 ---
  const resolvedTheme = ref<ResolvedTheme>("dark");
  const currentThemeId = ref<string>("default");
  const currentAccentSource = ref<"system" | "theme" | "custom">("system");
  const accentColor = ref<string>("#0078d4");
  const availableThemes = ref<ThemeDefinition[]>([]);
  const customColorOverrides = ref<Record<string, string>>({});
  const currentWindowEffect = ref<string>("auto");
  const backgroundImage = ref<string | null>(null);
  const bgBlur = ref<number>(0);
  const appOpacity = ref<number>(0);

  let mediaQuery: MediaQueryList | null = null;
  let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;

  // --- 计算属性 ---
  const isDark = computed(() => resolvedTheme.value === "dark");

  // --- DOM 操作 ---

  /** 将主题颜色注入 DOM CSS 变量，同时应用自定义颜色覆盖和背景图 */
  function applyThemeToDom(colors: ResolvedThemeColors, mode: ResolvedTheme) {
    if (typeof document === "undefined") return;

    const root = document.documentElement;

    if (mode === "dark") {
      root.setAttribute("data-theme", "dark");
    } else {
      root.removeAttribute("data-theme");
    }

    for (const [key, cssVar] of Object.entries(THEME_COLOR_MAP)) {
      root.style.setProperty(cssVar, colors[key as keyof ResolvedThemeColors]);
    }

    // 自定义颜色覆盖（最高优先级）
    for (const [key, value] of Object.entries(customColorOverrides.value)) {
      const cssVar = THEME_COLOR_MAP[key as keyof ResolvedThemeColors];
      if (cssVar) {
        root.style.setProperty(cssVar, value);
      }
    }

    applyWindowEffectAttribute(currentWindowEffect.value);
    applyBackgroundImage(backgroundImage.value);

    resolvedTheme.value = mode;
  }

  /** 同步 data-window-effect 属性到 <html>，供 CSS 选择器使用 */
  function applyWindowEffectAttribute(effect: string) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    if (effect === "mica" || effect === "acrylic") {
      root.setAttribute("data-window-effect", effect);
    } else {
      root.removeAttribute("data-window-effect");
    }
  }

  /** 清除原生 DWM 窗口效果，视觉外观由 CSS 处理以实现平滑过渡 */
  async function applyNativeWindowEffect(_effect: string | null) {
    try {
      const win = getCurrentWindow();
      await win.clearEffects();
      // clearEffects 后 WebView2 可能回退到默认背景，需重置为透明
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("reset_window_background");
      } catch {
        // 非关键：背景重置是安全措施
      }
    } catch {
      // clearEffects 在某些平台可能失败，静默忽略
    }
  }

  /** 设置背景图（本地路径经 convertFileSrc 转换），backdrop-filter 由 CSS 控制 */
  function applyBackgroundImage(url: string | null) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      const assetUrl = toAssetUrl(url);
      bgLayer.style.backgroundImage = assetUrl ? `url(${assetUrl})` : "none";
    }
    // CSS 据此判断是否禁用 backdrop-filter
    root.style.setProperty("--has-bg-image", url ? "1" : "0");
  }

  /** 设置背景图模糊，略微放大图层以防止边缘溢出 */
  function applyBgBlur(blur: number) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--effect-bg-blur", `${blur}px`);
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      if (blur > 0) {
        bgLayer.style.filter = `blur(${blur}px)`;
        bgLayer.style.transform = "scale(1.05)";
      } else {
        bgLayer.style.filter = "none";
        bgLayer.style.transform = "none";
      }
    }
  }

  /** 设置应用窗口透明度，由 CSS 变量驱动 */
  async function applyAppOpacity(opacity: number) {
    if (typeof document === "undefined") return;
    document.documentElement.style.setProperty("--app-opacity", String(opacity));
  }

  /** 按 ID 加载主题并应用到 DOM，未找到时回退到默认主题 */
  function loadAndApplyTheme(themeId: string, mode: ResolvedTheme): boolean {
    const builtin = getBuiltinTheme(themeId, mode);
    if (builtin) {
      const colors = deriveThemeColors(builtin);
      applyThemeToDom(colors, mode);
      return true;
    }

    const custom = availableThemes.value.find(t => t.id === themeId && t.mode === mode);
    if (custom) {
      const colors = deriveThemeColors(custom);
      applyThemeToDom(colors, mode);
      return true;
    }

    const fallback = getBuiltinTheme("default", mode);
    if (fallback) {
      const colors = deriveThemeColors(fallback);
      applyThemeToDom(colors, mode);
    }
    return false;
  }

  // --- 强调色 ---

  /** 用自定义十六进制值覆盖当前主题强调色，自动派生 hover/pressed/subtle 变体 */
  function applyAccentOverride(hex: string) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const derived = deriveAccentColors(hex, resolvedTheme.value);
    root.style.setProperty("--theme-accent", hex);
    root.style.setProperty("--theme-accent-hover", derived.accentHover);
    root.style.setProperty("--theme-accent-pressed", derived.accentPressed);
    root.style.setProperty("--theme-accent-subtle", derived.accentSubtle);
    root.style.setProperty("--theme-accent-text", derived.accentText);
    root.style.setProperty("--theme-border-accent", hex);
  }

  /** 根据 accentSource 应用强调色：system 读系统色、custom 用自定义色、theme 用主题内置色 */
  async function applyAccentSource() {
    if (currentAccentSource.value === "system") {
      try {
        const sysAccent = await call<string>("get_system_accent_color");
        accentColor.value = sysAccent;
        applyAccentOverride(sysAccent);
      } catch {
        // 回退到当前主题的强调色
        const theme = getBuiltinTheme(currentThemeId.value, resolvedTheme.value)
          ?? availableThemes.value.find(t => t.id === currentThemeId.value && t.mode === resolvedTheme.value);
        if (theme) accentColor.value = theme.colors.accent;
      }
    } else if (currentAccentSource.value === "custom") {
      const settingsStore = useSettingsStore();
      const custom = settingsStore.config?.appearance.custom_accent_color;
      if (custom) {
        accentColor.value = custom;
        applyAccentOverride(custom);
      }
    } else {
      // accentSource 为 "theme"
      const theme = getBuiltinTheme(currentThemeId.value, resolvedTheme.value)
        ?? availableThemes.value.find(t => t.id === currentThemeId.value && t.mode === resolvedTheme.value);
      if (theme) accentColor.value = theme.colors.accent;
    }
  }

  // --- 系统主题监听 ---

  function startSystemListener() {
    stopSystemListener();
    if (typeof window === "undefined") return;
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQueryListener = (e) => {
      const settingsStore = useSettingsStore();
      if (settingsStore.config?.appearance.theme === "system") {
        const newMode = e.matches ? "dark" : "light";
        loadAndApplyTheme(currentThemeId.value, newMode);
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

  // --- 初始化 ---

  /** 初始化主题系统，读取持久化设置并应用主题、背景、透明度等 */
  async function init() {
    const settingsStore = useSettingsStore();
    const appearance = settingsStore.config?.appearance;
    const mode = (appearance?.theme as ThemeMode) ?? "system";
    const resolved = resolve(mode);

    currentThemeId.value = appearance?.theme_id ?? "default";
    currentAccentSource.value = appearance?.accent_source ?? "system";

    backgroundImage.value = appearance?.background_image ?? null;

    const blur = appearance?.bg_blur ?? 0;
    applyBgBlur(blur);
    bgBlur.value = blur;

    applyBackgroundImage(backgroundImage.value);

    const appOpacityVal = appearance?.app_opacity ?? 0;
    await applyAppOpacity(appOpacityVal);
    appOpacity.value = appOpacityVal;

    await loadCustomThemes();

    loadAndApplyTheme(currentThemeId.value, resolved);

    await applyAccentSource();

    startSystemListener();
  }

  // --- Setter 方法 ---

  async function setMode(mode: ThemeMode) {
    const settingsStore = useSettingsStore();
    const resolved = resolve(mode);
    loadAndApplyTheme(currentThemeId.value, resolved);
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme: mode });
  }

  async function setTheme(id: string) {
    const settingsStore = useSettingsStore();
    currentThemeId.value = id;
    loadAndApplyTheme(id, resolvedTheme.value);
    currentAccentSource.value = "theme";
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme_id: id, accent_source: "theme" });
  }

  async function setAccentColor(hex: string) {
    const settingsStore = useSettingsStore();
    accentColor.value = hex;
    currentAccentSource.value = "custom";
    applyAccentOverride(hex);
    await settingsStore.patchAppearance({ accent_source: "custom", custom_accent_color: hex });
  }

  async function setAccentSource(src: "system" | "theme" | "custom") {
    const settingsStore = useSettingsStore();
    currentAccentSource.value = src;
    await applyAccentSource();
    const patch: Record<string, unknown> = { accent_source: src };
    if (src !== "custom") patch.custom_accent_color = null;
    await settingsStore.patchAppearance(patch);
  }

  async function setWindowEffect(effect: string) {
    const settingsStore = useSettingsStore();
    currentWindowEffect.value = effect;
    applyWindowEffectAttribute(effect);
    await applyNativeWindowEffect(effect);
    await settingsStore.patchAppearance({ effect });
  }

  async function setBackgroundImage(url: string | null) {
    const settingsStore = useSettingsStore();
    backgroundImage.value = url;
    applyBackgroundImage(url);
    await settingsStore.patchAppearance({ background_image: url });
  }

  async function setBgBlur(blur: number) {
    const settingsStore = useSettingsStore();
    bgBlur.value = blur;
    applyBgBlur(blur);
    await settingsStore.patchAppearance({ bg_blur: blur });
  }

  /** 设置应用透明度，限制在 5%-95% 范围内防止不可见或完全不透明 */
  async function setAppOpacity(opacity: number) {
    const settingsStore = useSettingsStore();
    const clamped = Math.max(0.05, Math.min(0.95, opacity));
    appOpacity.value = clamped;
    applyAppOpacity(clamped);
    await settingsStore.patchAppearance({ app_opacity: clamped });
  }

  /** 覆盖单个主题颜色，value 为 null 时移除覆盖 */
  async function setCustomColorOverride(key: string, value: string | null) {
    if (value === null) {
      delete customColorOverrides.value[key];
    } else {
      customColorOverrides.value[key] = value;
    }
    if (typeof document !== "undefined") {
      const cssVar = THEME_COLOR_MAP[key as keyof ResolvedThemeColors];
      if (cssVar) {
        const root = document.documentElement;
        if (value === null) {
          root.style.removeProperty(cssVar);
        } else {
          root.style.setProperty(cssVar, value);
        }
      }
    }
  }

  function clearCustomColorOverrides() {
    customColorOverrides.value = {};
    loadAndApplyTheme(currentThemeId.value, resolvedTheme.value);
  }

  // --- 自定义主题管理 ---

  async function loadCustomThemes() {
    try {
      const customs = await call<ThemeDefinition[]>("list_custom_themes");
      const builtins = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
      availableThemes.value = [...builtins, ...customs];
    } catch {
      availableThemes.value = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
    }
  }

  /** 从 JSON 导入主题（Zod 校验），内置主题 ID 加 "imported_" 前缀避免冲突 */
  async function importTheme(json: unknown): Promise<{ success: boolean; error?: string }> {
    const result = ThemeDefinitionSchema.safeParse(json);
    if (!result.success) {
      const errors = result.error.issues.map(i => `${i.path.join(".")}: ${i.message}`).join("; ");
      return { success: false, error: errors };
    }

    let theme = result.data;
    if (getBuiltinThemeIds().includes(theme.id)) {
      theme = { ...theme, id: `imported_${theme.id}` };
    }

    try {
      await call("save_custom_theme", { theme });
      await loadCustomThemes();
      return { success: true };
    } catch (e) {
      return { success: false, error: String(e) };
    }
  }

  async function exportTheme(id: string): Promise<ThemeDefinition | null> {
    const light = getBuiltinTheme(id, "light");
    const dark = getBuiltinTheme(id, "dark");
    if (light || dark) return light ?? dark!;
    return availableThemes.value.find(t => t.id === id) ?? null;
  }

  /** 删除自定义主题，若删除的是当前主题则回退到默认 */
  async function deleteTheme(id: string) {
    const theme = availableThemes.value.find(t => t.id === id);
    if (theme?.isBuiltIn) throw new Error("Cannot delete built-in theme");

    await call("delete_custom_theme", { theme_id: id });
    await loadCustomThemes();

    if (currentThemeId.value === id) {
      await setTheme("default");
    }
  }

  // --- 导出 ---

  return {
    // 状态
    resolvedTheme,
    isDark,
    currentThemeId,
    currentAccentSource,
    accentColor,
    availableThemes,
    customColorOverrides,
    currentWindowEffect,
    backgroundImage,
    bgBlur,
    appOpacity,

    // 初始化
    init,

    // 模式与主题
    setMode,
    setTheme,

    // 强调色
    setAccentColor,
    setAccentSource,

    // 窗口效果与背景
    setWindowEffect,
    setBackgroundImage,
    setBgBlur,
    setAppOpacity,

    // 自定义颜色覆盖
    setCustomColorOverride,
    clearCustomColorOverrides,

    // 自定义主题管理
    loadCustomThemes,
    importTheme,
    exportTheme,
    deleteTheme,

    // 系统监听
    startSystemListener,
    stopSystemListener,
  };
});
