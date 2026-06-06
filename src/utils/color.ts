/** HSL 颜色表示 */
export interface HSL {
  h: number; // 0-360
  s: number; // 0-100
  l: number; // 0-100
}

/** hex → HSL */
export function hexToHSL(hex: string): HSL {
  const h = hex.replace("#", "");
  const r = parseInt(h.substring(0, 2), 16) / 255;
  const g = parseInt(h.substring(2, 4), 16) / 255;
  const b = parseInt(h.substring(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;

  if (max === min) return { h: 0, s: 0, l: Math.round(l * 100) };

  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

  let hue: number;
  switch (max) {
    case r: hue = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
    case g: hue = ((b - r) / d + 2) / 6; break;
    default: hue = ((r - g) / d + 4) / 6; break;
  }

  return { h: Math.round(hue * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
}

/** HSL → hex */
export function hslToHex(hsl: HSL): string {
  const { h, s, l } = hsl;
  const a = s / 100 * Math.min(l / 100, 1 - l / 100);

  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const color = l / 100 - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
    return Math.round(255 * Math.max(0, Math.min(1, color)))
      .toString(16)
      .padStart(2, "0");
  };

  return `#${f(0)}${f(8)}${f(4)}`;
}

/** 调整亮度并返回 hex */
export function adjustLightness(hsl: HSL, delta: number): string {
  const newL = Math.max(0, Math.min(100, hsl.l + delta));
  return hslToHex({ ...hsl, l: newL });
}

/** 给颜色添加透明度，返回 rgba() 字符串 */
export function withAlpha(hex: string, alpha: number): string {
  const h = hex.replace("#", "");
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/** 计算相对亮度 (WCAG) */
export function relativeLuminance(hex: string): number {
  const h = hex.replace("#", "");
  const channels = [0, 2, 4].map(i => parseInt(h.substring(i, i + 2), 16) / 255);
  const [r, g, b] = channels.map(c =>
    c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
  );
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG 对比度 */
export function contrastRatio(hex1: string, hex2: string): number {
  const l1 = relativeLuminance(hex1);
  const l2 = relativeLuminance(hex2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/** 提亮到最低亮度 */
export function lightenToMinLuminance(hex: string, minLuminance: number): string {
  let hsl = hexToHSL(hex);
  while (relativeLuminance(hslToHex(hsl)) < minLuminance && hsl.l < 100) {
    hsl = { ...hsl, l: hsl.l + 2 };
  }
  return hslToHex(hsl);
}

/** 加深到最高亮度 */
export function darkenToMaxLuminance(hex: string, maxLuminance: number): string {
  let hsl = hexToHSL(hex);
  while (relativeLuminance(hslToHex(hsl)) > maxLuminance && hsl.l > 0) {
    hsl = { ...hsl, l: hsl.l - 2 };
  }
  return hslToHex(hsl);
}

/** 透明色推导映射表 */
const ALPHA_MAP: Record<string, { light: number; dark: number }> = {
  bgHover:       { light: 0.04, dark: 0.06 },
  bgActive:      { light: 0.06, dark: 0.08 },
  textPrimary:   { light: 0.9,  dark: 0.9 },
  textSecondary: { light: 0.6,  dark: 0.6 },
  textTertiary:  { light: 0.4,  dark: 0.4 },
  textDisabled:  { light: 0.25, dark: 0.25 },
  border:        { light: 0.06, dark: 0.06 },
  borderStrong:  { light: 0.12, dark: 0.12 },
  divider:       { light: 0.06, dark: 0.06 },
  overlay:       { light: 0.3,  dark: 0.5 },
};

/** 解析透明色 */
export function resolveAlphaColor(value: string, mode: "light" | "dark", varType: string): string {
  if (value.startsWith("rgba") || value.startsWith("rgb")) return value;
  const alpha = ALPHA_MAP[varType]?.[mode] ?? 1;
  return withAlpha(value, alpha);
}

/** 从单一强调色推导完整色系 */
export function deriveAccentColors(accent: string, mode: "light" | "dark") {
  const hsl = hexToHSL(accent);
  return {
    accentHover: adjustLightness(hsl, mode === "dark" ? +10 : -10),
    accentPressed: adjustLightness(hsl, mode === "dark" ? +20 : -20),
    accentSubtle: withAlpha(accent, mode === "dark" ? 0.08 : 0.12),
    accentText: mode === "dark"
      ? lightenToMinLuminance(accent, 0.6)
      : darkenToMaxLuminance(accent, 0.4),
  };
}
