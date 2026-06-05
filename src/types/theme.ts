import { z } from "zod";

/** 颜色值校验：支持 #RRGGBB、#RRGGBBAA、rgba() */
export const ColorValue = z.string().refine(
  (val) => /^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(val) || /^rgba?\(/.test(val),
  "Invalid color value (expected #RRGGBB, #RRGGBBAA, or rgba())"
);

export const ThemeColorsSchema = z.object({
  accent: ColorValue,
  accentHover: ColorValue.optional(),
  accentPressed: ColorValue.optional(),
  accentSubtle: ColorValue.optional(),
  accentText: ColorValue.optional(),
  bgSolid: ColorValue,
  bgCard: ColorValue,
  bgHover: ColorValue,
  bgActive: ColorValue,
  bgSubtle: ColorValue,
  bgInset: ColorValue,
  textPrimary: ColorValue,
  textSecondary: ColorValue,
  textTertiary: ColorValue,
  textDisabled: ColorValue,
  textOnAccent: ColorValue,
  textOnDanger: ColorValue,
  danger: ColorValue,
  dangerHover: ColorValue.optional(),
  dangerPressed: ColorValue.optional(),
  dangerSubtle: ColorValue.optional(),
  warning: ColorValue,
  warningSubtle: ColorValue.optional(),
  success: ColorValue,
  successSubtle: ColorValue.optional(),
  border: ColorValue,
  borderStrong: ColorValue,
  borderAccent: ColorValue,
  divider: ColorValue,
  overlay: ColorValue,
  closeHover: ColorValue,
});

export const ThemeDefinitionSchema = z.object({
  id: z.string().min(1).max(64).regex(/^[a-z0-9_-]+$/),
  name: z.string().min(1).max(32),
  mode: z.enum(["light", "dark"]),
  isBuiltIn: z.boolean(),
  colors: ThemeColorsSchema,
});

export type ThemeColors = z.infer<typeof ThemeColorsSchema>;
export type ResolvedThemeColors = Required<ThemeColors>;
export type ThemeDefinition = z.infer<typeof ThemeDefinitionSchema>;

/** CSS 变量名到 ThemeColors 字段的映射 */
export const THEME_COLOR_MAP: Record<keyof ResolvedThemeColors, string> = {
  accent: "--theme-accent",
  accentHover: "--theme-accent-hover",
  accentPressed: "--theme-accent-pressed",
  accentSubtle: "--theme-accent-subtle",
  accentText: "--theme-accent-text",
  bgSolid: "--theme-bg-solid",
  bgCard: "--theme-bg-card",
  bgHover: "--theme-bg-hover",
  bgActive: "--theme-bg-active",
  bgSubtle: "--theme-bg-subtle",
  bgInset: "--theme-bg-inset",
  textPrimary: "--theme-text-primary",
  textSecondary: "--theme-text-secondary",
  textTertiary: "--theme-text-tertiary",
  textDisabled: "--theme-text-disabled",
  textOnAccent: "--theme-text-on-accent",
  textOnDanger: "--theme-text-on-danger",
  danger: "--theme-danger",
  dangerHover: "--theme-danger-hover",
  dangerPressed: "--theme-danger-pressed",
  dangerSubtle: "--theme-danger-subtle",
  warning: "--theme-warning",
  warningSubtle: "--theme-warning-subtle",
  success: "--theme-success",
  successSubtle: "--theme-success-subtle",
  border: "--theme-border",
  borderStrong: "--theme-border-strong",
  borderAccent: "--theme-border-accent",
  divider: "--theme-divider",
  overlay: "--theme-overlay",
  closeHover: "--theme-close-hover",
};
