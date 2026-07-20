import { ArrowChevronRight } from "@gravity-ui/icons";
import type { ReactNode } from "react";
import { useViewStore } from "@/stores/view";

const VIEW_META: Record<string, { label: string; parent?: string }> = {
  dashboard: { label: "Dashboard" },
  profiles: { label: "Profiles", parent: "Identities" },
  ssh: { label: "SSH Keys", parent: "Identities" },
  credentials: { label: "Credentials", parent: "Identities" },
  repos: { label: "Repositories", parent: "Workspace" },
  activity: { label: "Activity", parent: "Workspace" },
  settings: { label: "Settings", parent: "System" },
  about: { label: "About", parent: "System" },
  onboarding: { label: "Welcome" },
};

interface Props {
  actions?: ReactNode;
}

export function Header({ actions }: Props) {
  const { current } = useViewStore();
  const meta = VIEW_META[current.name] ?? { label: current.name };

  return (
    <header className="h-11 shrink-0 flex items-center justify-between px-5 border-b border-(--color-border) bg-(--color-surface)">
      {/* Breadcrumb */}
      <nav aria-label="Breadcrumb" className="flex items-center gap-1.5 text-sm">
        {meta.parent && (
          <>
            <span className="text-(--color-muted)">{meta.parent}</span>
            <ArrowChevronRight className="size-3 text-(--color-muted)" aria-hidden="true" />
          </>
        )}
        <span className="font-medium text-(--color-fg)">{meta.label}</span>
      </nav>

      {/* Right-side slot */}
      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </header>
  );
}
