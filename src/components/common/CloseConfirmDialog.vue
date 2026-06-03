<template>
  <Dialog :visible="visible" title="关闭确认" :width="380" @close="$emit('cancel')">
    <p class="close-message">你希望关闭 Desk Manager 后应用如何处理？</p>
    <label class="remember-row">
      <input v-model="remember" type="checkbox" class="remember-input" />
      <span>记住我的选择（可在设置中调整）</span>
    </label>

    <template #footer>
      <button class="btn btn-secondary" @click="$emit('cancel')">取消</button>
      <button class="btn btn-secondary" @click="onConfirm('minimize_to_tray')">
        <AppIcon name="chevron-down" :size="14" />
        <span>最小化到托盘</span>
      </button>
      <button class="btn btn-danger" @click="onConfirm('quit')">
        <AppIcon name="close" :size="14" />
        <span>退出应用</span>
      </button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import Dialog from "@/components/common/Dialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";

export type CloseBehavior = "ask" | "minimize_to_tray" | "quit";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  cancel: [];
  confirm: [data: { behavior: CloseBehavior; remember: boolean }];
}>();

const remember = ref(true);

watch(
  () => remember.value,
  (val) => {
    if (val) {
      remember.value = true;
    }
  }
);

function onConfirm(behavior: "minimize_to_tray" | "quit") {
  emit("confirm", { behavior, remember: remember.value });
}
</script>

<style scoped>
.close-message {
  font-size: var(--font-size-lg);
  line-height: 1.6;
  color: var(--color-text-primary);
  margin-bottom: 14px;
}

.remember-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  user-select: none;
}

.remember-input {
  cursor: pointer;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
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

.btn-danger {
  background: var(--color-danger);
  border-color: var(--color-danger);
  color: var(--color-text-on-accent);
}

.btn-danger:hover {
  background: var(--color-danger-hover, var(--color-danger));
  filter: brightness(1.1);
}
</style>
