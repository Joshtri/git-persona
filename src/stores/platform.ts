import { create } from "zustand";
import { platformCapabilities } from "@/ipc/client";
import type { PlatformCapabilities } from "@/ipc/types.gen";

interface PlatformStore {
  capabilities: PlatformCapabilities | null;
  fetch: () => Promise<void>;
}

/**
 * The platform capability model, fetched once at startup. `capabilities` is
 * `null` only during that brief window; capability-gated UI treats an absent
 * capability as unavailable, so a guaranteed-to-fail action is never shown even
 * before the first fetch resolves.
 */
export const usePlatformStore = create<PlatformStore>((set) => ({
  capabilities: null,

  fetch: async () => {
    try {
      const capabilities = await platformCapabilities();
      set({ capabilities });
    } catch {
      // The command is pure and state-free; a failure here is effectively
      // impossible. Leave capabilities null (everything gated off) rather than
      // block launch.
    }
  },
}));
