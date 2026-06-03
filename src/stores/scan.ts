import { defineStore } from "pinia";
import { ref } from "vue";
import { useTauriCommand } from "@/composables/useTauriCommand";

export interface ScannedApp {
  name: string;
  path: string;
  icon_path: string | null;
  app_type: string;
  arguments: string | null;
  working_dir: string | null;
}

export const useScanStore = defineStore("scan", () => {
  const { call } = useTauriCommand();
  const scannedApps = ref<ScannedApp[]>([]);
  const loading = ref(false);

  async function scanStartMenu() {
    loading.value = true;
    try {
      scannedApps.value = await call<ScannedApp[]>("scan_start_menu");
    } finally {
      loading.value = false;
    }
  }

  async function scanUwpApps() {
    loading.value = true;
    try {
      scannedApps.value = await call<ScannedApp[]>("scan_uwp_apps");
    } finally {
      loading.value = false;
    }
  }

  async function scanFolder(folderPath: string) {
    loading.value = true;
    try {
      scannedApps.value = await call<ScannedApp[]>("scan_folder", { folderPath });
    } finally {
      loading.value = false;
    }
  }

  async function importApps(categoryId: number, apps: ScannedApp[]): Promise<number> {
    return await call<number>("import_scanned_apps", { categoryId, apps });
  }

  function clearResults() {
    scannedApps.value = [];
  }

  return { scannedApps, loading, scanStartMenu, scanUwpApps, scanFolder, importApps, clearResults };
});
