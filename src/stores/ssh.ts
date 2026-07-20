import { create } from "zustand";
import {
  sshAssignProfile,
  sshConfigOpen,
  sshGenerate,
  sshImport,
  sshList,
  sshRemove,
  sshReveal,
  sshRevealFolder,
  sshScan,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { SshAlgorithm, SshKey } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";

export interface GenerateInput {
  label: string;
  algorithm: SshAlgorithm;
  comment: string | null;
  outputDir: string;
  fileName: string;
  hostAlias: string | null;
  hostName: string | null;
}

interface SshStore {
  items: SshKey[];
  loading: boolean;
  error: AppError | null;
  importOpen: boolean;
  generateOpen: boolean;
  setImportOpen: (open: boolean) => void;
  setGenerateOpen: (open: boolean) => void;
  fetch: () => Promise<void>;
  scan: () => Promise<void>;
  importKey: (
    path: string,
    hostAlias?: string | null,
    hostName?: string | null
  ) => Promise<boolean>;
  generate: (input: GenerateInput) => Promise<boolean>;
  remove: (id: string) => Promise<boolean>;
  reveal: (id: string) => Promise<void>;
  revealFolder: () => Promise<void>;
  openConfig: () => Promise<void>;
  assignProfile: (id: string, profileId: string | null) => Promise<void>;
}

export const useSshStore = create<SshStore>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  importOpen: false,
  generateOpen: false,

  setImportOpen: (open) => set({ importOpen: open }),
  setGenerateOpen: (open) => set({ generateOpen: open }),

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const items = await sshList();
      set({ items, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  scan: async () => {
    set({ loading: true, error: null });
    try {
      const items = await sshScan();
      set({ items, loading: false });
    } catch (e) {
      const err = toAppError(e);
      set({ error: err, loading: false });
      useFeedbackStore.getState().toast(err.message, "error");
    }
  },

  importKey: async (path, hostAlias, hostName) => {
    try {
      await sshImport(path, hostAlias ?? null, hostName ?? null);
      await get().fetch();
      useFeedbackStore.getState().toast("SSH key imported", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  generate: async (input) => {
    try {
      await sshGenerate(
        input.label,
        input.algorithm,
        input.comment,
        input.outputDir,
        input.fileName,
        input.hostAlias,
        input.hostName
      );
      await get().fetch();
      useFeedbackStore.getState().toast("SSH key generated", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  remove: async (id) => {
    try {
      await sshRemove(id);
      await get().fetch();
      useFeedbackStore.getState().toast("SSH key removed", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  reveal: async (id) => {
    try {
      await sshReveal(id);
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  revealFolder: async () => {
    try {
      await sshRevealFolder();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  openConfig: async () => {
    try {
      await sshConfigOpen();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  assignProfile: async (id, profileId) => {
    try {
      await sshAssignProfile(id, profileId);
      await get().fetch();
      useFeedbackStore
        .getState()
        .toast(profileId ? "Key assigned to profile" : "Key unassigned", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
