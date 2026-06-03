import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSettingsStore } from "@/stores/settings";
import { useToastStore } from "@/stores/toast";
import type { CloseBehavior } from "@/components/common/CloseConfirmDialog.vue";

/**
 * 窗口关闭统一处理 composable
 *
 * 职责：
 * 1. 监听 Rust 端 emit 的 "window-close-requested" 事件
 * 2. 根据 settings.close_behavior 决定行为：
 *    - "ask" → 弹出 CloseConfirmDialog 让用户选
 *    - "minimize_to_tray" → 直接 hide
 *    - "quit" → 调 quit_app 命令
 * 3. 提供程序化触发关闭的 API（用于 TitleBar 上的 X 按钮等）
 */

let unlisten: UnlistenFn | null = null;

export function useWindowClose() {
  const settingsStore = useSettingsStore();
  const toast = useToastStore();

  const confirmVisible = ref(false);

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
        toast.error("退出应用失败");
      }
    } else {
      await savePosition();
      try {
        await getCurrentWindow().hide();
      } catch (e) {
        console.error("hide failed", e);
        toast.error("最小化到托盘失败");
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
    const behavior = settingsStore.config?.close_behavior ?? "ask";
    await applyBehavior(behavior);
  }

  async function onConfirmDialog(data: {
    behavior: CloseBehavior;
    remember: boolean;
  }) {
    confirmVisible.value = false;
    if (data.remember) {
      await settingsStore.patchCloseBehavior(data.behavior);
    }
    await performClose(data.behavior);
  }

  function onCancelDialog() {
    confirmVisible.value = false;
  }

  /**
   * 立即注册全局关闭事件监听。
   * 必须在 setup() 同步阶段调用（不能放在 onMounted 内部）。
   */
  async function startListening() {
    if (unlisten) return; // 已注册，跳过
    unlisten = await listen("window-close-requested", async () => {
      await requestClose();
    });
  }

  function stopListening() {
    unlisten?.();
    unlisten = null;
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
