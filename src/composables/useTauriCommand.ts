import { invoke } from "@tauri-apps/api/core";

interface AppError {
  code: string;
  message: string;
}

const PLUGIN_COMMAND_MAP: Record<string, string> = {
  get_categories: "plugin:desk-category|get_categories",
  create_category: "plugin:desk-category|create_category",
  update_category: "plugin:desk-category|update_category",
  delete_category: "plugin:desk-category|delete_category",
  reorder_categories: "plugin:desk-category|reorder_categories",
  link_folder: "plugin:desk-category|link_folder",
  unlink_folder: "plugin:desk-category|unlink_folder",

  get_items_by_category: "plugin:desk-item|get_items_by_category",
  create_item: "plugin:desk-item|create_item",
  update_item: "plugin:desk-item|update_item",
  launch_item: "plugin:desk-item|launch_item",
  move_item: "plugin:desk-item|move_item",
  reorder_items: "plugin:desk-item|reorder_items",
  delete_item: "plugin:desk-item|delete_item",
  toggle_pin_item: "plugin:desk-item|toggle_pin_item",
  batch_delete_items: "plugin:desk-item|batch_delete_items",

  search_items: "plugin:desk-search|search_items",

  scan_start_menu: "plugin:desk-scan|scan_start_menu",
  scan_uwp_apps: "plugin:desk-scan|scan_uwp_apps",
  scan_folder: "plugin:desk-scan|scan_folder",
  import_scanned_apps: "plugin:desk-scan|import_scanned_apps",
  auto_scan_on_start: "plugin:desk-scan|auto_scan_on_start",

  extract_icon_for_item: "plugin:desk-icon|extract_icon_for_item",
  get_item_icon_base64: "plugin:desk-icon|get_item_icon_base64",

  load_settings: "plugin:desk-settings|load_settings",
  update_settings: "plugin:desk-settings|update_settings",
  save_window_position: "plugin:desk-settings|save_window_position",
  get_system_accent_color: "plugin:desk-settings|get_system_accent_color",
  list_custom_themes: "plugin:desk-settings|list_custom_themes",
  save_custom_theme: "plugin:desk-settings|save_custom_theme",
  delete_custom_theme: "plugin:desk-settings|delete_custom_theme",

  fetch_web_meta: "plugin:desk-web|fetch_web_meta",
};

function resolveCommand(command: string): string {
  return PLUGIN_COMMAND_MAP[command] ?? command;
}

// Icon cache — avoids repeated IPC calls for the same item
const iconCache = new Map<number, string>();

export function useIconCache() {
  const { call } = useTauriCommand();

  async function getIcon(itemId: number): Promise<string | null> {
    // Check cache first
    if (iconCache.has(itemId)) {
      return iconCache.get(itemId)!;
    }

    try {
      const base64 = await call<string | null>("get_item_icon_base64", { itemId });
      if (base64) {
        iconCache.set(itemId, base64);
      }
      return base64;
    } catch {
      return null;
    }
  }

  function invalidateIcon(itemId: number) {
    iconCache.delete(itemId);
  }

  function clearIconCache() {
    iconCache.clear();
  }

  return { getIcon, invalidateIcon, clearIconCache };
}

export function useTauriCommand() {
  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await invoke<T>(resolveCommand(command), args);
    } catch (error) {
      const appError = error as AppError;
      console.error(`[Tauri Command Error] ${command}:`, appError);
      throw appError;
    }
  }

  return { call };
}
