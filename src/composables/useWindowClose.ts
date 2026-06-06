import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSettingsStore } from "@/stores/settings";
import { useToastStore } from "@/stores/toast";
import type { CloseBehavior } from "@/components/common/CloseConfirmDialog.vue";

/**
 * 窗口关闭行为统一处理
 *
 * 监听 Rust 端 "window-close-requested" 事件，根据 settings.close_behavior 决定行为：
 * - "ask" → 弹出 CloseConfirmDialog 让用户选
 * - "minimize_to_tray" → 直接 hide
 * - "quit" → 调 quit_app 命令
 *
 * confirmVisible 是模块级单例 ref，所有 useWindowClose() 调用共享同一个 ref，
 * 确保 TitleBar 设置 visible=true 后 App.vue 的 CloseConfirmDialog 也能看到。
 */

// --- 单例状态 ---
const confirmVisible = ref(false);
let unlisten: UnlistenFn | null = null;
let listening = false;

export function useWindowClose() {
  async function savePosition() {
    try {
      const win = getCurrentWindow();
      const pos = await win.outerPosition();
      const size = await win.innerSize();
      const scale = await win.scaleFactor();
      await invoke("save_window_position", {
        x: Math.round(pos.x / scale),
        y: Math.round(pos.y / scale),
        width: Math.round(size.width / scale),
        height: Math.round(size.height / scale),
      });
    } catch (e) {
      console.warn("savePosition failed", e);
    }
  }

  async function performClose(behavior: CloseBehavior) {
    if (behavior === "quit") {
      try {
        await invoke("quit_app");
      } catch (e) {
        console.error("quit_app failed", e);
        useToastStore().error("退出应用失败");
      }
    } else {
      await savePosition();
      try {
        await getCurrentWindow().hide();
      } catch (e) {
        console.error("hide failed", e);
        useToastStore().error("最小化到托盘失败");
      }
    }
  }

  async function applyBehavior(behavior: CloseBehavior) {
    if (behavior === "ask") {
      confirmVisible.value = true;
      return;
    }
    await performClose(behavior);
  }

  async function requestClose() {
    const settingsStore = useSettingsStore();
    const behavior = settingsStore.config?.close_behavior ?? "ask";
    await applyBehavior(behavior);
  }

  async function onConfirmDialog(data: {
    behavior: CloseBehavior;
    remember: boolean;
  }) {
    confirmVisible.value = false;
    if (data.remember) {
      const settingsStore = useSettingsStore();
      await settingsStore.patchCloseBehavior(data.behavior);
    }
    await performClose(data.behavior);
  }

  function onCancelDialog() {
    confirmVisible.value = false;
  }

  /**
   * 注册全局关闭事件监听（幂等，重复调用安全）
   */
  async function startListening() {
    if (listening) return;
    listening = true;
    unlisten = await listen("window-close-requested", async () => {
      await requestClose();
    });
  }

  function stopListening() {
    unlisten?.();
    unlisten = null;
    listening = false;
  }

  return {
    confirmVisible,
    requestClose,
    onConfirmDialog,
    onCancelDialog,
    performClose,
    startListening,
    stopListening,
  };
}
