import { create } from "zustand";
import {
  profileApply,
  profileCreate,
  profileDelete,
  profileGetActive,
  profileList,
  profileUpdate,
} from "@/ipc/client";
import { type AppError, toAppError } from "@/ipc/errors";
import type { Profile } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";

interface ProfilesStore {
  items: Profile[];
  activeProfileId: string | null;
  loading: boolean;
  error: AppError | null;
  fetch: () => Promise<void>;
  create: (
    label: string,
    name: string,
    email: string,
    signing_key?: string | null,
    color?: string | null
  ) => Promise<boolean>;
  update: (
    id: string,
    label: string,
    name: string,
    email: string,
    signing_key?: string | null,
    color?: string | null
  ) => Promise<boolean>;
  delete: (id: string) => Promise<boolean>;
  apply: (id: string) => Promise<boolean>;
}

export const useProfilesStore = create<ProfilesStore>((set, get) => ({
  items: [],
  activeProfileId: null,
  loading: false,
  error: null,

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const [items, active] = await Promise.all([profileList(), profileGetActive()]);
      set({ items, activeProfileId: active?.id ?? null, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  create: async (label, name, email, signing_key, color) => {
    try {
      await profileCreate(label, name, email, signing_key, color);
      await get().fetch();
      useFeedbackStore.getState().toast("Profile created", "success");
      return true;
    } catch (e) {
      const err = toAppError(e);
      useFeedbackStore.getState().toast(err.message, "error");
      return false;
    }
  },

  update: async (id, label, name, email, signing_key, color) => {
    try {
      await profileUpdate(id, label, name, email, signing_key, color);
      await get().fetch();
      useFeedbackStore.getState().toast("Profile updated", "success");
      return true;
    } catch (e) {
      const err = toAppError(e);
      useFeedbackStore.getState().toast(err.message, "error");
      return false;
    }
  },

  delete: async (id) => {
    try {
      await profileDelete(id);
      await get().fetch();
      useFeedbackStore.getState().toast("Profile deleted", "success");
      return true;
    } catch (e) {
      const err = toAppError(e);
      useFeedbackStore.getState().toast(err.message, "error");
      return false;
    }
  },

  apply: async (id) => {
    try {
      await profileApply(id);
      set({ activeProfileId: id });
      useFeedbackStore.getState().toast("Profile applied globally", "success");
      return true;
    } catch (e) {
      const err = toAppError(e);
      useFeedbackStore.getState().toast(err.message, "error");
      return false;
    }
  },
}));
