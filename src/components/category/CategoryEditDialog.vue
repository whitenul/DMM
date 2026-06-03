<template>
  <Dialog :visible="visible" :title="isEdit ? '编辑分类' : '新建分类'" :width="360" @close="$emit('close')" @confirm="onConfirm">
    <div class="form-group">
      <label class="form-label">名称</label>
      <input
        v-model="form.name"
        class="form-input"
        type="text"
        placeholder="输入分类名称"
        @keyup.enter="onConfirm"
      />
    </div>
    <div class="form-group">
      <label class="form-label">图标</label>
      <div class="icon-picker">
        <button
          v-for="opt in iconOptions"
          :key="opt.value"
          class="icon-option"
          :class="{ active: form.icon === opt.value }"
          :title="opt.value"
          @click="form.icon = opt.value"
        >
          <AppIcon :name="opt.value" :size="18" />
        </button>
      </div>
    </div>
    <div class="form-group">
      <label class="form-label">关联文件夹路径（可选）</label>
      <input
        v-model="form.folderPath"
        class="form-input"
        type="text"
        placeholder="如 D:\Games"
      />
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import Dialog from "@/components/common/Dialog.vue";
import AppIcon from "@/components/common/AppIcon.vue";
import { CATEGORY_ICON_OPTIONS } from "@/utils/icons";

const props = defineProps<{
  visible: boolean;
  editData?: { name: string; icon: string | null; folderPath: string | null } | null;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [data: { name: string; icon: string; folderPath: string }];
}>();

const isEdit = computed(() => !!props.editData);

const iconOptions = CATEGORY_ICON_OPTIONS;

const DEFAULT_ICON = "Folder";

const form = reactive({
  name: "",
  icon: DEFAULT_ICON,
  folderPath: "",
});

watch(
  () => props.visible,
  (val) => {
    if (val) {
      if (props.editData) {
        form.name = props.editData.name;
        form.icon = props.editData.icon || DEFAULT_ICON;
        form.folderPath = props.editData.folderPath || "";
      } else {
        form.name = "";
        form.icon = DEFAULT_ICON;
        form.folderPath = "";
      }
    }
  }
);

function onConfirm() {
  if (!form.name.trim()) return;
  emit("confirm", {
    name: form.name.trim(),
    icon: form.icon,
    folderPath: form.folderPath.trim(),
  });
}
</script>

<style scoped>
.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-bg-active);
  border-radius: var(--radius-sm);
  background: var(--color-bg-card);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  border-color: var(--color-accent);
}

.icon-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.icon-option {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-option:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.icon-option.active {
  border-color: var(--color-accent);
  background: var(--color-bg-hover);
  color: var(--color-accent);
}
</style>
