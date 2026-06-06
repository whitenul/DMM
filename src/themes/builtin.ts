import type { ThemeDefinition } from "@/types/theme";

// --- 浅色模式通用配色 ---
const lightBase = {
  bgSolid: "#f3f3f3",
  bgCard: "#ffffff",
  bgHover: "#000000",
  bgActive: "#000000",
  bgSubtle: "#fafafa",
  bgInset: "#ebebeb",
  textPrimary: "#000000",
  textSecondary: "#000000",
  textTertiary: "#000000",
  textDisabled: "#000000",
  textOnAccent: "#ffffff",
  textOnDanger: "#ffffff",
  danger: "#e81123",
  dangerHover: "#c42b1c",
  dangerPressed: "#9b0000",
  dangerSubtle: "#ffcccc",
  warning: "#ffb900",
  warningSubtle: "#fff3bf",
  success: "#00cc6a",
  successSubtle: "#ccf7e0",
  border: "#000000",
  borderStrong: "#000000",
  divider: "#000000",
  overlay: "#000000",
  closeHover: "#c42b1c",
} as const;

// --- 深色模式通用配色 ---
const darkBase = {
  bgSolid: "#1f1f1f",
  bgCard: "#2c2c2c",
  bgHover: "#ffffff",
  bgActive: "#ffffff",
  bgSubtle: "#161616",
  bgInset: "#0e0e0e",
  textPrimary: "#ffffff",
  textSecondary: "#ffffff",
  textTertiary: "#ffffff",
  textDisabled: "#ffffff",
  textOnAccent: "#ffffff",
  textOnDanger: "#ffffff",
  danger: "#ff6b6b",
  dangerHover: "#e81123",
  dangerPressed: "#c42b1c",
  dangerSubtle: "rgba(255, 107, 107, 0.12)",
  warning: "#ffb900",
  warningSubtle: "rgba(255, 185, 0, 0.12)",
  success: "#00cc6a",
  successSubtle: "rgba(0, 204, 106, 0.12)",
  border: "#ffffff",
  borderStrong: "#ffffff",
  divider: "#ffffff",
  overlay: "#000000",
  closeHover: "#c42b1c",
} as const;

// --- 强调色定义 ---
const accents = {
  default: {
    light: {
      accent: "#0078d4",
      accentHover: "#003d7a",
      accentPressed: "#002a4e",
      accentSubtle: "#e6f2ff",
      accentText: "#005fb8",
      borderAccent: "#005fb8",
    },
    dark: {
      accent: "#60cdff",
      accentHover: "#4db8e8",
      accentPressed: "#2b88d8",
      accentSubtle: "rgba(96, 205, 255, 0.08)",
      accentText: "#60cdff",
      borderAccent: "#60cdff",
    },
  },
  forest: {
    light: {
      accent: "#00cc6a",
      accentHover: "#009900",
      accentPressed: "#007700",
      accentSubtle: "#ccf7e0",
      accentText: "#005500",
      borderAccent: "#00cc6a",
    },
    dark: {
      accent: "#33e08a",
      accentHover: "#00cc6a",
      accentPressed: "#009900",
      accentSubtle: "rgba(0, 204, 106, 0.12)",
      accentText: "#33e08a",
      borderAccent: "#33e08a",
    },
  },
  sunset: {
    light: {
      accent: "#e8710a",
      accentHover: "#b37d00",
      accentPressed: "#8a6500",
      accentSubtle: "#fff3bf",
      accentText: "#6e4e00",
      borderAccent: "#e8710a",
    },
    dark: {
      accent: "#ffd335",
      accentHover: "#ffb900",
      accentPressed: "#b37d00",
      accentSubtle: "rgba(255, 185, 0, 0.12)",
      accentText: "#ffd335",
      borderAccent: "#ffd335",
    },
  },
  lavender: {
    light: {
      accent: "#8764b8",
      accentHover: "#6b3fa0",
      accentPressed: "#502d88",
      accentSubtle: "#f0e6ff",
      accentText: "#6b3fa0",
      borderAccent: "#8764b8",
    },
    dark: {
      accent: "#b898e0",
      accentHover: "#9b6fd4",
      accentPressed: "#8764b8",
      accentSubtle: "rgba(135, 100, 184, 0.12)",
      accentText: "#b898e0",
      borderAccent: "#b898e0",
    },
  },
} as const;

// --- 预装主题 ---
function makeTheme(
  id: string,
  name: string,
  mode: "light" | "dark",
  accentColors: Record<string, string>,
  base: Record<string, string>,
): ThemeDefinition {
  return {
    id,
    name,
    mode,
    isBuiltIn: true,
    colors: {
      ...accentColors,
      ...base,
    } as ThemeDefinition["colors"],
  };
}

export const BUILTIN_THEMES: ThemeDefinition[] = [
  // default
  makeTheme("default", "默认蓝", "light", accents.default.light, lightBase),
  makeTheme("default", "默认蓝", "dark", accents.default.dark, darkBase),
  // forest
  makeTheme("forest", "森林绿", "light", accents.forest.light, lightBase),
  makeTheme("forest", "森林绿", "dark", accents.forest.dark, darkBase),
  // sunset
  makeTheme("sunset", "暖橙", "light", accents.sunset.light, lightBase),
  makeTheme("sunset", "暖橙", "dark", accents.sunset.dark, darkBase),
  // lavender
  makeTheme("lavender", "薰衣草", "light", accents.lavender.light, lightBase),
  makeTheme("lavender", "薰衣草", "dark", accents.lavender.dark, darkBase),
];

/** 根据 themeId + mode 查找预装主题 */
export function getBuiltinTheme(themeId: string, mode: "light" | "dark"): ThemeDefinition | undefined {
  return BUILTIN_THEMES.find(t => t.id === themeId && t.mode === mode);
}

/** 获取所有不重复的预装主题 ID */
export function getBuiltinThemeIds(): string[] {
  return [...new Set(BUILTIN_THEMES.map(t => t.id))];
}
