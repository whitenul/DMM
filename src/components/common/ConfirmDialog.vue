<template>
  <Teleport to="body">
    <div v-if="visible" class="confirm-overlay" @click.self="$emit('cancel')">
      <div class="confirm-dialog">
        <div class="confirm-header">
          <h3 class="confirm-title">{{ title }}</h3>
        </div>
        <div class="confirm-body">
          <p class="confirm-message">{{ message }}</p>
        </div>
        <div class="confirm-footer">
          <button class="btn btn-secondary" @click="$emit('cancel')">取消</button>
          <button class="btn" :class="danger ? 'btn-danger' : 'btn-primary'" @click="$emit('confirm')">
            {{ confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  title: string;
  message: string;
  danger?: boolean;
  confirmText?: string;
}>();

defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 10001;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-overlay);
  backdrop-filter: blur(4px);
}

.confirm-dialog {
  background: var(--color-bg-solid);
  border: 1px solid var(--color-bg-hover);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-dialog);
  width: 400px;
  max-width: 90vw;
}

.confirm-header {
  padding: 16px 20px 8px;
}

.confirm-title {
  font-size: var(--font-size-xl);
  font-weight: 600;
  color: var(--color-text-primary);
}

.confirm-body {
  padding: 0 20px 16px;
}

.confirm-message {
  font-size: var(--font-size-md);
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.confirm-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px 16px;
}

.btn {
  padding: 6px 20px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-bg-active);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary {
  background: transparent;
  color: var(--color-text-primary);
}

.btn-secondary:hover {
  background: var(--color-bg-hover);
}

.btn-primary {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-text-on-accent);
}

.btn-primary:hover {
  background: var(--color-accent-hover);
}

.btn-danger {
  background: var(--color-danger);
  border-color: var(--color-danger);
  color: white;
}

.btn-danger:hover {
  opacity: 0.9;
}
</style>
