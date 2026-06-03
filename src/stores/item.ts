import { defineStore } from "pinia";
import { ref } from "vue";
import type { Item } from "@/types/item";
import { useTauriCommand } from "@/composables/useTauriCommand";

export const useItemStore = defineStore("item", () => {
  const { call } = useTauriCommand();
  const items = ref<Item[]>([]);
  const loading = ref(false);

  async function fetchItemsByCategory(categoryId: number) {
    loading.value = true;
    try {
      items.value = await call<Item[]>("get_items_by_category", { categoryId });
    } finally {
      loading.value = false;
    }
  }

  async function createItem(
    categoryId: number,
    name: string,
    itemType: string,
    path: string
  ) {
    await call("create_item", { categoryId, name, itemType, path });
  }

  async function launchItem(id: number) {
    await call("launch_item", { id });
  }

  async function deleteItem(id: number) {
    await call("delete_item", { id });
  }

  async function togglePinItem(id: number) {
    await call("toggle_pin_item", { id });
  }

  return {
    items,
    loading,
    fetchItemsByCategory,
    createItem,
    launchItem,
    deleteItem,
    togglePinItem,
  };
});
