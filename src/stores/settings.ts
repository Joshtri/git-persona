import { create } from "zustand";
import { settingsGet, settingsSet } from "@/ipc/client";
import { type AppError, toAppError } from "@/ipc/errors";
import type { AppSettings } from "@/ipc/types.gen";

interface SettingsStore {
  data: AppSettings | null;
  loading: boolean;
  error: AppError | null;
  fetch: () => Promise<void>;
  update: (settings: AppSettings) => Promise<void>;
}

const defaults: AppSettings = {
  theme: "Dark",
  show_audit_log: true,
  auto_scan_repos: false,
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  data: null,
  loading: false,
  error: null,

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const data = await settingsGet();
      set({ data, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false, data: defaults });
    }
  },

  update: async (settings) => {
    set({ loading: true, error: null });
    try {
      const data = await settingsSet(settings);
      set({ data, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },
}));
