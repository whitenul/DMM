<template>
  <ContentArea />
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useItemStore } from "@/stores/item";
import { useCategoryStore } from "@/stores/category";
import ContentArea from "@/components/layout/ContentArea.vue";

const itemStore = useItemStore();
const categoryStore = useCategoryStore();

onMounted(async () => {
  if (categoryStore.activeCategoryId !== null) {
    await itemStore.fetchItemsByCategory(categoryStore.activeCategoryId);
  }
});

watch(
  () => categoryStore.activeCategoryId,
  (newId) => {
    if (newId !== null) {
      itemStore.fetchItemsByCategory(newId);
    }
  }
);
</script>
