import { defineStore } from "pinia";
import { ref, computed } from "vue";

export interface DragItemInfo {
  itemId: number;
  sourceCategoryId: number;
}

export const useUIStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const dragItem = ref<DragItemInfo | null>(null);
  const searchOverlayOpen = ref(false);

  const isDragging = computed(() => dragItem.value !== null);

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function startDrag(itemId: number, sourceCategoryId: number) {
    dragItem.value = { itemId, sourceCategoryId };
  }

  function endDrag() {
    dragItem.value = null;
  }

  function openSearchOverlay() {
    searchOverlayOpen.value = true;
  }

  function closeSearchOverlay() {
    searchOverlayOpen.value = false;
  }

  function toggleSearchOverlay() {
    searchOverlayOpen.value = !searchOverlayOpen.value;
  }

  return {
    sidebarCollapsed,
    dragItem,
    isDragging,
    searchOverlayOpen,
    toggleSidebar,
    startDrag,
    endDrag,
    openSearchOverlay,
    closeSearchOverlay,
    toggleSearchOverlay,
  };
});
