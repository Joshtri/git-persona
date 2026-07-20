import { create } from "zustand";
import {
  credentialAssignProfile,
  credentialCreate,
  credentialDelete,
  credentialList,
  credentialOpenManager,
  credentialSwitch,
  credentialUpdate,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { Credential } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";

export interface CreateCredentialInput {
  profileId: string | null;
  host: string;
  username: string;
  token: string;
}

interface CredentialsStore {
  items: Credential[];
  loading: boolean;
  error: AppError | null;
  createOpen: boolean;
  editing: Credential | null;
  setCreateOpen: (open: boolean) => void;
  setEditing: (credential: Credential | null) => void;
  fetch: () => Promise<void>;
  create: (input: CreateCredentialInput) => Promise<boolean>;
  update: (id: string, username: string, token: string | null) => Promise<boolean>;
  assignProfile: (id: string, profileId: string | null) => Promise<void>;
  switchCredential: (id: string) => Promise<void>;
  remove: (id: string) => Promise<boolean>;
  openManager: () => Promise<void>;
}

export const useCredentialsStore = create<CredentialsStore>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  createOpen: false,
  editing: null,

  setCreateOpen: (open) => set({ createOpen: open }),
  setEditing: (credential) => set({ editing: credential }),

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const items = await credentialList();
      set({ items, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  create: async (input) => {
    try {
      await credentialCreate(input.profileId, input.host, input.username, input.token);
      await get().fetch();
      useFeedbackStore.getState().toast("Credential created", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  update: async (id, username, token) => {
    try {
      await credentialUpdate(id, username, token);
      await get().fetch();
      useFeedbackStore.getState().toast("Credential updated", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  assignProfile: async (id, profileId) => {
    try {
      await credentialAssignProfile(id, profileId);
      await get().fetch();
      useFeedbackStore
        .getState()
        .toast(profileId ? "Credential assigned to profile" : "Credential unassigned", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  switchCredential: async (id) => {
    try {
      await credentialSwitch(id);
      await get().fetch();
      useFeedbackStore.getState().toast("Credential is now active for its host", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  remove: async (id) => {
    try {
      await credentialDelete(id);
      await get().fetch();
      useFeedbackStore.getState().toast("Credential deleted", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  openManager: async () => {
    try {
      await credentialOpenManager();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
