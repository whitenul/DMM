<template>
  <div
    class="item-card"
    :class="{ pinned: item.is_pinned, selected: selected }"
    role="button"
    :aria-label="item.name"
    @click="onClick"
    @contextmenu.prevent="$emit('contextmenu', $event)"
  >
    <div v-if="selectMode" class="select-check" @click.stop="$emit('toggleSelect', item.id)">
      <AppIcon v-if="selected" name="check" :size="14" class="check-on" />
      <span v-else class="check-off"></span>
    </div>
    <div class="item-icon">
      <img
        v-if="iconUrl"
        :src="iconUrl"
        :alt="item.name"
        class="icon-image"
        @error="onIconError"
      />
      <AppIcon v-else :name="item.item_type" :size="32" class="icon-placeholder" />
    </div>
    <div class="item-info">
      <span class="item-name" :title="item.name">{{ item.name }}</span>
    </div>
    <AppIcon v-if="item.is_pinned && !selectMode" name="pin" :size="12" class="pin-indicator" title="已固定" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Item } from "@/types/item";
import AppIcon from "@/components/common/AppIcon.vue";

const props = defineProps<{
  item: Item;
  selectMode: boolean;
  selected: boolean;
}>();

const emit = defineEmits<{
  launch: [];
  contextmenu: [e: MouseEvent];
  toggleSelect: [id: number];
}>();

const iconLoadFailed = ref(false);
const iconUrl = ref<string | null>(null);

function onClick() {
  if (props.selectMode) {
    emit("toggleSelect", props.item.id);
  } else {
    emit("launch");
  }
}

function onIconError() {
  iconLoadFailed.value = true;
  iconUrl.value = null;
}

async function loadIcon() {
  if (!props.item.icon_path) {
    return;
  }
  try {
    const data = await invoke<string | null>("get_item_icon_base64", { itemId: props.item.id });
    if (data) {
      iconUrl.value = data;
    } else {
      iconLoadFailed.value = true;
    }
  } catch {
    iconLoadFailed.value = true;
  }
}

watch(() => props.item.id, () => {
  iconLoadFailed.value = false;
  iconUrl.value = null;
  loadIcon();
});

onMounted(loadIcon);
</script>

<style scoped>
.item-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: var(--spacing-md) var(--spacing-sm);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  border: 2px solid transparent;
  position: relative;
}

.item-card:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-bg-active);
  transform: translateY(-1px);
}

.item-card.pinned {
  border-color: var(--color-accent);
  background: var(--color-bg-hover);
}

.item-card.selected {
  border-color: var(--color-accent);
  background: var(--color-bg-active);
}

.select-check {
  position: absolute;
  top: 4px;
  left: 4px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

.check-on {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--color-accent);
  color: var(--color-text-on-accent);
  display: flex;
  align-items: center;
  justify-content: center;
}

.check-off {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--color-bg-active);
  background: var(--color-bg-card);
}

.item-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.icon-image {
  width: 40px;
  height: 40px;
  object-fit: contain;
}

.icon-placeholder {
  color: var(--color-text-secondary);
}

.item-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.item-name {
  font-size: var(--font-size-sm);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  width: 100%;
  line-height: 1.3;
}

.pin-indicator {
  position: absolute;
  top: 4px;
  right: 4px;
  opacity: 0.7;
  color: var(--color-warning);
}
</style>
