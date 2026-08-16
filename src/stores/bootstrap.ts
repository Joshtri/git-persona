import { create } from "zustand";
import { bootstrapFetch } from "@/ipc/client";
import type { AnnouncementInfo, BootstrapData } from "@/ipc/types.gen";

interface BootstrapStore {
  announcements: AnnouncementInfo[];
  featureFlags: Record<string, boolean>;
  dismissedIds: Set<string>;
  forceUpdate: boolean;
  fetch: () => Promise<void>;
  dismissAnnouncement: (id: string) => void;
}

export const useBootstrapStore = create<BootstrapStore>((set, get) => ({
  announcements: [],
  featureFlags: {},
  dismissedIds: new Set(),
  forceUpdate: false,

  fetch: async () => {
    let data: BootstrapData;
    try {
      data = await bootstrapFetch();
    } catch {
      // Server unavailable — app starts normally. No toast, no alarm.
      return;
    }

    set({
      announcements: data.announcements,
      featureFlags: data.feature_flags,
      forceUpdate: data.force_update ?? false,
    });
  },

  dismissAnnouncement: (id) => {
    const dismissed = new Set(get().dismissedIds);
    dismissed.add(id);
    set({ dismissedIds: dismissed });
  },
}));
