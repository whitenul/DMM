import { defineStore } from "pinia";
import { ref } from "vue";
import type { Category } from "@/types/category";
import { useTauriCommand } from "@/composables/useTauriCommand";

export const useCategoryStore = defineStore("category", () => {
  const { call } = useTauriCommand();
  const categories = ref<Category[]>([]);
  const activeCategoryId = ref<number | null>(null);
  const loading = ref(false);

  async function fetchCategories() {
    loading.value = true;
    try {
      categories.value = await call<Category[]>("get_categories");
      if (activeCategoryId.value === null && categories.value.length > 0) {
        activeCategoryId.value = categories.value[0].id;
      }
    } finally {
      loading.value = false;
    }
  }

  async function createCategory(name: string, parentId?: number, icon?: string) {
    await call("create_category", { name, parentId, icon });
    await fetchCategories();
  }

  async function updateCategory(
    id: number,
    updates: { name?: string; icon?: string; folderPath?: string }
  ) {
    await call("update_category", {
      id,
      name: updates.name,
      icon: updates.icon,
      folderPath: updates.folderPath,
    });
    await fetchCategories();
  }

  async function deleteCategory(id: number) {
    await call("delete_category", { id });
    await fetchCategories();
  }

  async function reorderCategories(orders: { id: number; sort_order: number }[]) {
    await call("reorder_categories", { orders });
  }

  async function linkFolder(categoryId: number, folderPath: string) {
    await call("link_folder", { categoryId, folderPath });
    await fetchCategories();
  }

  async function unlinkFolder(categoryId: number) {
    await call("unlink_folder", { categoryId });
    await fetchCategories();
  }

  function setActiveCategory(id: number) {
    activeCategoryId.value = id;
  }

  return {
    categories,
    activeCategoryId,
    loading,
    fetchCategories,
    createCategory,
    updateCategory,
    deleteCategory,
    reorderCategories,
    linkFolder,
    unlinkFolder,
    setActiveCategory,
  };
});
