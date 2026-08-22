import { create } from "zustand";
import {
  commitGuardInstall,
  commitGuardRepair,
  commitGuardSetAutoProtect,
  commitGuardSetEnabled,
  commitGuardSetMode,
  commitGuardStatus,
  commitGuardUninstall,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { CommitGuardStatus, GuardMode } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";
import { useSettingsStore } from "@/stores/settings";

interface CommitGuardStore {
  status: CommitGuardStatus | null;
  loading: boolean;
  error: AppError | null;
  initialized: boolean;
  init: () => Promise<void>;
  fetchStatus: () => Promise<void>;
  setEnabled: (enabled: boolean) => Promise<void>;
  setMode: (mode: GuardMode) => Promise<void>;
  setAutoProtect: (enabled: boolean) => Promise<void>;
  install: (repoId: string) => Promise<void>;
  repair: (repoId: string) => Promise<void>;
  uninstall: (repoId: string) => Promise<void>;
}

/** Commit Guard is the final safety layer: it verifies the active Git identity
 *  at commit time via a managed `pre-commit` hook. This store mirrors the
 *  backend `CommitGuardStatus` and drives the settings + per-repository UI. It is
 *  distinct from Smart Switching (proactive identity application). */
export const useCommitGuardStore = create<CommitGuardStore>((set, get) => ({
  status: null,
  loading: false,
  error: null,
  initialized: false,

  init: async () => {
    if (get().initialized) return;
    set({ initialized: true });
    await get().fetchStatus();
  },

  fetchStatus: async () => {
    set({ loading: true, error: null });
    try {
      const status = await commitGuardStatus();
      set({ status, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  setEnabled: async (enabled) => {
    try {
      const status = await commitGuardSetEnabled(enabled);
      set({ status });
      // The command persisted the flag backend-side; refresh the settings cache
      // so the Settings view stays in sync.
      await useSettingsStore.getState().fetch();
      useActivityStore.getState().fetch();
      useFeedbackStore
        .getState()
        .toast(enabled ? "Commit Guard enabled" : "Commit Guard disabled", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setMode: async (mode) => {
    try {
      const status = await commitGuardSetMode(mode);
      set({ status });
      await useSettingsStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setAutoProtect: async (enabled) => {
    try {
      const status = await commitGuardSetAutoProtect(enabled);
      set({ status });
      await useSettingsStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  install: async (repoId) => {
    try {
      set({ status: await commitGuardInstall(repoId) });
      useActivityStore.getState().fetch();
      useFeedbackStore.getState().toast("Commit Guard installed", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  repair: async (repoId) => {
    try {
      set({ status: await commitGuardRepair(repoId) });
      useFeedbackStore.getState().toast("Commit Guard hook repaired", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  uninstall: async (repoId) => {
    try {
      set({ status: await commitGuardUninstall(repoId) });
      useActivityStore.getState().fetch();
      useFeedbackStore.getState().toast("Commit Guard removed", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
