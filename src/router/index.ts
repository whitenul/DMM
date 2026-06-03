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
      name: "settings",
      component: () => import("@/pages/settings/SettingsView.vue"),
    },
  ],
});

export default router;
