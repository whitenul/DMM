<template>
  <div
    class="category-item"
    :class="{ active: active, 'drop-target': isDropTarget }"
    @click="$emit('select')"
    @contextmenu.prevent="$emit('contextmenu', $event)"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
  >
    <span class="category-icon">
      <AppIcon :name="category.icon || 'Folder'" :size="16" />
    </span>
    <span v-if="!collapsed" class="category-name">{{ category.name }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { Category } from "@/types/category";
import { useUIStore } from "@/stores/ui";
import { useItemStore } from "@/stores/item";
import { useCategoryStore } from "@/stores/category";
import { useToastStore } from "@/stores/toast";

const props = defineProps<{
  category: Category;
  active: boolean;
  collapsed: boolean;
}>();

defineEmits<{
  select: [];
  contextmenu: [e: MouseEvent];
}>();

const uiStore = useUIStore();
const itemStore = useItemStore();
const categoryStore = useCategoryStore();
const toast = useToastStore();
const isDropTarget = ref(false);

function onDragOver(e: DragEvent) {
  if (!uiStore.dragItem) return;
  if (uiStore.dragItem.sourceCategoryId === props.category.id) return;
  isDropTarget.value = true;
  e.dataTransfer!.dropEffect = "move";
}

function onDragLeave() {
  isDropTarget.value = false;
}

async function onDrop() {
  isDropTarget.value = false;
  const dragInfo = uiStore.dragItem;
  if (!dragInfo) return;
  if (dragInfo.sourceCategoryId === props.category.id) return;

  try {
    await itemStore.moveItem(dragInfo.itemId, props.category.id);
    toast.success(`已移动到「${props.category.name}」`);
    if (categoryStore.activeCategoryId !== null) {
      await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
    }
  } catch {
    toast.error("移动失败");
  }
  uiStore.endDrag();
}
</script>

<style scoped>
.category-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
  white-space: nowrap;
  border: 2px solid transparent;
}

.category-item:hover {
  background: var(--color-bg-hover);
}

.category-item.active {
  background: var(--color-bg-active);
  color: var(--color-accent);
}

.category-item.drop-target {
  background: var(--color-bg-active);
  border-color: var(--color-accent);
}

.category-icon {
  font-size: var(--font-size-xl);
  flex-shrink: 0;
}

.category-name {
  font-size: var(--font-size-md);
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
