import { defineStore } from "pinia";
import { ref } from "vue";

export const useUIStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const dragState = ref<"idle" | "dragging">("idle");

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  return { sidebarCollapsed, dragState, toggleSidebar };
});
