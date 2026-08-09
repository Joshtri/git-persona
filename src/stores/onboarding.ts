import { create } from "zustand";
import { onboardingApply, onboardingScan, onboardingSkip } from "@/ipc";
import { toAppError } from "@/ipc/errors";
import type { SystemScan } from "@/ipc/types.gen";
import { useActivityStore } from "@/stores/activity";
import { useFeedbackStore } from "@/stores/feedback";
import { useProfilesStore } from "@/stores/profiles";
import { useSettingsStore } from "@/stores/settings";
import { useSshStore } from "@/stores/ssh";

interface OnboardingStore {
  open: boolean;
  loading: boolean;
  applying: boolean;
  scan: SystemScan | null;
  createProfile: boolean;
  label: string;
  name: string;
  email: string;
  signingKey: string;
  color: string;
  selectedSsh: string[];
  setCreateProfile: (value: boolean) => void;
  setField: (key: "label" | "name" | "email" | "signingKey" | "color", value: string) => void;
  toggleSsh: (path: string) => void;
  openIfNeeded: () => Promise<void>;
  apply: () => Promise<void>;
  skip: () => Promise<void>;
  close: () => void;
}

export const useOnboardingStore = create<OnboardingStore>((set, get) => ({
  open: false,
  loading: false,
  applying: false,
  scan: null,
  createProfile: false,
  label: "",
  name: "",
  email: "",
  signingKey: "",
  color: "",
  selectedSsh: [],

  setCreateProfile: (value) => set({ createProfile: value }),
  setField: (key, value) => set({ [key]: value } as Partial<OnboardingStore>),
  toggleSsh: (path) =>
    set((s) => ({
      selectedSsh: s.selectedSsh.includes(path)
        ? s.selectedSsh.filter((p) => p !== path)
        : [...s.selectedSsh, path],
    })),

  // Called once on launch. Scans the system and opens the wizard only for a
  // first-run user (no completed onboarding and no existing profiles).
  openIfNeeded: async () => {
    set({ loading: true });
    try {
      const scan = await onboardingScan();
      if (scan.already_onboarded) {
        set({ loading: false, open: false, scan });
        return;
      }
      const id = scan.identity;
      set({
        loading: false,
        open: true,
        scan,
        createProfile: id !== null,
        name: id?.name ?? "",
        email: id?.email ?? "",
        signingKey: id?.signing_key ?? "",
        label: id?.email ?? id?.name ?? "",
        color: "",
        selectedSsh: scan.ssh_keys.map((k) => k.private_key_path),
      });
    } catch {
      // Detection failure must never block app launch — fail silent.
      set({ loading: false, open: false });
    }
  },

  apply: async () => {
    const s = get();
    if (s.createProfile && (s.name.trim() === "" || s.email.trim() === "")) {
      useFeedbackStore.getState().toast("Name and email are required to create a profile", "error");
      return;
    }
    set({ applying: true });
    try {
      await onboardingApply(
        s.createProfile,
        s.label.trim() || s.email.trim(),
        s.name.trim(),
        s.email.trim(),
        s.signingKey.trim() || null,
        s.color.trim() || null,
        s.selectedSsh
      );
      set({ applying: false, open: false });
      useFeedbackStore.getState().toast("Setup imported from your system", "success");
      useProfilesStore.getState().fetch();
      useSshStore.getState().fetch();
      useSettingsStore.getState().fetch();
      useActivityStore.getState().fetch();
    } catch (e) {
      set({ applying: false });
      useFeedbackStore.getState().toast(toAppError(e).message, "error");
    }
  },

  skip: async () => {
    try {
      await onboardingSkip();
      useSettingsStore.getState().fetch();
    } catch {
      // Skipping is best-effort; closing the wizard is what matters to the user.
    }
    set({ open: false });
  },

  close: () => set({ open: false }),
}));
