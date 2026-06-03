import type { Component } from "vue";
import {
  AppWindow,
  FileText,
  Folder,
  Globe,
  Package,
  Pencil,
  Pin,
  Search,
  Loader2,
  Check,
  CheckCircle,
  X,
  Gamepad2,
  Wrench,
  Palette,
  BookOpen,
  Music,
  Briefcase,
  Home,
  Star,
  FolderOpen,
  Trash2,
  Copy,
  Plus,
  Settings,
  ChevronRight,
  ChevronDown,
  ChevronLeft,
  MoreHorizontal,
  XCircle,
  AlertCircle,
  Info,
  Play,
  type LucideProps,
} from "@lucide/vue";

export type IconName = string;

export type IconComponent = Component<LucideProps>;

/* ------------------------------------------------------------------ */
/*  业务图标 Registry                                                  */
/* ------------------------------------------------------------------ */

/** 启动项 / Item 类型图标 */
export const TYPE_ICON_MAP: Record<string, IconComponent> = {
  App: AppWindow,
  File: FileText,
  Folder: Folder,
  Web: Globe,
  Default: Package,
};

/** 分类可选图标（用于 CategoryEditDialog 的图标选择器） */
export interface CategoryIconOption {
  value: string;
  component: IconComponent;
}

export const CATEGORY_ICON_OPTIONS: CategoryIconOption[] = [
  { value: "Folder", component: Folder },
  { value: "Gamepad2", component: Gamepad2 },
  { value: "Wrench", component: Wrench },
  { value: "Palette", component: Palette },
  { value: "BookOpen", component: BookOpen },
  { value: "Music", component: Music },
  { value: "Briefcase", component: Briefcase },
  { value: "Globe", component: Globe },
  { value: "Home", component: Home },
  { value: "Star", component: Star },
];

/** 通用 UI 图标（按字符串 key 引用） */
export const UI_ICON_MAP: Record<string, IconComponent> = {
  // 操作
  edit: Pencil,
  pin: Pin,
  unpin: Pin,
  delete: Trash2,
  trash: Trash2,
  copy: Copy,
  add: Plus,
  play: Play,
  search: Search,
  settings: Settings,
  // 导航
  "chevron-right": ChevronRight,
  "chevron-down": ChevronDown,
  "chevron-left": ChevronLeft,
  "more-horizontal": MoreHorizontal,
  "folder-open": FolderOpen,
  // 状态 / 反馈
  success: CheckCircle,
  error: XCircle,
  warning: AlertCircle,
  info: Info,
  loading: Loader2,
  close: X,
  check: Check,
  // 占位 / 默认
  empty: FolderOpen,
  package: Package,
};

/* ------------------------------------------------------------------ */
/*  兼容旧 API（迁移期保留）                                            */
/* ------------------------------------------------------------------ */

/**
 * @deprecated 用 `<AppIcon name="App" />` 替代
 * 保留仅为兼容已存在的 <CategoryItem>、<SearchOverlay> 等组件
 */
export function getTypeIcon(type: string): IconComponent {
  return TYPE_ICON_MAP[type] || TYPE_ICON_MAP.Default;
}

/**
 * 把任意字符串 key 解析为图标组件
 *
 * 查找顺序：UI_ICON_MAP → TYPE_ICON_MAP → 名称动态匹配（CamelCase）→ Default
 */
export function resolveIcon(name: string | null | undefined): IconComponent {
  if (!name) return Package;
  if (UI_ICON_MAP[name]) return UI_ICON_MAP[name];
  if (TYPE_ICON_MAP[name]) return TYPE_ICON_MAP[name];
  if (CATEGORY_ICON_OPTIONS.find((o) => o.value === name)) {
    return CATEGORY_ICON_OPTIONS.find((o) => o.value === name)!.component;
  }
  return Package;
}
