<template>
  <Teleport to="body">
    <Transition name="theme-editor-fade">
      <div v-if="visible" class="theme-editor-overlay" @click.self="handleOverlayClick">
        <div class="theme-editor">
          <!-- 头部 -->
          <div class="theme-editor__header">
            <h3>{{ isEditing ? "编辑主题" : "创建主题" }}</h3>
            <button class="theme-editor__close" @click="handleClose">&times;</button>
          </div>

          <!-- 主体：左右分栏 -->
          <div class="theme-editor__body">
            <!-- 左侧：编辑区 -->
            <div class="theme-editor__edit">
              <!-- 主题名称 -->
              <div class="theme-editor__field">
                <label>主题名称</label>
                <input v-model="form.name" type="text" placeholder="我的主题" />
              </div>

              <!-- 强调色系 -->
              <fieldset class="theme-editor__group">
                <legend>强调色</legend>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.accent" label="主色" />
                </div>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.accentHover" label="悬停" />
                  <span v-if="!manualAccentHover" class="theme-editor__auto-tag">自动</span>
                  <button v-else class="theme-editor__reset-btn" @click="resetField('accentHover')">重置</button>
                </div>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.accentPressed" label="按下" />
                  <span v-if="!manualAccentPressed" class="theme-editor__auto-tag">自动</span>
                  <button v-else class="theme-editor__reset-btn" @click="resetField('accentPressed')">重置</button>
                </div>
              </fieldset>

              <!-- 背景色 -->
              <fieldset class="theme-editor__group">
                <legend>背景色</legend>
                <div v-for="key in bgKeys" :key="key" class="theme-editor__row">
                  <ColorPicker v-model="form.colors[key]" :label="bgLabels[key]" />
                </div>
              </fieldset>

              <!-- 文字色 -->
              <fieldset class="theme-editor__group">
                <legend>文字色</legend>
                <div v-for="key in textKeys" :key="key" class="theme-editor__row">
                  <ColorPicker v-model="form.colors[key]" :label="textLabels[key]" />
                </div>
              </fieldset>

              <!-- 边框色 -->
              <fieldset class="theme-editor__group">
                <legend>边框/阴影色</legend>
                <div v-for="key in borderKeys" :key="key" class="theme-editor__row">
                  <ColorPicker v-model="form.colors[key]" :label="borderLabels[key]" />
                </div>
              </fieldset>

              <!-- 语义色 -->
              <fieldset class="theme-editor__group">
                <legend>语义色</legend>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.danger" label="危险" />
                </div>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.warning" label="警告" />
                </div>
                <div class="theme-editor__row">
                  <ColorPicker v-model="form.colors.success" label="成功" />
                </div>
              </fieldset>
            </div>

            <!-- 右侧：实时预览 -->
            <div class="theme-editor__preview-panel">
              <div class="theme-editor__preview-title">实时预览</div>
              <div class="theme-editor__preview" :style="previewStyle">
                <div class="preview-sidebar">
                  <div class="preview-item" style="opacity: 0.6">侧边栏项</div>
                  <div class="preview-item preview-item--active">选中项</div>
                  <div class="preview-item" style="opacity: 0.4">禁用项</div>
                </div>
                <div class="preview-content">
                  <div class="preview-card">
                    <div class="preview-text">主要文字</div>
                    <div class="preview-text preview-text--secondary">次要文字</div>
                    <div class="preview-text preview-text--tertiary">三级文字</div>
                    <div class="preview-actions">
                      <button class="preview-btn">强调按钮</button>
                      <button class="preview-btn preview-btn--danger">危险按钮</button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="theme-editor__footer">
            <button class="theme-editor__btn theme-editor__btn--cancel" @click="handleClose">取消</button>
            <button class="theme-editor__btn theme-editor__btn--save" @click="save">保存主题</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from "vue";
import ColorPicker from "./ColorPicker.vue";
import { useThemeStore } from "@/stores/theme";
import { deriveAccentColors, resolveAlphaColor } from "@/utils/color";
import type { ThemeDefinition, ThemeColors } from "@/types/theme";

const props = defineProps<{
  visible: boolean;
  theme: ThemeDefinition | null;
}>();

const emit = defineEmits<{
  "update:visible": [val: boolean];
  save: [theme: ThemeDefinition];
}>();

const themeStore = useThemeStore();
const isEditing = computed(() => !!props.theme);

const bgKeys: (keyof ThemeColors)[] = ["bgSolid", "bgCard", "bgHover", "bgActive", "bgSubtle", "bgInset"];
const textKeys: (keyof ThemeColors)[] = ["textPrimary", "textSecondary", "textTertiary", "textDisabled", "textOnAccent", "textOnDanger"];
const borderKeys: (keyof ThemeColors)[] = ["border", "borderStrong", "borderAccent", "divider", "overlay"];

const bgLabels: Record<string, string> = { bgSolid: "底色", bgCard: "卡片", bgHover: "悬停", bgActive: "激活", bgSubtle: "次级", bgInset: "内嵌" };
const textLabels: Record<string, string> = { textPrimary: "主要", textSecondary: "次要", textTertiary: "三级", textDisabled: "禁用", textOnAccent: "反色", textOnDanger: "危险文字" };
const borderLabels: Record<string, string> = { border: "边框", borderStrong: "强边框", borderAccent: "强调边框", divider: "分割线", overlay: "遮罩" };

// 追踪哪些字段是手动修改的
const manualAccentHover = ref(false);
const manualAccentPressed = ref(false);

const form = reactive<{
  name: string;
  colors: Record<string, string>;
}>({
  name: "",
  colors: {},
});

/** 获取当前 resolvedTheme 对应的 mode */
const currentMode = computed(() => themeStore.resolvedTheme);

// 初始化表单
watch(() => props.visible, (v) => {
  if (!v) return;
  if (props.theme) {
    form.name = props.theme.name;
    form.colors = { ...props.theme.colors };
  } else {
    const mode = currentMode.value;
    form.name = "";
    form.colors = {
      accent: "#0078d4",
      bgSolid: mode === "dark" ? "#1f1f1f" : "#f3f3f3",
      bgCard: mode === "dark" ? "#2c2c2c" : "#ffffff",
      bgHover: mode === "dark" ? "#ffffff" : "#000000",
      bgActive: mode === "dark" ? "#ffffff" : "#000000",
      bgSubtle: mode === "dark" ? "#161616" : "#fafafa",
      bgInset: mode === "dark" ? "#0e0e0e" : "#ebebeb",
      textPrimary: mode === "dark" ? "#ffffff" : "#000000",
      textSecondary: mode === "dark" ? "#ffffff" : "#000000",
      textTertiary: mode === "dark" ? "#ffffff" : "#000000",
      textDisabled: mode === "dark" ? "#ffffff" : "#000000",
      textOnAccent: "#ffffff",
      textOnDanger: "#ffffff",
      danger: mode === "dark" ? "#ff6b6b" : "#e81123",
      warning: "#ffb900",
      success: "#00cc6a",
      border: mode === "dark" ? "#ffffff" : "#000000",
      borderStrong: mode === "dark" ? "#ffffff" : "#000000",
      borderAccent: "#0078d4",
      divider: mode === "dark" ? "#ffffff" : "#000000",
      overlay: "#000000",
      closeHover: "#c42b1c",
    };
  }
  manualAccentHover.value = !!form.colors.accentHover;
  manualAccentPressed.value = !!form.colors.accentPressed;
});

// 自动推导强调色变体
watch(() => form.colors.accent, (newAccent) => {
  if (!manualAccentHover.value) {
    const derived = deriveAccentColors(newAccent, currentMode.value);
    form.colors.accentHover = derived.accentHover;
    form.colors.accentSubtle = derived.accentSubtle;
    form.colors.accentText = derived.accentText;
  }
  if (!manualAccentPressed.value) {
    const derived = deriveAccentColors(newAccent, currentMode.value);
    form.colors.accentPressed = derived.accentPressed;
  }
  form.colors.borderAccent = newAccent;
});

function resetField(field: "accentHover" | "accentPressed") {
  const derived = deriveAccentColors(form.colors.accent, currentMode.value);
  form.colors[field] = derived[field === "accentHover" ? "accentHover" : "accentPressed"];
  if (field === "accentHover") manualAccentHover.value = false;
  if (field === "accentPressed") manualAccentPressed.value = false;
}

// 实时预览样式
const previewStyle = computed(() => {
  const mode = currentMode.value;
  return {
    backgroundColor: form.colors.bgSolid,
    color: resolveAlphaColor(form.colors.textPrimary, mode, "textPrimary"),
    "--preview-card-bg": form.colors.bgCard,
    "--preview-accent": form.colors.accent,
    "--preview-accent-hover": form.colors.accentHover,
    "--preview-text-secondary": resolveAlphaColor(form.colors.textSecondary, mode, "textSecondary"),
    "--preview-text-tertiary": resolveAlphaColor(form.colors.textTertiary, mode, "textTertiary"),
    "--preview-hover": resolveAlphaColor(form.colors.bgHover, mode, "bgHover"),
    "--preview-border": resolveAlphaColor(form.colors.border, mode, "border"),
    "--preview-danger": form.colors.danger,
  };
});

function handleClose() {
  emit("update:visible", false);
}

function handleOverlayClick() {
  handleClose();
}

function save() {
  const id = props.theme?.id ?? `custom_${Date.now()}`;
  const theme: ThemeDefinition = {
    id,
    name: form.name || "未命名主题",
    mode: currentMode.value,
    isBuiltIn: false,
    colors: form.colors as unknown as ThemeColors,
  };
  emit("save", theme);
}
</script>

<style scoped>
/* 过渡动画 */
.theme-editor-fade-enter-active,
.theme-editor-fade-leave-active {
  transition: opacity 0.2s ease;
}

.theme-editor-fade-enter-from,
.theme-editor-fade-leave-to {
  opacity: 0;
}

.theme-editor-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-overlay, rgba(0, 0, 0, 0.3));
}

.theme-editor {
  width: 720px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-card, #ffffff);
  border-radius: var(--radius-xl, 16px);
  box-shadow: var(--shadow-dialog, 0 8px 16px rgba(0, 0, 0, 0.16));
  overflow: hidden;
}

.theme-editor__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-divider, rgba(0, 0, 0, 0.06));
}

.theme-editor__header h3 {
  font-size: var(--font-size-lg, 14px);
  font-weight: 600;
  margin: 0;
}

.theme-editor__close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
  padding: 4px 8px;
  border-radius: var(--radius-sm, 4px);
  transition: background 0.15s ease;
}

.theme-editor__close:hover {
  background: var(--color-bg-hover, rgba(0, 0, 0, 0.04));
}

/* 主体：左右分栏 */
.theme-editor__body {
  flex: 1;
  overflow: hidden;
  display: flex;
}

/* 左侧编辑区 */
.theme-editor__edit {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-right: 1px solid var(--color-divider, rgba(0, 0, 0, 0.06));
}

.theme-editor__field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.theme-editor__field label {
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
}

.theme-editor__field input[type="text"] {
  padding: 6px 10px;
  border: 1px solid var(--color-border, rgba(0, 0, 0, 0.06));
  border-radius: var(--radius-sm, 4px);
  background: var(--color-bg-solid, #f3f3f3);
  color: var(--color-text-primary, rgba(0, 0, 0, 0.9));
  font-size: var(--font-size-md, 13px);
  outline: none;
  transition: border-color 0.15s ease;
}

.theme-editor__field input[type="text"]:focus {
  border-color: var(--color-accent, #0078d4);
}

.theme-editor__group {
  border: 1px solid var(--color-divider, rgba(0, 0, 0, 0.06));
  border-radius: var(--radius-md, 8px);
  padding: 10px 12px;
}

.theme-editor__group legend {
  font-size: var(--font-size-sm, 12px);
  font-weight: 600;
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
  padding: 0 4px;
}

.theme-editor__row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 0;
}

.theme-editor__auto-tag {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 2px;
  background: var(--color-accent-subtle, #e6f2ff);
  color: var(--color-accent, #0078d4);
  flex-shrink: 0;
}

.theme-editor__reset-btn {
  font-size: 10px;
  padding: 1px 4px;
  border: none;
  border-radius: 2px;
  background: none;
  color: var(--color-text-tertiary, rgba(0, 0, 0, 0.4));
  cursor: pointer;
  text-decoration: underline;
  flex-shrink: 0;
}

/* 右侧预览面板 */
.theme-editor__preview-panel {
  width: 260px;
  flex-shrink: 0;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
}

.theme-editor__preview-title {
  font-size: var(--font-size-sm, 12px);
  font-weight: 600;
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
}

.theme-editor__preview {
  display: flex;
  border-radius: var(--radius-md, 8px);
  overflow: hidden;
  height: 200px;
  border: 1px solid var(--preview-border, rgba(0, 0, 0, 0.06));
  flex: 1;
}

.preview-sidebar {
  width: 80px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border-right: 1px solid var(--preview-border, rgba(0, 0, 0, 0.06));
}

.preview-item {
  padding: 4px 8px;
  border-radius: var(--radius-sm, 4px);
  font-size: 11px;
}

.preview-item--active {
  background: var(--preview-hover, rgba(0, 0, 0, 0.04));
}

.preview-content {
  flex: 1;
  padding: 12px;
  display: flex;
  align-items: flex-start;
}

.preview-card {
  background: var(--preview-card-bg, #ffffff);
  border-radius: var(--radius-md, 8px);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border: 1px solid var(--preview-border, rgba(0, 0, 0, 0.06));
  width: 100%;
}

.preview-text {
  font-size: 12px;
}

.preview-text--secondary {
  color: var(--preview-text-secondary, rgba(0, 0, 0, 0.6));
  font-size: 11px;
}

.preview-text--tertiary {
  color: var(--preview-text-tertiary, rgba(0, 0, 0, 0.4));
  font-size: 11px;
}

.preview-actions {
  display: flex;
  gap: 6px;
  margin-top: 4px;
}

.preview-btn {
  padding: 4px 12px;
  border: none;
  border-radius: var(--radius-sm, 4px);
  background: var(--preview-accent, #0078d4);
  color: #ffffff;
  font-size: 11px;
  cursor: pointer;
}

.preview-btn--danger {
  background: var(--preview-danger, #e81123);
}

/* 底部按钮 */
.theme-editor__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--color-divider, rgba(0, 0, 0, 0.06));
}

.theme-editor__btn {
  padding: 6px 16px;
  border-radius: var(--radius-sm, 4px);
  font-size: var(--font-size-md, 13px);
  cursor: pointer;
  border: none;
  transition: opacity 0.15s ease;
}

.theme-editor__btn:hover {
  opacity: 0.85;
}

.theme-editor__btn--cancel {
  background: var(--color-bg-subtle, #fafafa);
  color: var(--color-text-secondary, rgba(0, 0, 0, 0.6));
}

.theme-editor__btn--save {
  background: var(--color-accent, #0078d4);
  color: #ffffff;
}
</style>
