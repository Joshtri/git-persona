import { create } from "zustand";
import { activityList, activityPurge } from "@/ipc/client";
import { type AppError, toAppError } from "@/ipc/errors";
import type { AuditEntry } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";

interface ActivityStore {
  items: AuditEntry[];
  loading: boolean;
  error: AppError | null;
  fetch: () => Promise<void>;
  purge: (days: number) => Promise<void>;
}

export const useActivityStore = create<ActivityStore>((set, get) => ({
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

  purge: async (days) => {
    try {
      const removed = await activityPurge(days);
      await get().fetch();
      const msg = removed === 0 ? "Nothing to clear" : `Cleared ${removed} entries`;
      useFeedbackStore.getState().toast(msg, removed === 0 ? "info" : "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
