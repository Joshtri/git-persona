import { create } from "zustand";

export type ViewName =
  | "dashboard"
  | "profiles"
  | "repos"
  | "rules"
  | "ssh"
  | "credentials"
  | "activity"
  | "settings"
  | "about"
  | "onboarding";

export type View =
  | { name: "dashboard" }
  | { name: "profiles" }
  | { name: "repos" }
  | { name: "rules" }
  | { name: "ssh" }
  | { name: "credentials" }
  | { name: "activity" }
  | { name: "settings" }
  | { name: "about" }
  | { name: "onboarding" };

type NavGuard = (view: View) => boolean;

interface ViewStore {
  current: View;
  paletteOpen: boolean;
  blockedNav: View | null;
  guard: NavGuard | null;
  navigate: (view: View) => void;
  setGuard: (guard: NavGuard | null) => void;
  confirmNav: () => void;
  cancelNav: () => void;
  openPalette: () => void;
  closePalette: () => void;
}

export const useViewStore = create<ViewStore>((set, get) => ({
  current: { name: "dashboard" },
  paletteOpen: false,
  blockedNav: null,
  guard: null,
  navigate: (view) => {
    const guard = get().guard;
    if (guard && !guard(view)) {
      set({ blockedNav: view });
      return;
    }
    set({ current: view, paletteOpen: false, blockedNav: null });
  },
  setGuard: (guard) => set({ guard }),
  confirmNav: () => {
    const { blockedNav } = get();
    if (blockedNav) set({ current: blockedNav, paletteOpen: false, blockedNav: null });
  },
  cancelNav: () => set({ blockedNav: null }),
  openPalette: () => set({ paletteOpen: true }),
  closePalette: () => set({ paletteOpen: false }),
}));
