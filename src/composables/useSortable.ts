import { onMounted, onBeforeUnmount, type Ref } from "vue";
import Sortable from "sortablejs";

export function useSortable(
  containerRef: Ref<HTMLElement | null>,
  options: {
    group?: string;
    handle?: string;
    onEnd: (evt: Sortable.SortableEvent) => void;
    animation?: number;
  }
) {
  let sortable: Sortable | null = null;

  onMounted(() => {
    if (!containerRef.value) return;
    sortable = Sortable.create(containerRef.value, {
      group: options.group,
      handle: options.handle,
      animation: options.animation ?? 150,
      ghostClass: "sortable-ghost",
      chosenClass: "sortable-chosen",
      dragClass: "sortable-drag",
      onEnd: options.onEnd,
    });
  });

  onBeforeUnmount(() => {
    sortable?.destroy();
    sortable = null;
  });

  return { sortable };
}
