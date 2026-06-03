<script setup lang="ts">
import { computed } from "vue";
import { resolveIcon, type IconComponent } from "@/utils/icons";

interface Props {
  /** 图标名（业务 key 或 Lucide PascalCase 名） */
  name: string;
  /** 图标尺寸（px） */
  size?: number | string;
  /** 描边宽度（默认 2） */
  strokeWidth?: number | string;
  /** 颜色（默认 currentColor 继承文字色） */
  color?: string;
  /** 是否旋转（用于 loading 状态） */
  spin?: boolean;
  /** 无障碍标签 */
  title?: string;
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  strokeWidth: 2,
  color: "currentColor",
  spin: false,
  title: undefined,
});

const IconCmp = computed<IconComponent>(() => resolveIcon(props.name));
</script>

<template>
  <component
    :is="IconCmp"
    :size="size"
    :stroke-width="strokeWidth"
    :color="color"
    :class="['app-icon', { 'app-icon--spin': spin }]"
    :title="title"
    aria-hidden="true"
  />
</template>

<style>
.app-icon {
  flex-shrink: 0;
  vertical-align: middle;
}

@keyframes app-icon-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.app-icon--spin {
  animation: app-icon-spin 1s linear infinite;
  transform-origin: center;
}
</style>
