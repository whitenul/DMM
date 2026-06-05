<template>
  <aside class="sidebar" :class="{ collapsed: uiStore.sidebarCollapsed }" role="navigation" aria-label="分类导航">
    <div class="sidebar-header">
      <span v-if="!uiStore.sidebarCollapsed" class="sidebar-label">分类</span>
      <button class="icon-btn" @click="uiStore.toggleSidebar" title="折叠侧边栏">
        <AppIcon
          name="chevron-left"
          :size="16"
          :style="{ transform: uiStore.sidebarCollapsed ? 'scaleX(-1)' : '' }"
        />
      </button>
    </div>

    <div ref="categoryListRef" class="category-list">
      <CategoryItem
        v-for="cat in categoryStore.categories"
        :key="cat.id"
        :data-id="cat.id"
        :category="cat"
        :active="cat.id === categoryStore.activeCategoryId"
        :collapsed="uiStore.sidebarCollapsed"
        @select="categoryStore.setActiveCategory(cat.id)"
        @contextmenu="onCategoryContextMenu($event, cat)"
      />
    </div>

    <button class="add-category-btn" @click="showAddDialog" title="添加分类">
      <AppIcon name="add" :size="14" />
      <span v-if="!uiStore.sidebarCollapsed">添加分类</span>
    </button>

    <button class="icon-btn" @click="openSearchOverlay" title="搜索 (Ctrl+Shift+Space)">
      <AppIcon name="search" :size="16" />
      <span v-if="!uiStore.sidebarCollapsed">搜索</span>
    </button>

    <router-link to="/settings" class="settings-btn" :class="{ 'is-active': isSettingsActive }" title="设置">
      <AppIcon name="settings" :size="16" />
      <span v-if="!uiStore.sidebarCollapsed">设置</span>
    </router-link>

    <ContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @close="contextMenu.visible = false"
      @select="onMenuAction"
    />

    <CategoryEditDialog
      :visible="editDialog.visible"
      :edit-data="editDialog.editData"
      @close="editDialog.visible = false"
      @confirm="onEditConfirm"
    />

    <ConfirmDialog
      :visible="confirmState.visible"
      :title="confirmState.title"
      :message="confirmState.message"
      :danger="true"
      confirm-text="删除"
      @confirm="confirmState.visible = false; confirmState.onConfirm?.()"
      @cancel="confirmState.visible = false"
    />
  </aside>
</template>

<script setup lang="ts">
import { reactive, computed, ref, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import { useRoute } from "vue-router";
import { useCategoryStore } from "@/stores/category";
import { useUIStore } from "@/stores/ui";
import CategoryItem from "@/components/category/CategoryItem.vue";
import ContextMenu from "@/components/common/ContextMenu.vue";
import type { MenuEntry } from "@/components/common/ContextMenu.vue";
import CategoryEditDialog from "@/components/category/CategoryEditDialog.vue";
import ConfirmDialog from "@/components/common/ConfirmDialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";
import type { Category } from "@/types/category";
import Sortable from "sortablejs";

const route = useRoute();
const categoryStore = useCategoryStore();
const uiStore = useUIStore();
const categoryListRef = ref<HTMLElement | null>(null);
let sortableInstance: Sortable | null = null;

const isSettingsActive = computed(() => route.path === "/settings" || route.path === "/settings/");

const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  targetCategory: null as Category | null,
});

const editDialog = reactive({
  visible: false,
  editData: null as { name: string; icon: string | null; folderPath: string | null } | null,
});

const confirmState = reactive({
  visible: false,
  title: "",
  message: "",
  onConfirm: null as (() => void) | null,
});

function showConfirm(title: string, message: string, onConfirm: () => void) {
  confirmState.title = title;
  confirmState.message = message;
  confirmState.onConfirm = onConfirm;
  confirmState.visible = true;
}

const contextMenuItems = computed<MenuEntry[]>(() => [
  { label: "编辑", icon: "edit", action: "edit" },
  { label: "关联文件夹", icon: "folder", action: "link-folder" },
  { divider: true },
  { label: "删除", icon: "delete", action: "delete" },
]);

function onCategoryContextMenu(e: MouseEvent, cat: Category) {
  contextMenu.x = e.clientX;
  contextMenu.y = e.clientY;
  contextMenu.targetCategory = cat;
  contextMenu.visible = true;
}

function onMenuAction(action: string) {
  const cat = contextMenu.targetCategory;
  if (!cat) return;

  switch (action) {
    case "edit":
      editDialog.editData = {
        name: cat.name,
        icon: cat.icon,
        folderPath: cat.folder_path,
      };
      editDialog.visible = true;
      break;
    case "link-folder":
      editDialog.editData = {
        name: cat.name,
        icon: cat.icon,
        folderPath: cat.folder_path,
      };
      editDialog.visible = true;
      break;
    case "delete":
      showConfirm("删除分类", `确定删除分类「${cat.name}」吗？该分类下的所有应用也会被删除。`, () => {
        categoryStore.deleteCategory(cat.id);
      });
      break;
  }
}

function showAddDialog() {
  editDialog.editData = null;
  editDialog.visible = true;
}

function openSearchOverlay() {
  uiStore.openSearchOverlay();
}

async function onEditConfirm(data: { name: string; icon: string; folderPath: string }) {
  const cat = contextMenu.targetCategory;
  if (cat) {
    await categoryStore.updateCategory(cat.id, {
      name: data.name,
      icon: data.icon,
      folderPath: data.folderPath || undefined,
    });
  } else {
    await categoryStore.createCategory(data.name, undefined, data.icon);
  }
  editDialog.visible = false;
}

function initSortable() {
  if (sortableInstance) {
    sortableInstance.destroy();
    sortableInstance = null;
  }
  if (!categoryListRef.value) return;
  sortableInstance = Sortable.create(categoryListRef.value, {
    animation: 150,
    ghostClass: "sortable-ghost",
    onEnd: async (evt) => {
      const { oldIndex, newIndex } = evt;
      if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return;

      const cats = [...categoryStore.categories];
      const [moved] = cats.splice(oldIndex, 1);
      cats.splice(newIndex, 0, moved);

      const orders = cats.map((cat, index) => ({
        id: cat.id,
        sort_order: index,
      }));

      categoryStore.categories = cats;
      await categoryStore.reorderCategories(orders);
    },
  });
}

watch(
  () => categoryStore.categories.length,
  () => nextTick(initSortable)
);

onMounted(() => nextTick(initSortable));
onBeforeUnmount(() => {
  sortableInstance?.destroy();
  sortableInstance = null;
});
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  width: 200px;
  min-width: 200px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--color-bg-hover);
  transition: width var(--transition-normal), min-width var(--transition-normal);
  overflow: hidden;
  position: relative;
  z-index: 2;
}

.sidebar.collapsed {
  width: 52px;
  min-width: 52px;
  /* 折叠时不绘制背景，让背景图片/底色透过来 */
  background: transparent;
  border-right-color: transparent;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  height: 40px;
  flex-shrink: 0;
}

.sidebar.collapsed .sidebar-header {
  justify-content: center;
  padding: 8px 0;
}

/* 折叠时 header 内的 chevron 按钮需要居中且无外边距 */
.sidebar.collapsed .sidebar-header .icon-btn {
  margin: 0;
}

.sidebar-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 折叠时 header 内的展开按钮（chevron-right）作为唯一的展开入口 */
.icon-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 4px 8px;
  padding: 6px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.sidebar.collapsed .icon-btn {
  justify-content: center;
  margin: 4px;
  padding: 6px;
}

.icon-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-accent);
}

/* 折叠态：给底部操作按钮（添加分类/搜索/设置）一个上方的提示性图标按钮，作为展开快捷方式 */
.category-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
  min-height: 0;
}

.sidebar.collapsed .category-list {
  padding: 4px;
}

/* 折叠时分类项居中 */
.sidebar.collapsed .category-item {
  justify-content: center;
  padding: 6px;
}

.add-category-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px;
  padding: 6px 12px;
  border: 1px dashed var(--color-bg-active);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  flex-shrink: 0;
}

.sidebar.collapsed .add-category-btn {
  justify-content: center;
  margin: 8px 4px;
  padding: 6px;
}

.add-category-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.settings-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 4px 8px 8px;
  padding: 6px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.sidebar.collapsed .settings-btn {
  justify-content: center;
  margin: 4px 4px 8px;
  padding: 6px;
}

.settings-btn:hover,
.settings-btn:deep(.router-link-active),
.settings-btn.is-active {
  background: var(--color-bg-hover);
  color: var(--color-accent);
}
</style>
