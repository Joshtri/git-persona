import { create } from "zustand";
import { activityList } from "@/ipc/client";
import { type AppError, toAppError } from "@/ipc/errors";
import type { AuditEntry } from "@/ipc/types.gen";

interface ActivityStore {
  items: AuditEntry[];
  loading: boolean;
  error: AppError | null;
  fetch: () => Promise<void>;
}

export const useActivityStore = create<ActivityStore>((set) => ({
  items: [],
  loading: false,
  error: null,

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const items = await activityList();
      set({ items, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },
}));
