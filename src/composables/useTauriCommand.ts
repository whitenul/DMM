import { invoke } from "@tauri-apps/api/core";

interface AppError {
  code: "DATABASE" | "NOT_FOUND" | "PERMISSION" | "IO" | "UNKNOWN";
  message: string;
}

export function useTauriCommand() {
  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      const appError = error as AppError;
      console.error(`[Tauri Command Error] ${command}:`, appError);
      throw appError;
    }
  }

  return { call };
}
