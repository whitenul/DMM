<template>
  <div class="settings-view">
    <div class="settings-header">
      <button class="back-btn" @click="router.push('/')" :title="t('settings.back')">
        <AppIcon name="chevron-left" :size="14" />
        <span>{{ t('settings.back') }}</span>
      </button>
      <h2 class="settings-title">{{ t('settings.title') }}</h2>
    </div>
    <div class="settings-body">
      <nav class="settings-nav">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="settings-nav__item"
          active-class="settings-nav__item--active"
        >
          <AppIcon :name="item.icon" :size="16" />
          <span>{{ t(item.labelKey) }}</span>
        </router-link>
      </nav>
      <div class="settings-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import AppIcon from "@/components/common/AppIcon.vue";
import { useI18n } from "@/composables/useI18n";

const router = useRouter();
const { t } = useI18n();

const navItems = [
  { path: "/settings/appearance", labelKey: "settings.nav.appearance", icon: "palette" },
  { path: "/settings/shortcut", labelKey: "settings.nav.shortcut", icon: "keyboard" },
  { path: "/settings/scan", labelKey: "settings.nav.scan", icon: "search" },
  { path: "/settings/about", labelKey: "settings.nav.about", icon: "info" },
];
</script>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-solid);
  color: var(--color-text-primary);
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.06));
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  border-radius: var(--radius-sm, 4px);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all 0.15s ease;
}

.back-btn:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
  color: var(--color-text-primary);
}

.settings-title {
  font-size: var(--font-size-2xl);
  font-weight: 600;
}

.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.settings-nav {
  width: 140px;
  min-width: 140px;
  padding: 12px 8px;
  border-right: 1px solid var(--color-border, rgba(255,255,255,0.06));
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.settings-nav__item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-sm, 4px);
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: var(--font-size-md);
  transition: all 0.15s ease;
}

.settings-nav__item:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
  color: var(--color-text-primary);
}

.settings-nav__item--active {
  background: var(--color-bg-hover, rgba(255,255,255,0.06));
  color: var(--color-accent, #0078d4);
}

.settings-content {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px 24px 24px;
}
</style>
