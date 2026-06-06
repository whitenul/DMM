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

// ============================================================
// Utilities
// ============================================================

/**
 * Converts a local file path to a Tauri asset URL.
 * http/https/asset:// URLs are returned as-is.
 */
function toAssetUrl(path: string | null): string | null {
  if (!path) return null;
  if (path.startsWith("http://") || path.startsWith("https://") || path.startsWith("asset://")) return path;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

/** Resolves a ThemeMode to a concrete ResolvedTheme. */
function resolve(mode: ThemeMode): ResolvedTheme {
  if (mode === "system") {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

/**
 * Derives the full color set from a ThemeDefinition.
 * Fills optional fields and resolves alpha colors.
 */
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

  /**
   * Injects theme colors into the DOM as CSS custom properties.
   * Also applies custom color overrides and background image.
   */
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

    // Apply custom color overrides (highest priority)
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

  /** Syncs the data-window-effect attribute on <html> for CSS selectors. */
  function applyWindowEffectAttribute(effect: string) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    if (effect === "mica" || effect === "acrylic") {
      root.setAttribute("data-window-effect", effect);
    } else {
      root.removeAttribute("data-window-effect");
    }
  }

  /**
   * Clears native DWM window effects (Mica/Acrylic).
   * All visual appearance is handled by CSS for smooth transitions.
   * Only the CSS data-window-effect attribute is set for CSS selectors.
   */
  async function applyNativeWindowEffect(_effect: string | null) {
    try {
      const win = getCurrentWindow();
      await win.clearEffects();
      // After clearEffects() WebView2 may fall back to a default background.
      // Reset to transparent to maintain window transparency.
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("reset_window_background");
      } catch {
        // Non-critical: background reset is a safety measure
      }
    } catch {
      // clearEffects may fail on some platforms; ignore silently
    }
  }

  /**
   * Sets the background image (converts local paths via convertFileSrc).
   * Only sets the --has-bg-image CSS custom property and the bg-layer
   * background-image. Backdrop-filter is handled entirely by CSS.
   */
  function applyBackgroundImage(url: string | null) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      const assetUrl = toAssetUrl(url);
      bgLayer.style.backgroundImage = assetUrl ? `url(${assetUrl})` : "none";
    }
    // Sync the --has-bg-image CSS custom property
    // CSS uses this to disable backdrop-filter when a background image is present
    root.style.setProperty("--has-bg-image", url ? "1" : "0");
  }

  /**
   * Applies background-image blur and scales the layer slightly
   * to prevent edge artifacts.
   */
  function applyBgBlur(blur: number) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--effect-bg-blur", `${blur}px`);
    const bgLayer = document.querySelector(".bg-layer") as HTMLElement | null;
    if (bgLayer) {
      if (blur > 0) {
        bgLayer.style.filter = `blur(${blur}px)`;
        bgLayer.style.transform = "scale(1.05)"; // prevent edge bleed
      } else {
        bgLayer.style.filter = "none";
        bgLayer.style.transform = "none";
      }
    }
  }

  /**
   * Applies application-wide window opacity.
   * Purely CSS-driven: only sets the --app-opacity custom property.
   * The glass-layer background, backdrop-filter blur, and bg-layer opacity
   * are all handled by CSS calc() formulas for smooth gradual transitions.
   * DWM effects (Mica/Acrylic) are kept on at all times — they naturally
   * show the desktop at high transparency and provide material at low transparency.
   */
  async function applyAppOpacity(opacity: number) {
    if (typeof document === "undefined") return;
    document.documentElement.style.setProperty("--app-opacity", String(opacity));
  }

  /**
   * Loads a theme by ID and applies it to the DOM.
   * Falls back to the default theme if the requested one is not found.
   */
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

  // ============================================================
  // 强调色
  // ============================================================

  /**
   * Overrides the current theme's accent color with a custom hex value.
   * Derives hover/pressed/subtle variants automatically.
   */
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

  /**
   * Applies the accent color based on the current accentSource:
   *   - "system": reads the OS accent color via Tauri command
   *   - "custom": uses the user-picked custom color
   *   - "theme": uses the built-in theme's accent color
   */
  async function applyAccentSource() {
    if (currentAccentSource.value === "system") {
      try {
        const sysAccent = await call<string>("get_system_accent_color");
        accentColor.value = sysAccent;
        applyAccentOverride(sysAccent);
      } catch {
        // Fallback to the current theme's accent color
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
      // accentSource = "theme"
      const theme = getBuiltinTheme(currentThemeId.value, resolvedTheme.value)
        ?? availableThemes.value.find(t => t.id === currentThemeId.value && t.mode === resolvedTheme.value);
      if (theme) accentColor.value = theme.colors.accent;
    }
  }

  // ============================================================
  // System theme listener
  // ============================================================

  /** Starts listening for OS-level dark/light mode changes. */
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

  /** Stops the OS theme change listener. */
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

  /**
   * Initializes the theme system. Called once at application startup.
   * Reads persisted settings and applies theme, background, opacity, etc.
   */
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

  // ============================================================
  // Setter 方法
  // ============================================================

  /** Switches between light / dark / system mode. */
  async function setMode(mode: ThemeMode) {
    const settingsStore = useSettingsStore();
    const resolved = resolve(mode);
    loadAndApplyTheme(currentThemeId.value, resolved);
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme: mode });
  }

  /** Switches to a different theme by ID. */
  async function setTheme(id: string) {
    const settingsStore = useSettingsStore();
    currentThemeId.value = id;
    loadAndApplyTheme(id, resolvedTheme.value);
    currentAccentSource.value = "theme";
    await applyAccentSource();
    await settingsStore.patchAppearance({ theme_id: id, accent_source: "theme" });
  }

  /** Sets a custom accent color and switches accentSource to "custom". */
  async function setAccentColor(hex: string) {
    const settingsStore = useSettingsStore();
    accentColor.value = hex;
    currentAccentSource.value = "custom";
    applyAccentOverride(hex);
    await settingsStore.patchAppearance({ accent_source: "custom", custom_accent_color: hex });
  }

  /** Changes the accent color source (system / theme / custom). */
  async function setAccentSource(src: "system" | "theme" | "custom") {
    const settingsStore = useSettingsStore();
    currentAccentSource.value = src;
    await applyAccentSource();
    const patch: Record<string, unknown> = { accent_source: src };
    if (src !== "custom") patch.custom_accent_color = null;
    await settingsStore.patchAppearance(patch);
  }

  /** Sets the window effect (CSS attribute + native DWM material). */
  async function setWindowEffect(effect: string) {
    const settingsStore = useSettingsStore();
    currentWindowEffect.value = effect;
    applyWindowEffectAttribute(effect);
    await applyNativeWindowEffect(effect);
    await settingsStore.patchAppearance({ effect });
  }

  /** Sets or clears the background image. */
  async function setBackgroundImage(url: string | null) {
    const settingsStore = useSettingsStore();
    backgroundImage.value = url;
    applyBackgroundImage(url);
    await settingsStore.patchAppearance({ background_image: url });
  }

  /** Sets the background-image blur amount (0-20 px). */
  async function setBgBlur(blur: number) {
    const settingsStore = useSettingsStore();
    bgBlur.value = blur;
    applyBgBlur(blur);
    await settingsStore.patchAppearance({ bg_blur: blur });
  }

  /**
   * Sets application opacity, clamped to the range 5% - 95%.
   * Prevents fully invisible or fully opaque states that break UX.
   */
  async function setAppOpacity(opacity: number) {
    const settingsStore = useSettingsStore();
    const clamped = Math.max(0.05, Math.min(0.95, opacity));
    appOpacity.value = clamped;
    applyAppOpacity(clamped);
    await settingsStore.patchAppearance({ app_opacity: clamped });
  }

  /**
   * Overrides a single theme color on top of the current preset.
   * Set value to null to remove the override.
   */
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

  /** Removes all custom color overrides and re-applies the current theme. */
  function clearCustomColorOverrides() {
    customColorOverrides.value = {};
    loadAndApplyTheme(currentThemeId.value, resolvedTheme.value);
  }

  // ============================================================
  // Custom theme management
  // ============================================================

  /** Loads the list of user-defined custom themes from the backend. */
  async function loadCustomThemes() {
    try {
      const customs = await call<ThemeDefinition[]>("list_custom_themes");
      const builtins = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
      availableThemes.value = [...builtins, ...customs];
    } catch {
      availableThemes.value = getBuiltinThemeIds().map(id => getBuiltinTheme(id, resolvedTheme.value)!).filter(Boolean);
    }
  }

  /**
   * Imports a theme from JSON with Zod validation.
   * Built-in theme IDs are prefixed with "imported_" to avoid collisions.
   */
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

  /** Exports a theme by ID (built-in or custom). */
  async function exportTheme(id: string): Promise<ThemeDefinition | null> {
    const light = getBuiltinTheme(id, "light");
    const dark = getBuiltinTheme(id, "dark");
    if (light || dark) return light ?? dark!;
    return availableThemes.value.find(t => t.id === id) ?? null;
  }

  /** Deletes a custom theme. Falls back to "default" if the active theme is deleted. */
  async function deleteTheme(id: string) {
    const theme = availableThemes.value.find(t => t.id === id);
    if (theme?.isBuiltIn) throw new Error("Cannot delete built-in theme");

    await call("delete_custom_theme", { theme_id: id });
    await loadCustomThemes();

    if (currentThemeId.value === id) {
      await setTheme("default");
    }
  }

  // ============================================================
  // Exports
  // ============================================================

  return {
    // State
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

    // Initialization
    init,

    // Mode & theme
    setMode,
    setTheme,

    // Accent color
    setAccentColor,
    setAccentSource,

    // Window effects & background
    setWindowEffect,
    setBackgroundImage,
    setBgBlur,
    setAppOpacity,

    // Custom color overrides
    setCustomColorOverride,
    clearCustomColorOverrides,

    // Custom theme management
    loadCustomThemes,
    importTheme,
    exportTheme,
    deleteTheme,

    // System listener
    startSystemListener,
    stopSystemListener,
  };
});
