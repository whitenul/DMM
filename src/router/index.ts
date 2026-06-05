import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "main",
      component: () => import("@/pages/main/MainView.vue"),
    },
    {
      path: "/settings",
      component: () => import("@/pages/settings/SettingsView.vue"),
      children: [
        { path: "", redirect: "/settings/appearance" },
        { path: "appearance", component: () => import("@/pages/settings/sections/AppearanceSection.vue") },
        { path: "shortcut", component: () => import("@/pages/settings/sections/ShortcutSection.vue") },
        { path: "scan", component: () => import("@/pages/settings/sections/ScanSection.vue") },
        { path: "about", component: () => import("@/pages/settings/sections/AboutSection.vue") },
      ],
    },
  ],
});

export default router;
