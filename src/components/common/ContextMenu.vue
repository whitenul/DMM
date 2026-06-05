<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="context-menu-overlay"
      @click="$emit('close')"
      @contextmenu.prevent="$emit('close')"
    >
      <div
        class="context-menu"
        :style="menuStyle"
        role="menu"
        @click.stop
      >
        <template v-for="(item, index) in items" :key="index">
          <div v-if="item.divider" class="context-menu-divider" role="separator" />
          <button
            v-else
            class="context-menu-item"
            :class="{ disabled: item.disabled }"
            role="menuitem"
            @click="onItemClick(item)"
          >
            <AppIcon v-if="item.icon" :name="item.icon" :size="14" class="menu-item-icon" />
            <span class="menu-item-label">{{ item.label }}</span>
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from "vue";
import AppIcon from "@/components/common/AppIcon.vue";

export interface MenuItem {
  label: string;
  icon?: string;
  action: string;
  disabled?: boolean;
  divider?: false;
}

export interface MenuDivider {
  divider: true;
}

export type MenuEntry = MenuItem | MenuDivider;

const props = defineProps<{
  visible: boolean;
  x: number;
  y: number;
  items: MenuEntry[];
}>();

const emit = defineEmits<{
  close: [];
  select: [action: string];
}>();

const menuStyle = computed(() => {
  const menuWidth = 180;
  const menuHeight = props.items.length * 36;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  let left = props.x;
  let top = props.y;
  if (left + menuWidth > vw) left = vw - menuWidth - 8;
  if (top + menuHeight > vh) top = vh - menuHeight - 8;
  if (left < 0) left = 8;
  if (top < 0) top = 8;
  return { left: `${left}px`, top: `${top}px` };
});

function onItemClick(item: MenuItem) {
  if (item.disabled) return;
  emit("select", item.action);
  emit("close");
}
</script>

<style scoped>
.context-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
}

.context-menu {
  position: absolute;
  min-width: 160px;
  padding: 4px;
  background: var(--color-bg-solid);
  border: 1px solid var(--color-bg-hover);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-dialog);
  backdrop-filter: blur(20px);
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: background var(--transition-fast);
  text-align: left;
}

.context-menu-item:hover:not(.disabled) {
  background: var(--color-bg-hover);
}

.context-menu-item.disabled {
  color: var(--color-text-secondary);
  opacity: 0.5;
  cursor: default;
}

.menu-item-icon {
  width: 18px;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.menu-item-label {
  flex: 1;
}

.context-menu-divider {
  height: 1px;
  margin: 4px 8px;
  background: var(--color-bg-hover);
}
</style>
