<template>
  <Teleport to="body">
    <div class="toast-container" aria-live="polite" aria-atomic="true">
      <TransitionGroup name="toast">
        <div
          v-for="msg in toastStore.messages"
          :key="msg.id"
          class="toast"
          :class="msg.type"
          @click="toastStore.remove(msg.id)"
        >
          <AppIcon :name="msg.type" :size="16" class="toast-icon" />
          <span class="toast-message">{{ msg.message }}</span>
          <button class="toast-close" @click.stop="toastStore.remove(msg.id)">✕</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToastStore } from "@/stores/toast";
import AppIcon from "@/components/common/AppIcon.vue";

const toastStore = useToastStore();
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  background: var(--color-bg-card);
  border: 1px solid var(--color-bg-active);
  box-shadow: var(--shadow-dialog);
  pointer-events: auto;
  cursor: pointer;
  max-width: 360px;
  min-width: 200px;
}

.toast.error {
  border-left: 3px solid var(--color-danger);
  color: var(--color-danger);
}

.toast.warning {
  border-left: 3px solid var(--color-warning);
  color: var(--color-warning);
}

.toast.success {
  border-left: 3px solid var(--color-success);
  color: var(--color-success);
}

.toast.info {
  border-left: 3px solid var(--color-accent);
  color: var(--color-accent);
}

.toast-icon {
  flex-shrink: 0;
}

.toast-message {
  flex: 1;
  font-size: var(--font-size-md);
  line-height: 1.4;
  color: var(--color-text-primary);
  word-break: break-word;
}

.toast-close {
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  padding: 2px;
  line-height: 1;
  opacity: 0.6;
}

.toast-close:hover {
  opacity: 1;
}

.toast-enter-active {
  transition: all 0.25s ease-out;
}

.toast-leave-active {
  transition: all 0.2s ease-in;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}
</style>
