import { computed, ref, watch } from "vue";
import { useSettingsStore } from "@/stores/settings";

export type Locale = "zh-CN" | "en";

const STORAGE_KEY = "dm-locale";

/** 翻译字典 —— 设置相关（按需扩展） */
const messages: Record<Locale, Record<string, string>> = {
  "zh-CN": {
    "settings.title": "设置",
    "settings.back": "返回",
    "settings.nav.appearance": "外观",
    "settings.nav.shortcut": "快捷键",
    "settings.nav.scan": "扫描",
    "settings.nav.about": "关于",
    "appearance.title": "外观",
    "appearance.mode": "明暗模式",
    "appearance.mode.desc": "选择应用的明暗主题模式",
    "appearance.mode.light": "浅色",
    "appearance.mode.dark": "深色",
    "appearance.mode.system": "跟随系统",
    "appearance.theme": "主题色",
    "appearance.theme.desc": "选择预设主题或自定义主题",
    "appearance.accent": "强调色来源",
    "appearance.accent.desc": "选择强调色的来源方式",
    "appearance.accent.system": "跟随系统",
    "appearance.accent.theme": "跟随主题",
    "appearance.accent.custom": "自定义",
    "appearance.accent.customLabel": "自定义强调色",
    "appearance.accent.customDesc": "选择自定义的强调色",
    "appearance.effect": "窗口效果",
    "appearance.effect.desc": "选择窗口的视觉效果",
    "appearance.effect.auto": "自动",
    "appearance.effect.mica": "Mica",
    "appearance.effect.acrylic": "亚克力",
    "appearance.effect.none": "无",
    "appearance.bg": "背景图片",
    "appearance.bg.desc": "设置窗口的背景图片",
    "appearance.bg.select": "选择图片",
    "appearance.bg.change": "更换图片",
    "appearance.bg.clear": "清除",
    "appearance.bg.blur": "图片模糊度",
    "appearance.bg.blur.desc": "对背景图片应用高斯模糊，制作毛玻璃效果",
    "appearance.appOpacity": "应用透明度",
    "appearance.appOpacity.desc": "值越高越透明，整个 app 一起变透明，能看到桌面/底层内容",
    "appearance.lang": "语言",
    "appearance.lang.desc": "选择应用界面语言",
    "appearance.lang.zh": "中文",
    "appearance.lang.en": "English",
    "shortcut.title": "快捷键",
    "shortcut.globalSearch": "全局搜索",
    "shortcut.globalSearch.desc": "触发全局搜索的快捷键",
    "shortcut.recorder.placeholder": "点击录制",
    "shortcut.recorder.recording": "按下快捷键...",
    "scan.title": "扫描",
    "scan.autoOnStart": "启动时自动扫描",
    "scan.autoOnStart.desc": "应用启动时自动扫描已安装的应用",
    "scan.startMenu": "扫描开始菜单",
    "scan.startMenu.desc": "扫描开始菜单中的快捷方式",
    "scan.uwp": "扫描 UWP 应用",
    "scan.uwp.desc": "扫描 Windows 应用商店中的 UWP 应用",
    "scan.targetCategory": "扫描目标分类",
    "scan.targetCategory.desc": "扫描结果保存到哪个分类",
    "scan.trigger": "触发方式",
    "scan.trigger.startMenu": "开始菜单",
    "scan.trigger.uwp": "UWP",
    "scan.trigger.all": "全部",
    "scan.runNow": "立即扫描",
    "scan.running": "正在扫描...",
    "about.title": "关于",
    "about.appName": "Desk Manager",
    "about.version": "版本",
    "about.autostart": "开机自启动",
    "about.autostart.desc": "开机时自动启动 Desk Manager",
  },
  "en": {
    "settings.title": "Settings",
    "settings.back": "Back",
    "settings.nav.appearance": "Appearance",
    "settings.nav.shortcut": "Shortcuts",
    "settings.nav.scan": "Scan",
    "settings.nav.about": "About",
    "appearance.title": "Appearance",
    "appearance.mode": "Theme Mode",
    "appearance.mode.desc": "Choose the light or dark mode",
    "appearance.mode.light": "Light",
    "appearance.mode.dark": "Dark",
    "appearance.mode.system": "System",
    "appearance.theme": "Theme",
    "appearance.theme.desc": "Pick a preset theme or customize your own",
    "appearance.accent": "Accent Color Source",
    "appearance.accent.desc": "How the accent color is determined",
    "appearance.accent.system": "Follow System",
    "appearance.accent.theme": "Follow Theme",
    "appearance.accent.custom": "Custom",
    "appearance.accent.customLabel": "Custom Accent",
    "appearance.accent.customDesc": "Pick a custom accent color",
    "appearance.effect": "Window Effect",
    "appearance.effect.desc": "Visual effect for the window",
    "appearance.effect.auto": "Auto",
    "appearance.effect.mica": "Mica",
    "appearance.effect.acrylic": "Acrylic",
    "appearance.effect.none": "None",
    "appearance.bg": "Background Image",
    "appearance.bg.desc": "Set a background image for the window",
    "appearance.bg.select": "Select Image",
    "appearance.bg.change": "Change Image",
    "appearance.bg.clear": "Clear",
    "appearance.bg.blur": "Image Blur",
    "appearance.bg.blur.desc": "Apply Gaussian blur to the background image for a frosted glass effect",
    "appearance.bg.mask": "Image Mask",
    "appearance.bg.mask.desc": "Dark overlay above the image, 0 = fully clear, 100% = fully covered",
    "appearance.appOpacity": "App Opacity",
    "appearance.appOpacity.desc": "Overall opacity of background + image. Lower = desktop shows through. App UI is not affected.",
    "appearance.lang": "Language",
    "appearance.lang.desc": "Choose UI language",
    "appearance.lang.zh": "中文",
    "appearance.lang.en": "English",
    "shortcut.title": "Shortcuts",
    "shortcut.globalSearch": "Global Search",
    "shortcut.globalSearch.desc": "Shortcut to open global search",
    "shortcut.recorder.placeholder": "Click to record",
    "shortcut.recorder.recording": "Press a shortcut...",
    "scan.title": "Scan",
    "scan.autoOnStart": "Auto Scan on Start",
    "scan.autoOnStart.desc": "Automatically scan installed apps on launch",
    "scan.startMenu": "Scan Start Menu",
    "scan.startMenu.desc": "Scan shortcuts in the Start Menu",
    "scan.uwp": "Scan UWP Apps",
    "scan.uwp.desc": "Scan UWP apps from the Windows Store",
    "scan.targetCategory": "Target Category",
    "scan.targetCategory.desc": "Where to save scanned results",
    "scan.trigger": "Trigger",
    "scan.trigger.startMenu": "Start Menu",
    "scan.trigger.uwp": "UWP",
    "scan.trigger.all": "All",
    "scan.runNow": "Scan Now",
    "scan.running": "Scanning...",
    "about.title": "About",
    "about.appName": "Desk Manager",
    "about.version": "Version",
    "about.autostart": "Launch on Startup",
    "about.autostart.desc": "Start Desk Manager automatically at login",
    "about.closeBehavior": "Close Behavior",
    "about.closeBehavior.desc": "What happens when the close button is clicked",
    "about.closeBehavior.ask": "Ask Every Time",
    "about.closeBehavior.minimize": "Minimize to Tray",
    "about.closeBehavior.quit": "Quit Immediately",
  },
};

const locale = ref<Locale>((localStorage.getItem(STORAGE_KEY) as Locale) || "zh-CN");

/** 同步 settings store 中的 language 字段 */
export function initI18n() {
  const settingsStore = useSettingsStore();
  // 初始从 settingsStore 同步
  const fromSettings = settingsStore.config?.appearance?.language as Locale | undefined;
  if (fromSettings && (fromSettings === "zh-CN" || fromSettings === "en")) {
    locale.value = fromSettings;
  }
  localStorage.setItem(STORAGE_KEY, locale.value);

  // 监听 settings 中的 language 变化，同步到 locale
  watch(
    () => settingsStore.config?.appearance?.language,
    (newLang) => {
      if (newLang === "zh-CN" || newLang === "en") {
        locale.value = newLang;
        localStorage.setItem(STORAGE_KEY, newLang);
        // 更新 document lang
        if (typeof document !== "undefined") {
          document.documentElement.lang = newLang;
        }
      }
    }
  );
}

export function useI18n() {
  const t = (key: string): string => {
    return messages[locale.value]?.[key] ?? messages["zh-CN"][key] ?? key;
  };
  const currentLocale = computed(() => locale.value);
  const setLocale = (l: Locale) => {
    locale.value = l;
    localStorage.setItem(STORAGE_KEY, l);
  };
  return { t, currentLocale, setLocale };
}
