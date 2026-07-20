import { create } from "zustand";
import {
  onRepoScanProgress,
  pickFolders,
  repoAssignProfile,
  repoList,
  repoRefresh,
  repoRemove,
  repoReveal,
  repoScan,
  repoToggleFavorite,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { Repo, ScanProgress } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";
import { useSmartSwitchStore } from "@/stores/smartSwitch";

interface ReposStore {
  items: Repo[];
  loading: boolean;
  scanning: boolean;
  progress: ScanProgress | null;
  error: AppError | null;
  fetch: () => Promise<void>;
  scan: () => Promise<void>;
  refresh: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  assignProfile: (id: string, profileId: string | null) => Promise<void>;
  unassignAll: (profileId: string) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  reveal: (id: string) => Promise<void>;
}

function replaceItem(items: Repo[], repo: Repo): Repo[] {
  return items.map((r) => (r.id === repo.id ? repo : r));
}

export const useReposStore = create<ReposStore>((set, get) => ({
  items: [],
  loading: false,
  scanning: false,
  progress: null,
  error: null,

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const items = await repoList();
      set({ items, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  scan: async () => {
    const feedback = useFeedbackStore.getState();
    let paths: string[];
    try {
      paths = await pickFolders();
    } catch (e) {
      feedback.toast(toAppError(e).message, "error");
      return;
    }
    if (paths.length === 0) return;

    set({ scanning: true, error: null, progress: null });
    const unlisten = await onRepoScanProgress((progress) => set({ progress }));
    try {
      const items = await repoScan(paths);
      set({ items, scanning: false, progress: null });
      feedback.toast(`Scan complete — ${items.length} repositories`, "success");
      useActivityStore.getState().fetch();
      // Newly discovered repositories should join the watch set.
      useSmartSwitchStore.getState().refreshWatch();
    } catch (e) {
      const err = toAppError(e);
      set({ error: err, scanning: false, progress: null });
      feedback.toast(err.message, "error");
    } finally {
      unlisten();
    }
  },

  refresh: async (id) => {
    try {
      const repo = await repoRefresh(id);
      set({ items: replaceItem(get().items, repo) });
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  remove: async (id) => {
    try {
      await repoRemove(id);
      set({ items: get().items.filter((r) => r.id !== id) });
      useFeedbackStore.getState().toast("Repository removed", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  assignProfile: async (id, profileId) => {
    try {
      const repo = await repoAssignProfile(id, profileId);
      set({ items: replaceItem(get().items, repo) });
      useFeedbackStore
        .getState()
        .toast(profileId ? "Profile assigned" : "Profile unassigned", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  unassignAll: async (profileId) => {
    const affected = get().items.filter((r) => r.active_profile_id === profileId);
    if (affected.length === 0) return;
    try {
      for (const r of affected) {
        await repoAssignProfile(r.id, null);
      }
      await get().fetch();
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  toggleFavorite: async (id) => {
    try {
      const repo = await repoToggleFavorite(id);
      set({ items: replaceItem(get().items, repo) });
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  reveal: async (id) => {
    try {
      await repoReveal(id);
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
