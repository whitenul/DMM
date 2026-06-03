import { defineStore } from "pinia";
import { ref } from "vue";
import type { Item } from "@/types/item";
import { useTauriCommand } from "@/composables/useTauriCommand";
import { useToastStore } from "@/stores/toast";

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
    path: string,
    itemArgs?: string,
    workingDir?: string
  ) {
    await call("create_item", { categoryId, name, itemType, path, arguments: itemArgs || null, workingDir: workingDir || null });
  }

  async function updateItem(
    id: number,
    updates: {
      name?: string;
      itemType?: string;
      path?: string;
      arguments?: string;
      workingDir?: string;
    }
  ) {
    await call("update_item", {
      id,
      name: updates.name,
      itemType: updates.itemType,
      path: updates.path,
      arguments: updates.arguments,
      workingDir: updates.workingDir,
    });
  }

  async function moveItem(id: number, targetCategoryId: number) {
    await call("move_item", { id, targetCategoryId });
  }

  async function launchItem(id: number) {
    try {
      await call("launch_item", { id });
    } catch (e: unknown) {
      const toast = useToastStore();
      const msg =
        e && typeof e === "object" && "message" in e
          ? (e as { message: string }).message
          : "启动失败，请检查路径是否正确";
      toast.error(msg);
    }
  }

  async function deleteItem(id: number) {
    await call("delete_item", { id });
  }

  async function batchDeleteItems(ids: number[]) {
    await call("batch_delete_items", { ids });
  }

  async function togglePinItem(id: number) {
    await call("toggle_pin_item", { id });
  }

  async function reorderItems(orders: { id: number; sort_order: number }[]) {
    await call("reorder_items", { orders });
  }

  return {
    items,
    loading,
    fetchItemsByCategory,
    createItem,
    updateItem,
    moveItem,
    launchItem,
    deleteItem,
    batchDeleteItems,
    togglePinItem,
    reorderItems,
  };
});
