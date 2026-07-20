import { create } from "zustand";
import {
  onSmartSwitch,
  onSmartSwitchStatus,
  smartSwitchCancel,
  smartSwitchConfirm,
  smartSwitchPause,
  smartSwitchRestart,
  smartSwitchResume,
  smartSwitchSetEnabled,
  smartSwitchStatus,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { SmartSwitchingSettings, SmartSwitchStatus, SwitchEvent } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";
import { useProfilesStore } from "@/stores/profiles";
import { useSettingsStore } from "@/stores/settings";

interface SmartSwitchStore {
  status: SmartSwitchStatus | null;
  pending: SwitchEvent | null;
  loading: boolean;
  error: AppError | null;
  initialized: boolean;
  init: () => Promise<void>;
  fetchStatus: () => Promise<void>;
  setEnabled: (enabled: boolean) => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  restart: () => Promise<void>;
  refreshWatch: () => Promise<void>;
  confirmPending: () => Promise<void>;
  dismissPending: () => Promise<void>;
  patchSettings: (partial: Partial<SmartSwitchingSettings>) => Promise<void>;
}

export const useSmartSwitchStore = create<SmartSwitchStore>((set, get) => ({
  status: null,
  pending: null,
  loading: false,
  error: null,
  initialized: false,

  init: async () => {
    if (get().initialized) return;
    set({ initialized: true });
    await onSmartSwitchStatus((status) => set({ status }));
    await onSmartSwitch((event) => {
      if (event.status === "PendingConfirmation") {
        set({ pending: event });
        return;
      }
      if (event.status === "Switched") {
        const notify = useSettingsStore.getState().data?.smart_switching.show_notification ?? true;
        if (notify) {
          const label = event.profile_label ?? "a profile";
          const repo = event.repo_name ?? "repository";
          useFeedbackStore.getState().toast(`Switched to ${label} for ${repo}`, "success");
        }
        useProfilesStore.getState().fetch();
        useActivityStore.getState().fetch();
      }
    });
    await get().fetchStatus();
  },

  fetchStatus: async () => {
    set({ loading: true, error: null });
    try {
      const status = await smartSwitchStatus();
      set({ status, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  setEnabled: async (enabled) => {
    try {
      const status = await smartSwitchSetEnabled(enabled);
      set({ status });
      // The command persisted the flag backend-side; refresh the settings cache
      // so the Settings view and any later save stay in sync.
      await useSettingsStore.getState().fetch();
      useFeedbackStore
        .getState()
        .toast(enabled ? "Smart Switching enabled" : "Smart Switching disabled", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  pause: async () => {
    try {
      set({ status: await smartSwitchPause() });
      useFeedbackStore.getState().toast("Watcher paused", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  resume: async () => {
    try {
      set({ status: await smartSwitchResume() });
      useFeedbackStore.getState().toast("Watcher resumed", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  restart: async () => {
    try {
      set({ status: await smartSwitchRestart() });
      useFeedbackStore.getState().toast("Watcher restarted", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  refreshWatch: async () => {
    // Silently rebuild the watch set (e.g. after a scan added repositories).
    // No-op when Smart Switching is off.
    if (!get().status?.enabled) return;
    try {
      set({ status: await smartSwitchRestart() });
    } catch {
      // Non-fatal — the next explicit action will reconcile.
    }
  },

  confirmPending: async () => {
    const pending = get().pending;
    if (!pending) return;
    try {
      await smartSwitchConfirm(pending.git_root);
      set({ pending: null });
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  dismissPending: async () => {
    const pending = get().pending;
    set({ pending: null });
    if (!pending) return;
    try {
      await smartSwitchCancel(pending.git_root);
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  patchSettings: async (partial) => {
    const settings = useSettingsStore.getState().data;
    if (!settings) return;
    const next = {
      ...settings,
      smart_switching: { ...settings.smart_switching, ...partial },
    };
    await useSettingsStore.getState().update(next);
    await get().fetchStatus();
  },
}));
