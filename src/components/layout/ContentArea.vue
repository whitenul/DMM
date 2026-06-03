<template>
  <main class="content-area">
    <div v-if="categoryStore.activeCategoryId === null" class="empty-state">
      <AppIcon name="chevron-left" :size="48" class="empty-icon" />
      <p class="empty-text">请选择一个分类</p>
    </div>

    <div v-else-if="!itemStore.items.length && !itemStore.loading" class="empty-state">
      <AppIcon name="folder-open" :size="48" class="empty-icon" />
      <p class="empty-text">当前分类下没有应用</p>
      <p class="empty-hint">点击下方按钮添加应用</p>
    </div>

    <template v-else>
      <div v-if="selectMode" class="select-toolbar">
        <span class="select-count">已选 {{ selectedIds.size }} 项</span>
        <button class="toolbar-btn" @click="selectAll">全选</button>
        <button class="toolbar-btn danger" @click="batchDelete" :disabled="!selectedIds.size">删除</button>
        <button class="toolbar-btn" @click="exitSelectMode">取消</button>
      </div>

      <div ref="itemGridRef" class="item-grid" :class="{ 'select-mode': selectMode }">
        <ItemCard
          v-for="item in itemStore.items"
          :key="item.id"
          :data-id="item.id"
          :item="item"
          :select-mode="selectMode"
          :selected="selectedIds.has(item.id)"
          @launch="itemStore.launchItem(item.id)"
          @contextmenu="onItemContextMenu($event, item)"
          @toggle-select="toggleSelect"
        />
      </div>
    </template>

    <div v-if="!selectMode" class="fab-group">
      <button class="add-item-fab" @click="enterSelectMode" title="批量管理">
        <svg width="18" height="18" viewBox="0 0 18 18">
          <rect x="2" y="3" width="14" height="2" rx="1" fill="currentColor" />
          <rect x="2" y="8" width="14" height="2" rx="1" fill="currentColor" />
          <rect x="2" y="13" width="14" height="2" rx="1" fill="currentColor" />
        </svg>
      </button>
      <button class="add-item-fab" @click="onScanClick" title="扫描导入">
        <AppIcon name="search" :size="18" />
      </button>
      <button class="add-item-fab primary" @click="onAddClick" title="手动添加">
        <AppIcon name="add" :size="20" />
      </button>
    </div>

    <ContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @close="contextMenu.visible = false"
      @select="onMenuAction"
    />

    <ItemEditDialog
      :visible="editDialog.visible"
      :category-id="categoryStore.activeCategoryId"
      :edit-data="editDialog.editData"
      @close="editDialog.visible = false"
      @confirm="onEditConfirm"
    />

    <ScanDialog
      :visible="scanDialog.visible"
      :category-id="categoryStore.activeCategoryId"
      @close="scanDialog.visible = false"
      @imported="onScanImported"
    />

    <ConfirmDialog
      :visible="confirmState.visible"
      :title="confirmState.title"
      :message="confirmState.message"
      :danger="confirmState.danger"
      confirm-text="删除"
      @confirm="confirmState.visible = false; confirmState.onConfirm?.()"
      @cancel="confirmState.visible = false"
    />
  </main>
</template>

<script setup lang="ts">
import { reactive, computed, ref, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import { useItemStore } from "@/stores/item";
import { useCategoryStore } from "@/stores/category";
import { useUIStore } from "@/stores/ui";
import { useToastStore } from "@/stores/toast";
import ItemCard from "@/components/item/ItemCard.vue";
import ContextMenu from "@/components/common/ContextMenu.vue";
import type { MenuEntry } from "@/components/common/ContextMenu.vue";
import ItemEditDialog from "@/components/item/ItemEditDialog.vue";
import ScanDialog from "@/components/item/ScanDialog.vue";
import ConfirmDialog from "@/components/common/ConfirmDialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";
import type { Item } from "@/types/item";
import Sortable from "sortablejs";

const itemStore = useItemStore();
const categoryStore = useCategoryStore();
const uiStore = useUIStore();
const toast = useToastStore();
const itemGridRef = ref<HTMLElement | null>(null);
let sortableInstance: Sortable | null = null;

const selectMode = ref(false);
const selectedIds = ref<Set<number>>(new Set());

const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  targetItem: null as Item | null,
});

const editDialog = reactive({
  visible: false,
  editData: null as {
    name: string;
    itemType: string;
    path: string;
    arguments: string | null;
    workingDir: string | null;
  } | null,
});

const scanDialog = reactive({
  visible: false,
});

const confirmState = reactive({
  visible: false,
  title: "",
  message: "",
  danger: false,
  onConfirm: null as (() => void) | null,
});

function showConfirm(title: string, message: string, onConfirm: () => void, danger = true) {
  confirmState.title = title;
  confirmState.message = message;
  confirmState.danger = danger;
  confirmState.onConfirm = onConfirm;
  confirmState.visible = true;
}

const contextMenuItems = computed<MenuEntry[]>(() => [
  { label: "打开", icon: "play", action: "launch" },
  { label: "打开文件位置", icon: "folder", action: "open-location" },
  { divider: true },
  { label: "编辑", icon: "edit", action: "edit" },
  { label: contextMenu.targetItem?.is_pinned ? "取消固定" : "固定到顶部", icon: "pin", action: "pin" },
  { divider: true },
  { label: "复制路径", icon: "copy", action: "copy-path" },
  { divider: true },
  { label: "删除", icon: "delete", action: "delete" },
]);

function onItemContextMenu(e: MouseEvent, item: Item) {
  contextMenu.x = e.clientX;
  contextMenu.y = e.clientY;
  contextMenu.targetItem = item;
  contextMenu.visible = true;
}

async function onMenuAction(action: string) {
  const item = contextMenu.targetItem;
  if (!item) return;

  switch (action) {
    case "launch":
      await itemStore.launchItem(item.id);
      break;
    case "open-location": {
      // 使用 explorer /select 打开文件所在目录并选中
      const path = item.path;
      if (path) {
        try {
          await import("@tauri-apps/plugin-shell").then((m) => m.open(`explorer /select,"${path}"`));
        } catch {
          toast.error("无法打开文件位置");
        }
      }
      break;
    }
    case "edit":
      editDialog.editData = {
        name: item.name,
        itemType: item.item_type,
        path: item.path,
        arguments: item.arguments,
        workingDir: item.working_dir,
      };
      editDialog.visible = true;
      break;
    case "pin":
      await itemStore.togglePinItem(item.id);
      if (categoryStore.activeCategoryId !== null) {
        await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
      }
      break;
    case "copy-path": {
      const path = item.path;
      if (path) {
        try {
          await navigator.clipboard.writeText(path);
          toast.success("路径已复制到剪贴板");
        } catch {
          toast.error("复制路径失败");
        }
      }
      break;
    }
    case "delete":
      showConfirm("删除应用", `确定删除「${item.name}」吗？`, async () => {
        await itemStore.deleteItem(item.id);
        if (categoryStore.activeCategoryId !== null) {
          await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
        }
      });
      break;
  }
}

function showAddDialog() {
  editDialog.editData = null;
  editDialog.visible = true;
}

function onScanClick() {
  if (categoryStore.activeCategoryId === null) {
    toast.warning("请先选择或创建一个分类");
    return;
  }
  scanDialog.visible = true;
}

function onAddClick() {
  if (categoryStore.activeCategoryId === null) {
    toast.warning("请先选择或创建一个分类");
    return;
  }
  showAddDialog();
}

async function onEditConfirm(data: {
  categoryId: number;
  name: string;
  itemType: string;
  path: string;
  arguments: string;
  workingDir: string;
}) {
  const item = contextMenu.targetItem;
  if (item) {
    await itemStore.updateItem(item.id, {
      name: data.name,
      itemType: data.itemType,
      path: data.path,
      arguments: data.arguments,
      workingDir: data.workingDir,
    });
  } else {
    await itemStore.createItem(
      data.categoryId,
      data.name,
      data.itemType,
      data.path,
      data.arguments || undefined,
      data.workingDir || undefined
    );
  }
  editDialog.visible = false;
  if (categoryStore.activeCategoryId !== null) {
    await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
  }
}

async function onScanImported() {
  if (categoryStore.activeCategoryId !== null) {
    await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
  }
}

function enterSelectMode() {
  selectMode.value = true;
  selectedIds.value = new Set();
  destroySortable();
}

function exitSelectMode() {
  selectMode.value = false;
  selectedIds.value = new Set();
  nextTick(initSortable);
}

function toggleSelect(id: number) {
  const newSet = new Set(selectedIds.value);
  if (newSet.has(id)) {
    newSet.delete(id);
  } else {
    newSet.add(id);
  }
  selectedIds.value = newSet;
}

function selectAll() {
  const newSet = new Set(itemStore.items.map((i) => i.id));
  selectedIds.value = newSet;
}

async function batchDelete() {
  const count = selectedIds.value.size;
  if (count === 0) return;
  showConfirm("批量删除", `确定删除选中的 ${count} 个应用吗？`, async () => {
    try {
      await itemStore.batchDeleteItems(Array.from(selectedIds.value));
      toast.success(`已删除 ${count} 个应用`);
      exitSelectMode();
      if (categoryStore.activeCategoryId !== null) {
        await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
      }
    } catch {
      toast.error("批量删除失败");
    }
  });
}

function initSortable() {
  if (sortableInstance) {
    sortableInstance.destroy();
    sortableInstance = null;
  }
  if (!itemGridRef.value || selectMode.value) return;
  sortableInstance = Sortable.create(itemGridRef.value, {
    animation: 150,
    ghostClass: "sortable-ghost",
    delay: 200,
    delayOnTouchOnly: false,
    onStart: (evt) => {
      const id = evt.item.getAttribute("data-id");
      if (id && categoryStore.activeCategoryId !== null) {
        uiStore.startDrag(Number(id), categoryStore.activeCategoryId);
      }
    },
    onEnd: async (evt) => {
      uiStore.endDrag();
      const { oldIndex, newIndex } = evt;
      if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return;

      const items = [...itemStore.items];
      const [moved] = items.splice(oldIndex, 1);
      items.splice(newIndex, 0, moved);

      const orders = items.map((item, index) => ({
        id: item.id,
        sort_order: index,
      }));

      itemStore.items = items;
      await itemStore.reorderItems(orders);
    },
  });
}

function destroySortable() {
  sortableInstance?.destroy();
  sortableInstance = null;
}

watch(
  () => itemStore.items.length,
  () => {
    if (!selectMode.value) nextTick(initSortable);
  }
);

watch(
  () => categoryStore.activeCategoryId,
  () => {
    exitSelectMode();
  }
);

onMounted(() => nextTick(initSortable));
onBeforeUnmount(() => {
  destroySortable();
});
</script>

<style scoped>
.content-area {
  flex: 1;
  padding: var(--spacing-lg);
  padding-bottom: 80px;
  overflow-y: auto;
  position: relative;
  background: var(--color-bg-card);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
}

.empty-icon {
  opacity: 0.5;
  color: var(--color-text-secondary);
}

.empty-text {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
}

.empty-hint {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  opacity: 0.7;
}

.select-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  margin-bottom: 12px;
  background: var(--color-bg-solid);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.select-count {
  font-size: var(--font-size-md);
  color: var(--color-text-primary);
  font-weight: 500;
  flex: 1;
}

.toolbar-btn {
  padding: 4px 14px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-bg-active);
  background: var(--color-bg-card);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.toolbar-btn:hover {
  background: var(--color-bg-hover);
}

.toolbar-btn.danger {
  color: var(--color-danger);
  border-color: var(--color-danger);
}

.toolbar-btn.danger:hover {
  background: var(--color-danger);
  color: var(--color-text-on-accent);
}

.toolbar-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.item-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: var(--spacing-md);
}

.item-grid.select-mode {
  user-select: none;
}

.fab-group {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.add-item-fab {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: var(--color-bg-solid);
  color: var(--color-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-icon);
  transition: all var(--transition-fast);
}

.add-item-fab:hover {
  transform: scale(1.05);
  background: var(--color-bg-hover);
}

.add-item-fab.primary {
  background: var(--color-accent);
  color: var(--color-text-on-accent);
}

.add-item-fab.primary:hover {
  background: var(--color-accent-hover);
}
</style>
