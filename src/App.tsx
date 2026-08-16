import { useEffect } from "react";
import { AppShell } from "@/components/layout/AppShell";
import { useBlockReload } from "@/hooks/useBlockReload";
import { useActivityStore } from "@/stores/activity";
import { useBootstrapStore } from "@/stores/bootstrap";
import { useOnboardingStore } from "@/stores/onboarding";
import { useProfilesStore } from "@/stores/profiles";
import { useReposStore } from "@/stores/repos";
import { useSettingsStore } from "@/stores/settings";
import { useSmartSwitchStore } from "@/stores/smartSwitch";
import "@/styles/index.css";

function resolveTheme(preference: string | undefined, systemDark: boolean): "dark" | "light" {
  if (preference === "Light") return "light";
  if (preference === "System") return systemDark ? "dark" : "light";
  return "dark";
}

export function App() {
  const { data: settings, fetch: fetchSettings } = useSettingsStore();

  useBlockReload();

  useEffect(() => {
    fetchSettings();
    useProfilesStore.getState().fetch();
    useActivityStore.getState().fetch();
    useReposStore.getState().fetch();
    // Subscribe to Smart Switching events + load its status once, app-wide, so
    // auto-switch toasts and confirmations work regardless of the active view.
    useSmartSwitchStore.getState().init();
    // First-run detection: opens the onboarding wizard only when the user hasn't
    // been onboarded yet (checked backend-side). Failure never blocks launch.
    useOnboardingStore.getState().openIfNeeded();
    // Fetch server-controlled state (updates, announcements, feature flags).
    // Best-effort: a network failure or offline state is silently swallowed.
    useBootstrapStore.getState().fetch();
  }, [fetchSettings]);

  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");

    const apply = () => {
      document.documentElement.dataset.theme = resolveTheme(settings?.theme, mql.matches);
    };

    apply();

    if (settings?.theme === "System") {
      mql.addEventListener("change", apply);
      return () => mql.removeEventListener("change", apply);
    }
  }, [settings?.theme]);

  return <AppShell />;
}
