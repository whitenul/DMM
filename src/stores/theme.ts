import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/stores/settings";
import { useTauriCommand } from "@/composables/useTauriCommand";
import { getBuiltinTheme, getBuiltinThemeIds } from "@/themes/builtin";
import { deriveAccentColors, resolveAlphaColor } from "@/utils/color";
import type { ThemeDefinition, ResolvedThemeColors } from "@/types/theme";
import { THEME_COLOR_MAP, ThemeDefinitionSchema } from "@/types/theme";

export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

// ============================================================
// 工具函数
// ============================================================

/** 将本地文件路径转为 Tauri asset URL，http/https/asset:// 直接放行 */
function toAssetUrl(path: string | null): string | null {
  if (!path) return null;
  if (path.startsWith("http://") || path.startsWith("https://") || path.startsWith("asset://")) return path;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

function resolve(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

/** 从 ThemeDefinition 推导出完整的颜色集（补全可选字段 + 透明色） */
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

// ============================================================
// Pinia Store
// ============================================================

export const useThemeStore = defineStore("theme", () => {
  const { call } = useTauriCommand();

  // ---- 响应式状态 ----
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

  // 系统主题监听
  let mediaQuery: MediaQueryList | null = null;
  let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;

  // ---- 计算属性 ----
  const isDark = computed(() => resolvedTheme.value === "dark");

  // ============================================================
  // DOM 操作
  // ============================================================

  /** 将颜色注入 DOM */
  function applyThemeToDom(colors: ResolvedThemeColors, mode: ResolvedTheme) {
    if (typeof document === "undefined") return;

    const root = document.documentElement;

    // 设置 data-theme（控制阴影等非颜色变量）
    if (mode === "dark") {
      root.setAttribute("data-theme", "dark");
    } else {
      root.removeAttribute("data-theme");
    }

    // 注入 --theme-* 变量
    for (const [key, cssVar] of Object.entries(THEME_COLOR_MAP)) {
      root.style.setProperty(cssVar, colors[key as keyof ResolvedThemeColors]);
    }

    // 应用 customColors 覆盖（最高优先级）
    for (const [key, value] of Object.entries(customColorOverrides.value)) {
      const cssVar = THEME_COLOR_MAP[key as keyof ResolvedThemeColors];
      if (cssVar) {
        root.style.setProperty(cssVar, value);
      }
    }

    // 设置窗口效果 data 属性
    applyWindowEffectAttribute(currentWindowEffect.value);

    // 设置背景图片
    applyBackgroundImage(backgroundImage.value);

    resolvedTheme.value = mode;
  }

  /** 设置窗口效果 data 属性 */
  function applyWindowEffectAttribute(effect: string) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    if (effect === "mica" || effect === "acrylic") {
      root.setAttribute("data-window-effect", effect);
    } else {
      root.removeAttribute("data-window-effect");
    }
  }

  /** 设置背景图片（使用 convertFileSrc 转换本地路径） */
  function applyBackgroundImage(url: string | null) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      const assetUrl = toAssetUrl(url);
      bgLayer.style.backgroundImage = assetUrl ? `url(${assetUrl})` : "none";
    }
    // 同步 --has-bg-image CSS 变量，控制 bg-overlay 的显隐
    root.style.setProperty("--has-bg-image", url ? "1" : "0");
  }

  /** 应用背景遮罩透明度（图片上方的暗色遮罩） */
  function applyBgOpacity(opacity: number) {
    if (typeof document === "undefined") return;
    document.documentElement.style.setProperty("--effect-bg-opacity", String(opacity));
  }

  /** 应用图片模糊度 */
  function applyBgBlur(blur: number) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--effect-bg-blur", `${blur}px`);
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      if (blur > 0) {
        bgLayer.style.filter = `blur(${blur}px)`;
        // 模糊时放大边缘避免白边
        bgLayer.style.transform = "scale(1.05)";
      } else {
        bgLayer.style.filter = "none";
        bgLayer.style.transform = "none";
      }
    }
  }

  /** 应用整个应用窗口的透明度 */
  function applyAppOpacity(opacity: number) {
    if (typeof document === "undefined") return;
    document.documentElement.style.setProperty("--app-opacity", String(opacity));
  }

  /** 加载主题并应用 */
  function loadAndApplyTheme(themeId: string, mode: ResolvedTheme): boolean {
    // 1. 尝试从预装主题加载
    const builtin = getBuiltinTheme(themeId, mode);
    if (builtin) {
      const colors = deriveThemeColors(builtin);
      applyThemeToDom(colors, mode);
      return true;
    }

    // 2. 尝试从自定义主题加载
    const custom = availableThemes.value.find(t => t.id === themeId && t.mode === mode);
    if (custom) {
      const colors = deriveThemeColors(custom);
      applyThemeToDom(colors, mode);
      return true;
    }

    // 3. 回退到默认主题
    const fallback = getBuiltinTheme("default", mode);
    if (fallback) {
      const colors = deriveThemeColors(fallback);
      applyThemeToDom(colors, mode);
    }
    return false;
  }

  // ============================================================
  // 强调色
  // ============================================================

  /** 用指定颜色覆盖当前主题的强调色变量 */
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

  /** 根据当前 accentSource 应用强调色 */
  async function applyAccentSource() {
    if (currentAccentSource.value === "system") {
      try {
        const sysAccent = await call<string>("get_system_accent_color");
        accentColor.value = sysAccent;
        applyAccentOverride(sysAccent);
      } catch {
        // 读取失败，使用当前主题的强调色
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
      // accentSource = "theme"，使用主题自带强调色
      const theme = getBuiltinTheme(currentThemeId.value, resolvedTheme.value)
        ?? availableThemes.value.find(t => t.id === currentThemeId.value && t.mode === resolvedTheme.value);
      if (theme) accentColor.value = theme.colors.accent;
    }
  }

  // ============================================================
  // 系统主题监听
  // ============================================================

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

  // ============================================================
  // 初始化
  // ============================================================

  /** 初始化主题（应用启动时调用一次） */
  async function init() {
    const settingsStore = useSettingsStore();
    const appearance = settingsStore.config?.appearance;
    const mode = (appearance?.theme as ThemeMode) ?? "system";
    const resolved = resolve(mode);

    // 读取主题配置
    currentThemeId.value = appearance?.theme_id ?? "default";
    currentAccentSource.value = appearance?.accent_source ?? "system";

    // 读取背景设置
    backgroundImage.value = appearance?.background_image ?? null;
    const opacity = appearance?.bg_opacity ?? 1;
    applyBgOpacity(opacity);

    const blur = appearance?.bg_blur ?? 0;
    applyBgBlur(blur);
    bgBlur.value = blur;

    const appOpacityVal = appearance?.app_opacity ?? 0;
    applyAppOpacity(appOpacityVal);
    appOpacity.value = appOpacityVal;

    // 加载自定义主题列表
    await loadCustomThemes();

    // 应用主题
    loadAndApplyTheme(currentThemeId.value, resolved);

    // 应用背景图片
    applyBackgroundImage(backgroundImage.value);

    // 处理强调色
    await applyAccentSource();

    startSystemListener();
  }

  // ============================================================
  // Setter 方法
  // ============================================================

  /** 切换明暗模式 */
  async function setMode(mode: ThemeMode) {
    const settingsStore = useSettingsStore();
    const resolved = resolve(mode);
    loadAndApplyTheme(currentThemeId.value, resolved);
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme: mode });
  }

  /** 切换主题 */
  async function setTheme(id: string) {
    const settingsStore = useSettingsStore();
    currentThemeId.value = id;
    loadAndApplyTheme(id, resolvedTheme.value);
    // 切换到预装主题时自动设 accentSource 为 "theme"
    currentAccentSource.value = "theme";
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme_id: id, accent_source: "theme" });
  }

  /** 设置自定义强调色 */
  async function setAccentColor(hex: string) {
    const settingsStore = useSettingsStore();
    accentColor.value = hex;
    currentAccentSource.value = "custom";
    applyAccentOverride(hex);
    await settingsStore.patchAppearance({ accent_source: "custom", custom_accent_color: hex });
  }

  /** 切换强调色来源 */
  async function setAccentSource(src: "system" | "theme" | "custom") {
    const settingsStore = useSettingsStore();
    currentAccentSource.value = src;
    await applyAccentSource();
    const patch: Record<string, unknown> = { accent_source: src };
    if (src !== "custom") patch.custom_accent_color = null;
    await settingsStore.patchAppearance(patch);
  }

  /** 设置窗口效果（同时更新 Tauri 窗口和 CSS data 属性） */
  async function setWindowEffect(effect: string) {
    const settingsStore = useSettingsStore();
    currentWindowEffect.value = effect;
    applyWindowEffectAttribute(effect);

    // 同步 Tauri 窗口效果
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      let effects: string[];
      switch (effect) {
        case "mica": effects = ["Mica"]; break;
        case "acrylic": effects = ["Acrylic"]; break;
        case "none": effects = []; break;
        default: effects = ["Mica", "Acrylic"]; break;
      }
      await (win as any).set_effects({ effects });
    } catch (e) {
      console.warn("setWindowEffect failed", e);
    }

    await settingsStore.patchAppearance({ effect });
  }

  /** 设置背景图片 */
  async function setBackgroundImage(url: string | null) {
    const settingsStore = useSettingsStore();
    backgroundImage.value = url;
    applyBackgroundImage(url);
    await settingsStore.patchAppearance({ background_image: url });
  }

  /** 设置图片模糊度 */
  async function setBgBlur(blur: number) {
    const settingsStore = useSettingsStore();
    bgBlur.value = blur;
    applyBgBlur(blur);
    await settingsStore.patchAppearance({ bg_blur: blur });
  }

  /** 设置应用透明度 */
  async function setAppOpacity(opacity: number) {
    const settingsStore = useSettingsStore();
    appOpacity.value = opacity;
    applyAppOpacity(opacity);
    await settingsStore.patchAppearance({ app_opacity: opacity });
  }

  /** 设置背景透明度 */
  async function setBgOpacity(opacity: number) {
    const settingsStore = useSettingsStore();
    applyBgOpacity(opacity);
    await settingsStore.patchAppearance({ bg_opacity: opacity });
  }

  /** 设置自定义颜色覆盖（在预设主题基础上微调） */
  async function setCustomColorOverride(key: string, value: string | null) {
    if (value === null) {
      delete customColorOverrides.value[key];
    } else {
      customColorOverrides.value[key] = value;
    }
    // 立即应用覆盖
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

  /** 清除所有自定义颜色覆盖 */
  function clearCustomColorOverrides() {
    customColorOverrides.value = {};
    // 重新应用当前主题以恢复默认值
    loadAndApplyTheme(currentThemeId.value, resolvedTheme.value);
  }

  // ============================================================
  // 自定义主题管理
  // ============================================================

  /** 加载自定义主题列表 */
  async function loadCustomThemes() {
    try {
      const customs = await call<ThemeDefinition[]>("list_custom_themes");
      // 合并预装主题和自定义主题
      const builtins = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
      availableThemes.value = [...builtins, ...customs];
    } catch {
      availableThemes.value = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
    }
  }

  /** 导入主题（带 Zod 校验） */
  async function importTheme(json: unknown): Promise<{ success: boolean; error?: string }> {
    const result = ThemeDefinitionSchema.safeParse(json);
    if (!result.success) {
      const errors = result.error.issues.map(i => `${i.path.join(".")}: ${i.message}`).join("; ");
      return { success: false, error: errors };
    }

    let theme = result.data;

    // ID 冲突处理：预装主题加前缀
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

  /** 导出主题 */
  async function exportTheme(id: string): Promise<ThemeDefinition | null> {
    // 先从预装主题找
    const light = getBuiltinTheme(id, "light");
    const dark = getBuiltinTheme(id, "dark");
    if (light || dark) return light ?? dark!;

    // 再从自定义主题找
    return availableThemes.value.find(t => t.id === id) ?? null;
  }

  /** 删除自定义主题 */
  async function deleteTheme(id: string) {
    const theme = availableThemes.value.find(t => t.id === id);
    if (theme?.isBuiltIn) throw new Error("Cannot delete built-in theme");

    await call("delete_custom_theme", { theme_id: id });
    await loadCustomThemes();

    // 如果删除的是当前主题，回退到 default 并切换
    if (currentThemeId.value === id) {
      await setTheme("default");
    }
  }

  // ============================================================
  // 返回
  // ============================================================

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
    setBgOpacity,
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
