<template>
  <div class="titlebar">
    <div class="titlebar-left">
      <span class="titlebar-title" data-tauri-drag-region>Desk Manager</span>
    </div>
    <div class="titlebar-right">
      <button class="titlebar-btn" @click="onMinimize" title="最小化">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect y="5" width="12" height="1" fill="currentColor" />
        </svg>
      </button>
      <button class="titlebar-btn" @click="onToggleMaximize" title="最大化/还原">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1.2" fill="none" />
        </svg>
      </button>
      <button class="titlebar-btn close-btn" @click="onCloseClick" title="关闭">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path
            d="M1 1L11 11M11 1L1 11"
            stroke="currentColor"
            stroke-width="1.2"
          />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useWindowClose } from "@/composables/useWindowClose";

/**
 * TitleBar 上的 X 按钮只需要调用 useWindowClose.requestClose()，
 * 真正的关闭决策（弹窗/hide/quit）由 App.vue 中挂载的全局监听器处理。
 * 这样设计保证 X 按钮、Alt+F4、托盘右键等所有路径都走同一套逻辑。
 */
const windowClose = useWindowClose();

async function onMinimize() {
  try {
    await getCurrentWindow().minimize();
  } catch (e) {
    console.error("minimize failed", e);
  }
}

async function onToggleMaximize() {
  try {
    const win = getCurrentWindow();
    if (await win.isMaximized()) {
      await win.unmaximize();
    } else {
      await win.maximize();
    }
  } catch (e) {
    console.error("toggle maximize failed", e);
  }
}

async function onCloseClick() {
  await windowClose.requestClose();
}
</script>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding-left: 12px;
  background: var(--color-bg-solid);
  user-select: none;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.titlebar-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: default;
}

.titlebar-right {
  display: flex;
  height: 100%;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.titlebar-btn:hover {
  background: var(--color-bg-hover);
}

.close-btn:hover {
  background: var(--color-close-hover);
  color: var(--color-text-on-accent);
}
</style>
