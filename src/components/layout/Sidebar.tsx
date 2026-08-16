import { Popover } from "@base-ui/react/popover";
import {
  Check,
  ChevronsExpandVertical,
  CircleInfo,
  Folder,
  Gear,
  House,
  Key,
  ListTimeline,
  Lock,
  Person,
  Sliders,
} from "@gravity-ui/icons";
import { type ComponentType, type SVGAttributes, useEffect, useState } from "react";
import { useAppVersion } from "@/hooks/useAppVersion";
// import { Avatar, Kbd, Separator } from "@/components/ui";
import { cn } from "@/lib/cn";
import { useProfilesStore } from "@/stores/profiles";
import { useViewStore, type ViewName } from "@/stores/view";
import { Avatar } from "../avatar";
import { BrandLogo } from "../brand-logo";
import { Kbd } from "../kbd";
import { Separator } from "../separator";

interface NavItem {
  name: ViewName;
  label: string;
  Icon: ComponentType<SVGAttributes<SVGElement>>;
}

const MAIN_NAV: NavItem = { name: "dashboard", label: "Dashboard", Icon: House };

const IDENTITY_NAV: NavItem[] = [
  { name: "profiles", label: "Profiles", Icon: Person },
  { name: "ssh", label: "SSH Keys", Icon: Key },
  { name: "credentials", label: "Credentials", Icon: Lock },
];

const WORKSPACE_NAV: NavItem[] = [
  { name: "repos", label: "Repositories", Icon: Folder },
  { name: "rules", label: "Rules", Icon: Sliders },
  { name: "activity", label: "Activity", Icon: ListTimeline },
];

const SYSTEM_NAV: NavItem[] = [
  { name: "settings", label: "Settings", Icon: Gear },
  { name: "about", label: "About", Icon: CircleInfo },
];

function NavButton({
  item,
  active,
  onClick,
}: {
  item: NavItem;
  active: boolean;
  onClick: () => void;
}) {
  const { Icon } = item;
  return (
    <button
      key={item.name}
      type="button"
      onClick={onClick}
      className={cn(
        "relative w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-(--radius-md) text-sm transition-colors",
        active
          ? "bg-(--color-brand-500)/15 text-(--color-brand-400) font-medium"
          : "text-(--color-secondary) hover:text-(--color-fg) hover:bg-(--color-surface-2)"
      )}
    >
      {active && (
        <span
          className="absolute left-0 top-1 bottom-1 w-0.5 rounded-full bg-(--color-brand-500)"
          aria-hidden="true"
        />
      )}
      <Icon className="size-4 shrink-0" aria-hidden="true" />
      {item.label}
    </button>
  );
}

function NavSection({ label, items }: { label: string; items: NavItem[] }) {
  const { current, navigate } = useViewStore();
  return (
    <div className="flex flex-col gap-0.5">
      <p className="px-2.5 pb-1 pt-0.5 text-[10px] font-semibold uppercase tracking-widest text-(--color-muted) select-none">
        {label}
      </p>
      {items.map((item) => (
        <NavButton
          key={item.name}
          item={item}
          active={current.name === item.name}
          onClick={() => navigate({ name: item.name })}
        />
      ))}
    </div>
  );
}

function ProfileSwitcher() {
  const [open, setOpen] = useState(false);
  const navigate = useViewStore((s) => s.navigate);
  const { items: profiles, activeProfileId, apply, fetch } = useProfilesStore();
  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null;

  // The sidebar is always mounted, so a user who lands anywhere but Profiles
  // would otherwise see an empty switcher. Load profiles once if we have none.
  useEffect(() => {
    if (useProfilesStore.getState().items.length === 0) {
      fetch();
    }
  }, [fetch]);

  const choose = (id: string) => {
    setOpen(false);
    if (id !== activeProfileId) {
      apply(id);
    }
  };

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className="mx-2.5 mt-3 mb-1 flex items-center gap-2 px-2.5 py-1.5 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) hover:bg-(--color-surface-3) transition-colors cursor-pointer"
        title="Switch active profile"
      >
        {activeProfile ? (
          <>
            <Avatar name={activeProfile.identity.name} color={activeProfile.color} size="sm" />
            <div className="flex flex-col min-w-0 text-left">
              <span className="text-xs font-medium text-(--color-fg) truncate leading-tight">
                {activeProfile.label}
              </span>
              <span className="text-[10px] text-(--color-muted) truncate leading-tight">
                {activeProfile.identity.email}
              </span>
            </div>
            <ChevronsExpandVertical
              className="ml-auto size-3 text-(--color-muted) shrink-0"
              aria-hidden="true"
            />
          </>
        ) : (
          <>
            <span className="text-xs text-(--color-muted)">Select a profile</span>
            <ChevronsExpandVertical
              className="ml-auto size-3 text-(--color-muted) shrink-0"
              aria-hidden="true"
            />
          </>
        )}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="start">
          <Popover.Popup className="z-50 w-48 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <p className="px-2.5 pt-1 pb-1 text-[10px] font-semibold uppercase tracking-widest text-(--color-muted) select-none">
              Switch profile
            </p>
            {profiles.map((profile) => (
              <button
                key={profile.id}
                type="button"
                onClick={() => choose(profile.id)}
                className="flex w-full items-center gap-2 rounded-(--radius-sm) px-2 py-1.5 text-left hover:bg-(--color-surface-2)"
              >
                <Avatar name={profile.identity.name} color={profile.color} size="sm" />
                <div className="flex flex-col min-w-0 flex-1">
                  <span className="text-xs font-medium text-(--color-fg) truncate leading-tight">
                    {profile.label}
                  </span>
                  <span className="text-[10px] text-(--color-muted) truncate leading-tight">
                    {profile.identity.email}
                  </span>
                </div>
                {profile.id === activeProfileId && (
                  <Check className="size-3.5 shrink-0 text-(--color-success)" aria-hidden="true" />
                )}
              </button>
            ))}
            {profiles.length === 0 && (
              <p className="px-2.5 py-1.5 text-xs text-(--color-muted)">No profiles yet</p>
            )}
            <Separator className="my-1" />
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                navigate({ name: "profiles" });
              }}
              className="flex w-full items-center gap-2 rounded-(--radius-sm) px-2.5 py-1.5 text-left text-xs text-(--color-secondary) hover:bg-(--color-surface-2)"
            >
              <Person className="size-3.5 shrink-0" aria-hidden="true" />
              Manage profiles
            </button>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}

export function Sidebar() {
  const { current, navigate, openPalette } = useViewStore();
  const version = useAppVersion();

  return (
    <aside className="w-52 flex flex-col shrink-0 border-r border-(--color-border) bg-(--color-surface) select-none">
      <div className="h-11 flex items-center gap-2.5 px-3.5 border-b border-(--color-border)">
        <BrandLogo size="lg" />
        <span className="text-sm font-semibold text-(--color-fg) tracking-tight">GitPersona</span>
      </div>

      {/* Quick active-profile switcher */}
      <ProfileSwitcher />

      <nav className="flex-1 flex flex-col gap-3 px-2 py-2 overflow-y-auto">
        <NavButton
          item={MAIN_NAV}
          active={current.name === "dashboard"}
          onClick={() => navigate({ name: "dashboard" })}
        />

        <Separator />

        <NavSection label="Identities" items={IDENTITY_NAV} />
        <NavSection label="Workspace" items={WORKSPACE_NAV} />
        <NavSection label="System" items={SYSTEM_NAV} />
      </nav>

      <div className="border-t border-(--color-border) px-3 py-2.5 flex items-center justify-between">
        <span className="text-[11px] text-(--color-muted)">v{version}</span>
        <button
          type="button"
          onClick={openPalette}
          className="flex items-center gap-1 text-(--color-muted) hover:text-(--color-secondary) transition-colors"
          aria-label="Open command palette"
        >
          <Kbd className="text-[10px] px-1 py-0">⌘K</Kbd>
        </button>
      </div>
    </aside>
  );
}
