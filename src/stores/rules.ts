import { create } from "zustand";
import {
  ruleCreate,
  ruleDelete,
  ruleDuplicate,
  ruleExport,
  ruleImport,
  ruleList,
  rulePreview,
  ruleReorder,
  ruleSetAllEnabled,
  ruleSetEnabled,
  ruleSummary,
  ruleUpdate,
} from "@/ipc";
import { pickRulesExportPath, pickRulesImportFile } from "@/ipc/dialog";
import { type AppError, toAppError } from "@/ipc/errors";
import type {
  Rule,
  RuleOperator,
  RulePreviewResult,
  RuleSubject,
  RuleSummary,
} from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";

export interface RuleInput {
  name: string;
  subject: RuleSubject;
  operator: RuleOperator;
  value: string;
  targetProfileId: string;
}

export interface PreviewInput {
  path: string;
  name: string;
  remoteUrl: string;
}

interface RulesStore {
  items: Rule[];
  loading: boolean;
  error: AppError | null;
  createOpen: boolean;
  previewOpen: boolean;
  editing: Rule | null;
  preview: RulePreviewResult | null;
  summary: RuleSummary | null;
  setCreateOpen: (open: boolean) => void;
  setPreviewOpen: (open: boolean) => void;
  setEditing: (rule: Rule | null) => void;
  fetch: () => Promise<void>;
  fetchSummary: () => Promise<void>;
  create: (input: RuleInput) => Promise<boolean>;
  update: (id: string, input: RuleInput) => Promise<boolean>;
  remove: (id: string) => Promise<boolean>;
  duplicate: (id: string) => Promise<void>;
  setEnabled: (id: string, enabled: boolean) => Promise<void>;
  setAllEnabled: (enabled: boolean) => Promise<void>;
  move: (id: string, direction: "up" | "down") => Promise<void>;
  runPreview: (input: PreviewInput) => Promise<void>;
  exportRules: () => Promise<void>;
  importRules: (replace: boolean) => Promise<void>;
}

export const useRulesStore = create<RulesStore>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  createOpen: false,
  previewOpen: false,
  editing: null,
  preview: null,
  summary: null,

  setCreateOpen: (open) => set({ createOpen: open }),
  setPreviewOpen: (open) => set({ previewOpen: open, preview: open ? get().preview : null }),
  setEditing: (rule) => set({ editing: rule }),

  fetch: async () => {
    set({ loading: true, error: null });
    try {
      const items = await ruleList();
      set({ items, loading: false });
    } catch (e) {
      set({ error: toAppError(e), loading: false });
    }
  },

  fetchSummary: async () => {
    try {
      const summary = await ruleSummary();
      set({ summary });
    } catch {
      // The dashboard widget is non-critical; ignore transient failures.
    }
  },

  create: async (input) => {
    try {
      await ruleCreate(
        input.name,
        input.subject,
        input.operator,
        input.value,
        input.targetProfileId
      );
      await get().fetch();
      useFeedbackStore.getState().toast("Rule created", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  update: async (id, input) => {
    try {
      await ruleUpdate(
        id,
        input.name,
        input.subject,
        input.operator,
        input.value,
        input.targetProfileId
      );
      await get().fetch();
      useFeedbackStore.getState().toast("Rule updated", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  remove: async (id) => {
    try {
      await ruleDelete(id);
      await get().fetch();
      useFeedbackStore.getState().toast("Rule deleted", "success");
      useActivityStore.getState().fetch();
      return true;
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
      return false;
    }
  },

  duplicate: async (id) => {
    try {
      await ruleDuplicate(id);
      await get().fetch();
      useFeedbackStore.getState().toast("Rule duplicated", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setEnabled: async (id, enabled) => {
    try {
      await ruleSetEnabled(id, enabled);
      await get().fetch();
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  setAllEnabled: async (enabled) => {
    try {
      await ruleSetAllEnabled(enabled);
      await get().fetch();
      useFeedbackStore
        .getState()
        .toast(enabled ? "All rules enabled" : "All rules disabled", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  move: async (id, direction) => {
    // Rules are already priority-ordered; swap this rule with its neighbour and
    // persist the new order.
    const ordered = [...get().items];
    const index = ordered.findIndex((r) => r.id === id);
    if (index === -1) return;
    const target = direction === "up" ? index - 1 : index + 1;
    if (target < 0 || target >= ordered.length) return;
    const swapped = ordered[target];
    const current = ordered[index];
    if (!swapped || !current) return;
    ordered[target] = current;
    ordered[index] = swapped;
    try {
      await ruleReorder(ordered.map((r) => r.id));
      await get().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  runPreview: async (input) => {
    try {
      const result = await rulePreview({
        path: input.path,
        name: input.name,
        remote_url: input.remoteUrl.trim() ? input.remoteUrl.trim() : null,
      });
      set({ preview: result });
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  exportRules: async () => {
    try {
      const path = await pickRulesExportPath();
      if (!path) return;
      await ruleExport(path);
      useFeedbackStore.getState().toast("Rules exported", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  importRules: async (replace) => {
    try {
      const path = await pickRulesImportFile();
      if (!path) return;
      await ruleImport(path, replace);
      await get().fetch();
      useFeedbackStore.getState().toast("Rules imported", "success");
      useActivityStore.getState().fetch();
    } catch (e) {
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },
}));
