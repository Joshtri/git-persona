import { create } from "zustand";
import { bootstrapFetch } from "@/ipc/client";
import type { AnnouncementInfo, BootstrapData, UpdateInfo } from "@/ipc/types.gen";

interface BootstrapStore {
  update: UpdateInfo | null;
  announcements: AnnouncementInfo[];
  featureFlags: Record<string, boolean>;
  dismissedIds: Set<string>;
  updateDismissed: boolean;
  fetch: () => Promise<void>;
  dismissAnnouncement: (id: string) => void;
  dismissUpdate: () => void;
}

export const useBootstrapStore = create<BootstrapStore>((set, get) => ({
  update: null,
  announcements: [],
  featureFlags: {},
  dismissedIds: new Set(),
  updateDismissed: false,

  fetch: async () => {
    let data: BootstrapData;
    try {
      data = await bootstrapFetch();
    } catch {
      // Server unavailable — app starts normally. No toast, no alarm.
      return;
    }

    set({
      update: data.update,
      announcements: data.announcements,
      featureFlags: data.feature_flags,
    });
  },

  dismissAnnouncement: (id) => {
    const dismissed = new Set(get().dismissedIds);
    dismissed.add(id);
    set({ dismissedIds: dismissed });
  },

  dismissUpdate: () => set({ updateDismissed: true }),
}));
