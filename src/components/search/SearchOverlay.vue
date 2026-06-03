<template>
  <Teleport to="body">
    <div v-if="uiStore.searchOverlayOpen" class="search-overlay-backdrop" @click.self="close">
      <div class="search-overlay" tabindex="-1" @keydown.escape="close" @keydown.down.prevent="selectNext" @keydown.up.prevent="selectPrev" @keydown.enter.prevent="launchSelected">
        <div class="search-bar">
          <AppIcon name="search" :size="18" class="search-icon" />
          <input
            ref="inputRef"
            v-model="searchStore.query"
            class="search-input"
            type="text"
            placeholder="搜索应用..."
            @input="onInput"
          />
          <AppIcon v-if="searchStore.loading" name="loading" :size="14" spin class="loading-indicator" />
          <button class="close-btn" @click="close" title="关闭 (Esc)">
            <AppIcon name="close" :size="14" />
          </button>
        </div>

        <div v-if="searchStore.results.length" class="search-results">
          <div
            v-for="(result, index) in searchStore.results"
            :key="result.item.id"
            class="search-result-item"
            :class="{ selected: index === selectedIndex }"
            @click="launchItem(result)"
            @mouseenter="selectedIndex = index"
          >
            <div class="result-icon">
              <img
                v-if="result.item.icon_path"
                :src="getIconSrc(result.item.icon_path)"
                :alt="result.item.name"
                class="icon-image"
                @error="($event.target as HTMLImageElement).style.display = 'none'"
              />
              <AppIcon v-else :name="result.item.item_type" :size="20" class="icon-placeholder" />
            </div>
            <div class="result-info">
              <span class="result-name">{{ result.item.name }}</span>
              <span class="result-meta">{{ result.category_name }} · {{ result.item.path }}</span>
            </div>
            <span class="result-type-badge">{{ result.item.item_type }}</span>
          </div>
        </div>

        <div v-else-if="searchStore.query && !searchStore.loading" class="no-results">
          <p>未找到匹配的应用</p>
        </div>

        <div v-else-if="!searchStore.query" class="search-hint">
          <p>输入关键词搜索应用</p>
          <p class="hint-sub">支持中文名、拼音首字母搜索</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useSearchStore } from "@/stores/search";
import { useItemStore } from "@/stores/item";
import { useUIStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";
import type { SearchResult } from "@/types/search";

const searchStore = useSearchStore();
const itemStore = useItemStore();
const uiStore = useUIStore();

const inputRef = ref<HTMLInputElement | null>(null);
const selectedIndex = ref(0);

watch(
  () => uiStore.searchOverlayOpen,
  async (open) => {
    if (open) {
      await nextTick();
      inputRef.value?.focus();
      selectedIndex.value = 0;
    } else {
      searchStore.clearResults();
    }
  }
);

function onInput() {
  searchStore.debouncedSearch(searchStore.query);
  selectedIndex.value = 0;
}

function selectNext() {
  if (selectedIndex.value < searchStore.results.length - 1) {
    selectedIndex.value++;
  }
}

function selectPrev() {
  if (selectedIndex.value > 0) {
    selectedIndex.value--;
  }
}

async function launchSelected() {
  const result = searchStore.results[selectedIndex.value];
  if (result) {
    await launchItem(result);
  }
}

async function launchItem(result: SearchResult) {
  await itemStore.launchItem(result.item.id);
  searchStore.clearResults();
  uiStore.closeSearchOverlay();
}

function close() {
  uiStore.closeSearchOverlay();
}

function getIconSrc(iconPath: string): string {
  try {
    return convertFileSrc(iconPath);
  } catch {
    return "";
  }
}
</script>

<style scoped>
.search-overlay-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 80px;
  z-index: 9999;
  backdrop-filter: blur(8px);
}

.search-overlay {
  width: 560px;
  max-width: calc(100vw - 32px);
  max-height: 480px;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-dialog);
  overflow: hidden;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
}

.search-icon {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-lg);
  outline: none;
}

.search-input::placeholder {
  color: var(--color-text-secondary);
  opacity: 0.6;
}

.loading-indicator {
  color: var(--color-text-secondary);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.close-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.search-result-item.selected {
  background: var(--color-bg-active);
}

.result-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: var(--color-bg-solid);
  flex-shrink: 0;
  overflow: hidden;
}

.icon-image {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.icon-placeholder {
  color: var(--color-text-secondary);
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  display: block;
  font-size: var(--font-size-lg);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-meta {
  display: block;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

.result-type-badge {
  font-size: var(--font-size-xs);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  background: var(--color-bg-active);
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.no-results,
.search-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 4px;
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
  padding: 40px 0;
}

.hint-sub {
  font-size: var(--font-size-sm);
  opacity: 0.6;
}
</style>
