<template>
  <div class="titlebar" role="titlebar" @mousedown="onTitlebarMouseDown">
    <div class="titlebar-left">
      <span class="titlebar-title">Desk Manager</span>
    </div>
    <div class="titlebar-right">
      <button class="titlebar-btn" @click="onMinimize" title="最小化">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect y="5" width="12" height="1" fill="currentColor" />
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

const windowClose = useWindowClose();

function onTitlebarMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement;
  if (target.closest("button, a, input, select, textarea")) return;

  e.preventDefault();
  // 启动窗口拖拽
  getCurrentWindow().startDragging().catch((err) => {
    console.error("startDragging failed", err);
  });
}

async function onMinimize() {
  try {
    await getCurrentWindow().minimize();
  } catch (e) {
    console.error("minimize failed", e);
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
  background: transparent;
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
