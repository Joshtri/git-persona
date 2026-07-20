import { AboutView } from "@/features/about";
import { ActivityView } from "@/features/activity";
import { CredentialsView } from "@/features/credentials";
import { DashboardView } from "@/features/dashboard";
import { CommandPalette } from "@/features/palette/CommandPalette";
import { ProfilesView } from "@/features/profiles";
import { ReposView } from "@/features/repos";
import { SettingsView } from "@/features/settings/SettingsView";
import { SshView } from "@/features/ssh";
import { useHotkey } from "@/hooks/useHotkey";
import { isMod } from "@/lib/keyboard";
import { useViewStore } from "@/stores/view";
import { Header } from "./Header";
import { Sidebar } from "./Sidebar";
import { ToastRegion } from "./ToastRegion";
import { ViewContainer } from "./ViewContainer";

function ActiveView() {
  const { current } = useViewStore();
  switch (current.name) {
    case "dashboard":
      return <DashboardView />;
    case "profiles":
      return <ProfilesView />;
    case "repos":
      return <ReposView />;
    case "ssh":
      return <SshView />;
    case "credentials":
      return <CredentialsView />;
    case "activity":
      return <ActivityView />;
    case "settings":
      return <SettingsView />;
    case "about":
      return <AboutView />;
    default:
      return <DashboardView />;
  }
}

export function AppShell() {
  const { openPalette, navigate } = useViewStore();

  useHotkey(
    "k",
    (e) => {
      if (isMod(e)) {
        e.preventDefault();
        openPalette();
      }
    },
    { ignoreInputs: false }
  );

  useHotkey(
    ",",
    (e) => {
      if (isMod(e)) {
        e.preventDefault();
        navigate({ name: "settings" });
      }
    },
    { ignoreInputs: false }
  );

  return (
    <div className="flex h-full bg-(--color-bg)">
      <Sidebar />
      <div className="flex flex-col flex-1 min-w-0">
        <Header />
        <ViewContainer>
          <ActiveView />
        </ViewContainer>
      </div>

      <CommandPalette />
      <ToastRegion />
    </div>
  );
}
