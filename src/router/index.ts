import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "main",
      component: () => import("@/pages/main/MainWindow.vue"),
    },
    {
      path: "/search",
      name: "search",
      component: () => import("@/pages/search/SearchWindow.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/pages/settings/SettingsWindow.vue"),
    },
  ],
});

export default router;
