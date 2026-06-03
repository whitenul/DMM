import { defineStore } from "pinia";
import { ref } from "vue";
import type { SearchResult } from "@/types/search";
import { useTauriCommand } from "@/composables/useTauriCommand";

export const useSearchStore = defineStore("search", () => {
  const { call } = useTauriCommand();
  const query = ref("");
  const results = ref<SearchResult[]>([]);
  const loading = ref(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function search(q: string) {
    query.value = q;
    if (!q.trim()) {
      results.value = [];
      return;
    }
    loading.value = true;
    try {
      results.value = await call<SearchResult[]>("search_items", { query: q.trim() });
    } finally {
      loading.value = false;
    }
  }

  function debouncedSearch(q: string, delay = 150) {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => search(q), delay);
  }

  function clearResults() {
    query.value = "";
    results.value = [];
  }

  return { query, results, loading, search, debouncedSearch, clearResults };
});
