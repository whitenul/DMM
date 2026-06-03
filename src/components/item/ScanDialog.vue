<template>
  <Dialog :visible="visible" title="扫描并导入应用" :width="560" @close="$emit('close')">
    <div class="scan-actions">
      <button class="btn btn-primary" :disabled="scanStore.loading" @click="scanStore.scanStartMenu()">
        {{ scanStore.loading ? "扫描中..." : "扫描开始菜单" }}
      </button>
      <button class="btn btn-primary" :disabled="scanStore.loading" @click="scanStore.scanUwpApps()">
        扫描 UWP 应用
      </button>
    </div>

    <div v-if="scanStore.scannedApps.length" class="scan-results">
      <div class="scan-header">
        <label class="select-all">
          <input v-model="selectAll" type="checkbox" @change="toggleSelectAll" />
          <span>全选 ({{ selectedApps.length }}/{{ scanStore.scannedApps.length }})</span>
        </label>
      </div>
      <div class="scan-list">
        <label
          v-for="(app, index) in scanStore.scannedApps"
          :key="index"
          class="scan-item"
        >
          <input v-model="selectedSet" type="checkbox" :value="index" />
          <AppIcon :name="app.app_type" :size="16" class="scan-item-icon" />
          <div class="scan-item-info">
            <span class="scan-item-name">{{ app.name }}</span>
            <span class="scan-item-path">{{ app.path }}</span>
          </div>
        </label>
      </div>
    </div>

    <div v-else-if="!scanStore.loading" class="scan-empty">
      <AppIcon name="folder-open" :size="32" class="empty-icon" />
      <p>点击上方按钮扫描系统应用</p>
    </div>

    <template #footer>
      <button class="btn btn-secondary" @click="$emit('close')">取消</button>
      <button
        class="btn btn-primary"
        :disabled="!selectedApps.length"
        @click="onImport"
      >
        导入选中 ({{ selectedApps.length }})
      </button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Dialog from "@/components/common/Dialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";
import { useScanStore } from "@/stores/scan";

const props = defineProps<{
  visible: boolean;
  categoryId: number | null;
}>();

const emit = defineEmits<{
  close: [];
  imported: [];
}>();

const scanStore = useScanStore();
const selectedSet = ref<number[]>([]);
const selectAll = ref(false);

const selectedApps = computed(() => {
  return selectedSet.value.map((i) => scanStore.scannedApps[i]);
});

function toggleSelectAll() {
  if (selectAll.value) {
    selectedSet.value = scanStore.scannedApps.map((_, i) => i);
  } else {
    selectedSet.value = [];
  }
}

async function onImport() {
  if (props.categoryId === null || !selectedApps.value.length) return;
  await scanStore.importApps(props.categoryId, selectedApps.value);
  selectedSet.value = [];
  selectAll.value = false;
  emit("imported");
  emit("close");
}
</script>

<style scoped>
.scan-actions {
  margin-bottom: 16px;
}

.btn {
  padding: 6px 20px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-bg-active);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.btn-secondary {
  background: transparent;
  color: var(--color-text-primary);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-bg-hover);
}

.btn-primary {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-accent-hover);
}

.scan-header {
  display: flex;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-bg-hover);
  margin-bottom: 4px;
}

.select-all {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.scan-list {
  max-height: 300px;
  overflow-y: auto;
}

.scan-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 4px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.scan-item:hover {
  background: var(--color-bg-hover);
}

.scan-item input[type="checkbox"] {
  flex-shrink: 0;
}

.scan-item-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.scan-item-info {
  flex: 1;
  min-width: 0;
}

.scan-item-name {
  display: block;
  font-size: var(--font-size-md);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scan-item-path {
  display: block;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scan-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
  padding: 24px;
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
}

.scan-empty .empty-icon {
  opacity: 0.4;
}
</style>
