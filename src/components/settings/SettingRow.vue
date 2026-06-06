<template>
  <div class="setting-row">
    <div class="setting-row__label">
      <span class="setting-row__title">
        {{ title }}
        <button
          v-if="description"
          ref="triggerRef"
          type="button"
          class="setting-row__hint"
          :aria-label="description"
          @mouseenter="onShow"
          @mouseleave="onHide"
          @focus="onShow"
          @blur="onHide"
        >
          <svg width="12" height="12" viewBox="0 0 16 16" aria-hidden="true">
            <circle cx="8" cy="8" r="7" fill="none" stroke="currentColor" stroke-width="1.2" />
            <path
              d="M8 7v4M8 5v.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </span>
    </div>
    <div class="setting-row__control"><slot /></div>

    <Teleport to="body">
      <Transition name="tooltip">
        <div
          v-if="visible"
          ref="tooltipRef"
          class="setting-tooltip"
          :style="tooltipStyle"
          role="tooltip"
        >
          {{ description }}
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";

defineProps<{
  title: string;
  description?: string;
}>();

const triggerRef = ref<HTMLButtonElement | null>(null);
const tooltipRef = ref<HTMLDivElement | null>(null);
const visible = ref(false);
const tooltipStyle = ref<Record<string, string>>({});

function onShow() {
  visible.value = true;
  nextTick(positionTooltip);
}

function onHide() {
  visible.value = false;
}

function positionTooltip() {
  if (!triggerRef.value || !tooltipRef.value) return;
  const trigger = triggerRef.value.getBoundingClientRect();
  const tooltip = tooltipRef.value.getBoundingClientRect();
  const margin = 8;
  const viewportW = window.innerWidth;
  const viewportH = window.innerHeight;

  let top = trigger.bottom + margin;
  let left = trigger.left + trigger.width / 2 - tooltip.width / 2;

  if (left < 8) left = 8;
  if (left + tooltip.width > viewportW - 8) {
    left = viewportW - tooltip.width - 8;
  }

  if (top + tooltip.height > viewportH - 8) {
    top = trigger.top - tooltip.height - margin;
  }

  tooltipStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
  };
}
</script>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 0;
  min-height: 40px;
}

.setting-row__label {
  display: flex;
  flex-direction: column;
  gap: 0;
  flex: 1;
  min-width: 0;
  padding-right: 8px;
}

.setting-row__title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-md);
  color: var(--color-text-primary);
}

.setting-row__hint {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: help;
  transition: color 0.15s ease, background 0.15s ease;
  flex-shrink: 0;
}

.setting-row__hint:hover,
.setting-row__hint:focus-visible {
  color: var(--color-accent, #0078d4);
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.06));
  outline: none;
}

.setting-row__hint svg {
  display: block;
}

.setting-row__control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  justify-content: flex-end;
}

.tooltip-enter-active,
.tooltip-leave-active {
  transition: opacity 0.12s ease, transform 0.12s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: translateY(-2px);
}
</style>

<!-- Tooltip 样式需要全局（因为 Teleport 到 body） -->
<style>
.setting-tooltip {
  position: fixed;
  z-index: 9999;
  max-width: 240px;
  padding: 6px 10px;
  background: var(--color-bg-card, #2b2b2b);
  color: var(--color-text-primary, #fff);
  font-size: 12px;
  line-height: 1.5;
  border-radius: var(--radius-sm, 4px);
  border: 1px solid var(--color-border-strong, rgba(255, 255, 255, 0.1));
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  pointer-events: none;
  word-wrap: break-word;
}
</style>
