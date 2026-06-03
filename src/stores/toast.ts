import { defineStore } from "pinia";
import { ref } from "vue";

export interface ToastMessage {
  id: number;
  type: "success" | "error" | "warning" | "info";
  message: string;
  duration: number;
}

let nextId = 0;

export const useToastStore = defineStore("toast", () => {
  const messages = ref<ToastMessage[]>([]);

  function show(type: ToastMessage["type"], message: string, duration = 3000) {
    const id = nextId++;
    messages.value.push({ id, type, message, duration });
    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }
  }

  function remove(id: number) {
    messages.value = messages.value.filter((m) => m.id !== id);
  }

  function success(message: string, duration?: number) {
    show("success", message, duration);
  }

  function error(message: string, duration?: number) {
    show("error", message, duration ?? 5000);
  }

  function warning(message: string, duration?: number) {
    show("warning", message, duration);
  }

  function info(message: string, duration?: number) {
    show("info", message, duration);
  }

  return { messages, show, remove, success, error, warning, info };
});
