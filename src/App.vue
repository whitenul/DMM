<template>
  <div class="app-layout">
    <TitleBar />
    <div class="app-body">
      <Sidebar />
      <main class="app-content">
        <router-view v-slot="{ Component, route }">
          <transition name="fade" mode="out-in">
            <component :is="Component" :key="route.fullPath" />
          </transition>
        </router-view>
      </main>
    </div>
    <SearchOverlay />
    <Toast />
    <CloseConfirmDialog
      :visible="closeConfirmVisible"
      @cancel="onCancelCloseDialog"
      @confirm="onConfirmCloseDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useCategoryStore } from "@/stores/category";
import { useSettingsStore } from "@/stores/settings";
import { useToastStore } from "@/stores/toast";
import { useUIStore } from "@/stores/ui";
import { useTheme } from "@/composables/useTheme";
import { useWindowClose } from "@/composables/useWindowClose";
import TitleBar from "@/components/layout/TitleBar.vue";
import Sidebar from "@/components/layout/Sidebar.vue";
import SearchOverlay from "@/components/search/SearchOverlay.vue";
import Toast from "@/components/common/Toast.vue";
import CloseConfirmDialog from "@/components/common/CloseConfirmDialog.vue";

const categoryStore = useCategoryStore();
const settingsStore = useSettingsStore();
const toast = useToastStore();
const uiStore = useUIStore();
const theme = useTheme();

// 解构 composable 返回值，保证模板中 ref 自动解包
const {
  confirmVisible: closeConfirmVisible,
  onConfirmDialog: onConfirmCloseDialog,
  onCancelDialog: onCancelCloseDialog,
  startListening,
  stopListening,
} = useWindowClose();

// ⚠️ 关键：必须在 setup 同步阶段调用，不能放在 onMounted 里
// 因为 listen() 是异步的，但注册动作本身不需要等 onMounted
startListening();

let unlistenFolderChanged: (() => void) | null = null;
let unlistenKeydown: ((e: KeyboardEvent) => void) | null = null;

onMounted(async () => {
  await settingsStore.loadSettings();
  theme.init();
  await categoryStore.fetchCategories();

  unlistenFolderChanged = await listen<number>("folder-changed", async (event) => {
    if (categoryStore.activeCategoryId === event.payload) {
      const { useItemStore } = await import("@/stores/item");
      useItemStore().fetchItemsByCategory(event.payload);
    }
  });

  try {
    const imported = await invoke<number>("auto_scan_on_start");
    if (imported > 0) {
      toast.info(`自动扫描发现 ${imported} 个新应用`);
      await categoryStore.fetchCategories();
    }
  } catch {
    // auto scan failure is non-critical
  }

  unlistenKeydown = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.shiftKey && e.code === "Space") {
      e.preventDefault();
      uiStore.toggleSearchOverlay();
    } else if (e.key === "Escape" && uiStore.searchOverlayOpen) {
      uiStore.closeSearchOverlay();
    }
  };
  window.addEventListener("keydown", unlistenKeydown);
});

onBeforeUnmount(() => {
  stopListening();
  unlistenFolderChanged?.();
  if (unlistenKeydown) {
    window.removeEventListener("keydown", unlistenKeydown);
    unlistenKeydown = null;
  }
});
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-bg-hover);
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.app-content {
  flex: 1;
  overflow: auto;
  position: relative;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateX(8px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}
</style>
