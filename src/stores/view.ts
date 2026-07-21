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

interface ViewStore {
  current: View;
  paletteOpen: boolean;
  navigate: (view: View) => void;
  openPalette: () => void;
  closePalette: () => void;
}

export const useViewStore = create<ViewStore>((set) => ({
  current: { name: "dashboard" },
  paletteOpen: false,
  navigate: (view) => set({ current: view, paletteOpen: false }),
  openPalette: () => set({ paletteOpen: true }),
  closePalette: () => set({ paletteOpen: false }),
}));
