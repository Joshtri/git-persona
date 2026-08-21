import { create } from "zustand";
import {
  onRepoScanProgress,
  pickFolders,
  repoAssignProfile,
  repoGroupCreate,
  repoGroupDelete,
  repoGroupList,
  repoGroupUpdate,
  repoList,
  repoRefresh,
  repoRemove,
  repoReveal,
  repoScan,
  repoSetGroup,
  repoSetGroupMany,
  repoToggleFavorite,
} from "@/ipc";
import { type AppError, toAppError } from "@/ipc/errors";
import type { Repo, RepoGroup, ScanProgress } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";
import { useSmartSwitchStore } from "@/stores/smartSwitch";

export interface GroupDialogState {
  editing: RepoGroup | null;
  // When set, the repo(s) to move into the group right after it is created.
  assignRepoIds: string[] | null;
}

interface ReposStore {
  items: Repo[];
  groups: RepoGroup[];
  loading: boolean;
  scanning: boolean;
  progress: ScanProgress | null;
  error: AppError | null;
  groupDialog: GroupDialogState | null;
  openGroupDialog: (state: GroupDialogState) => void;
  closeGroupDialog: () => void;
  fetch: () => Promise<void>;
  scan: () => Promise<void>;
  scanIntoGroup: (groupId: string) => Promise<void>;
  quickScanGroup: (groupId: string) => Promise<void>;
  refresh: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  assignProfile: (id: string, profileId: string | null) => Promise<void>;
  unassignAll: (profileId: string) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  reveal: (id: string) => Promise<void>;
  createGroup: (name: string, color: string | null) => Promise<string | null>;
  updateGroup: (id: string, name: string, color: string | null) => Promise<boolean>;
  deleteGroup: (id: string) => Promise<void>;
  setGroup: (id: string, groupId: string | null) => Promise<void>;
  setGroupMany: (ids: string[], groupId: string | null) => Promise<void>;
}

function replaceItem(items: Repo[], repo: Repo): Repo[] {
  return items.map((r) => (r.id === repo.id ? repo : r));
}

// Parent directory of an absolute path, handling both Windows and POSIX
// separators so a group's repos can be re-scanned from where they live.
function parentDir(path: string): string {
  const idx = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return idx <= 0 ? path : path.slice(0, idx);
}

export const useReposStore = create<ReposStore>((set, get) => ({
  items: [],
  groups: [],
  loading: false,
  scanning: false,
  progress: null,
  error: null,
  groupDialog: null,

  openGroupDialog: (state) => set({ groupDialog: state }),
  closeGroupDialog: () => set({ groupDialog: null }),

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const [items, groups] = await Promise.all([repoList(), repoGroupList()]);
      set({ items, groups, loading: false });
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

  scanIntoGroup: async (groupId) => {
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
      const before = new Set(get().items.map((r) => r.git_root));
      const items = await repoScan(paths);
      const fresh = items.filter((r) => !before.has(r.git_root));
      if (fresh.length > 0) {
        const updated = await repoSetGroupMany(
          fresh.map((r) => r.id),
          groupId
        );
        const byId = new Map(updated.map((r) => [r.id, r]));
        set({ items: items.map((r) => byId.get(r.id) ?? r), scanning: false, progress: null });
      } else {
        set({ items, scanning: false, progress: null });
      }
      feedback.toast(
        fresh.length > 0
          ? `Scan complete — ${fresh.length} repositories added to group`
          : "Scan complete — no new repositories",
        "success"
      );
      useActivityStore.getState().fetch();
      useSmartSwitchStore.getState().refreshWatch();
    } catch (e) {
      const err = toAppError(e);
      set({ error: err, scanning: false, progress: null });
      feedback.toast(err.message, "error");
    } finally {
      unlisten();
    }
  },

  quickScanGroup: async (groupId) => {
    const feedback = useFeedbackStore.getState();
    const groupRepos = get().items.filter((r) => r.group_id === groupId);
    if (groupRepos.length === 0) {
      feedback.toast(
        'No repositories in this group yet — use "Scan repos into this group" first',
        "info"
      );
      return;
    }
    const dirs = [...new Set(groupRepos.map((r) => parentDir(r.git_root)))];

    set({ scanning: true, error: null, progress: null });
    const unlisten = await onRepoScanProgress((progress) => set({ progress }));
    try {
      const before = new Set(get().items.map((r) => r.git_root));
      const items = await repoScan(dirs);
      const fresh = items.filter((r) => !before.has(r.git_root));
      if (fresh.length > 0) {
        const updated = await repoSetGroupMany(
          fresh.map((r) => r.id),
          groupId
        );
        const byId = new Map(updated.map((r) => [r.id, r]));
        set({ items: items.map((r) => byId.get(r.id) ?? r), scanning: false, progress: null });
      } else {
        set({ items, scanning: false, progress: null });
      }
      feedback.toast(
        fresh.length > 0
          ? `Quick scan complete — ${fresh.length} added to group`
          : "Quick scan complete — no new repositories",
        "success"
      );
      useActivityStore.getState().fetch();
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

  createGroup: async (name, color) => {
    try {
      const group = await repoGroupCreate(name, color);
      set({ groups: [...get().groups, group] });
      useFeedbackStore.getState().toast(`Group "${group.name}" created`, "success");
      return group.id;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return null;
    }
  },

  updateGroup: async (id, name, color) => {
    try {
      const group = await repoGroupUpdate(id, name, color);
      set({ groups: get().groups.map((g) => (g.id === group.id ? group : g)) });
      useFeedbackStore.getState().toast("Group updated", "success");
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  deleteGroup: async (id) => {
    try {
      await repoGroupDelete(id);
      // Repos detach from the group on the backend; mirror that locally.
      set({
        groups: get().groups.filter((g) => g.id !== id),
        items: get().items.map((r) => (r.group_id === id ? { ...r, group_id: null } : r)),
      });
      useFeedbackStore.getState().toast("Group deleted", "success");
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setGroup: async (id, groupId) => {
    try {
      const repo = await repoSetGroup(id, groupId);
      set({ items: replaceItem(get().items, repo) });
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setGroupMany: async (ids, groupId) => {
    if (ids.length === 0) return;
    try {
      const updated = await repoSetGroupMany(ids, groupId);
      const byId = new Map(updated.map((r) => [r.id, r]));
      set({ items: get().items.map((r) => byId.get(r.id) ?? r) });
      useFeedbackStore
        .getState()
        .toast(
          `Moved ${updated.length} ${updated.length === 1 ? "repository" : "repositories"}`,
          "success"
        );
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
