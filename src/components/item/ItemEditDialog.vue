<template>
  <Dialog :visible="visible" :title="isEdit ? '编辑应用' : '添加应用'" :width="420" @close="$emit('close')" @confirm="onConfirm">
    <div class="form-group">
      <label class="form-label">名称</label>
      <input
        v-model="form.name"
        class="form-input"
        type="text"
        placeholder="应用名称"
        @keyup.enter="onConfirm"
      />
    </div>
    <div class="form-group">
      <label class="form-label">类型</label>
      <div class="type-selector">
        <button
          v-for="t in typeOptions"
          :key="t.value"
          class="type-option"
          :class="{ active: form.itemType === t.value }"
          @click="form.itemType = t.value"
        >
          <span class="type-icon">{{ t.icon }}</span>
          <span>{{ t.label }}</span>
        </button>
      </div>
    </div>
    <div class="form-group">
      <label class="form-label">路径</label>
      <div class="input-with-btn">
        <input
          v-model="form.path"
          class="form-input"
          type="text"
          :placeholder="pathPlaceholder"
          @blur="onPathBlur"
        />
        <button v-if="form.itemType === 'Web'" class="browse-btn" @click="onFetchMeta" title="获取网页信息" :disabled="fetchingMeta">
          <AppIcon :name="fetchingMeta ? 'loading' : 'search'" :size="14" :spin="fetchingMeta" />
        </button>
        <button v-else class="browse-btn" @click="onBrowse" title="浏览">
          <AppIcon name="folder-open" :size="14" />
        </button>
      </div>
    </div>
    <div class="form-group">
      <label class="form-label">启动参数（可选）</label>
      <input
        v-model="form.arguments"
        class="form-input"
        type="text"
        placeholder="如 --fullscreen"
      />
    </div>
    <div class="form-group">
      <label class="form-label">工作目录（可选）</label>
      <input
        v-model="form.workingDir"
        class="form-input"
        type="text"
        placeholder="留空则使用路径所在目录"
      />
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { reactive, computed, watch, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import Dialog from "@/components/common/Dialog.vue";

const props = defineProps<{
  visible: boolean;
  categoryId: number | null;
  editData?: {
    name: string;
    itemType: string;
    path: string;
    arguments: string | null;
    workingDir: string | null;
  } | null;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [data: {
    categoryId: number;
    name: string;
    itemType: string;
    path: string;
    arguments: string;
    workingDir: string;
  }];
}>();

const isEdit = computed(() => !!props.editData);

const typeOptions = [
  { value: "App", label: "应用程序", icon: "App" },
  { value: "File", label: "文件", icon: "File" },
  { value: "Folder", label: "文件夹", icon: "Folder" },
  { value: "Web", label: "网页链接", icon: "Web" },
];

const pathPlaceholder = computed(() => {
  const hints: Record<string, string> = {
    App: "如 C:\\Program Files\\app\\app.exe",
    File: "文件路径",
    Folder: "文件夹路径",
    Web: "如 https://example.com",
  };
  return hints[form.itemType] || "路径";
});

const form = reactive({
  name: "",
  itemType: "App",
  path: "",
  arguments: "",
  workingDir: "",
});

const fetchingMeta = ref(false);

async function onFetchMeta() {
  const url = form.path.trim();
  if (!url || !url.startsWith("http")) return;
  fetchingMeta.value = true;
  try {
    const meta = await invoke<{ title: string | null; icon_path: string | null }>("fetch_web_meta", { url });
    if (meta.title && !form.name.trim()) {
      form.name = meta.title;
    }
  } catch {
    // non-critical
  } finally {
    fetchingMeta.value = false;
  }
}

function onPathBlur() {
  if (form.itemType === "Web" && form.path.trim().startsWith("http") && !form.name.trim()) {
    onFetchMeta();
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      if (props.editData) {
        form.name = props.editData.name;
        form.itemType = props.editData.itemType;
        form.path = props.editData.path;
        form.arguments = props.editData.arguments || "";
        form.workingDir = props.editData.workingDir || "";
      } else {
        form.name = "";
        form.itemType = "App";
        form.path = "";
        form.arguments = "";
        form.workingDir = "";
      }
    }
  }
);

async function onBrowse() {
  if (form.itemType === "Web") return;
  if (form.itemType === "Folder") {
    const selected = await open({ directory: true });
    if (selected) {
      form.path = selected;
    }
  } else {
    const filters = form.itemType === "App"
      ? [{ name: "可执行文件", extensions: ["exe", "bat", "cmd", "lnk"] }]
      : undefined;
    const selected = await open({
      multiple: false,
      filters,
    });
    if (selected) {
      form.path = selected;
      if (!form.name && typeof selected === "string") {
        const parts = selected.replace(/\\/g, "/").split("/");
        form.name = parts[parts.length - 1].replace(/\.[^.]+$/, "");
      }
    }
  }
}

function onConfirm() {
  if (!form.name.trim() || !form.path.trim()) return;
  if (props.categoryId === null) return;
  emit("confirm", {
    categoryId: props.categoryId,
    name: form.name.trim(),
    itemType: form.itemType,
    path: form.path.trim(),
    arguments: form.arguments.trim(),
    workingDir: form.workingDir.trim(),
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

.type-selector {
  display: flex;
  gap: 6px;
}

.type-option {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: 1px solid var(--color-bg-active);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.type-option:hover {
  background: var(--color-bg-hover);
}

.type-option.active {
  border-color: var(--color-accent);
  background: var(--color-bg-hover);
  color: var(--color-accent);
}

.type-icon {
  font-size: var(--font-size-lg);
}

.input-with-btn {
  display: flex;
  gap: 4px;
}

.input-with-btn .form-input {
  flex: 1;
}

.browse-btn {
  padding: 8px 12px;
  border: 1px solid var(--color-bg-active);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-md);
  transition: all var(--transition-fast);
}

.browse-btn:hover {
  background: var(--color-bg-hover);
}
</style>
