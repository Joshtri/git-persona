import { create } from "zustand";

export type ToastVariant = "success" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  variant: ToastVariant;
}

interface UndoEntry {
  label: string;
  perform: () => Promise<void>;
}

interface FeedbackStore {
  toasts: Toast[];
  undo: UndoEntry | null;
  toast: (message: string, variant?: ToastVariant) => void;
  dismiss: (id: string) => void;
  setUndo: (entry: UndoEntry | null) => void;
  performUndo: () => Promise<void>;
}

let nextId = 0;

export const useFeedbackStore = create<FeedbackStore>((set, get) => ({
  toasts: [],
  undo: null,

  toast: (message, variant = "info") => {
    const id = String(nextId++);
    set((s) => ({ toasts: [...s.toasts, { id, message, variant }] }));
    setTimeout(() => get().dismiss(id), 4000);
  },

  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),

  setUndo: (entry) => set({ undo: entry }),

  performUndo: async () => {
    const { undo } = get();
    if (!undo) return;
    set({ undo: null });
    await undo.perform();
  },
}));
