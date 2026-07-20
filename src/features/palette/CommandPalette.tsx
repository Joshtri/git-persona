import { Separator } from "@base-ui/react";
import {
  ArrowRightFromSquare,
  ArrowRotateLeft,
  CircleInfo,
  FileArrowUp,
  FileText,
  Folder,
  FolderOpen,
  Gear,
  House,
  Key,
  ListTimeline,
  Lock,
  Magnifier,
  Person,
  Plus,
  TrashBin,
  Xmark,
} from "@gravity-ui/icons";
import type { ComponentType, SVGAttributes } from "react";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Kbd } from "@/components/kbd";
import { cn } from "@/lib/cn";
import { useCredentialsStore } from "@/stores/credentials";
import { useProfilesStore } from "@/stores/profiles";
import { useReposStore } from "@/stores/repos";
import { useSshStore } from "@/stores/ssh";
import { useViewStore } from "@/stores/view";

interface Command {
  id: string;
  label: string;
  description?: string;
  Icon: ComponentType<SVGAttributes<SVGElement>>;
  kbd?: string[];
  group: string;
  action: () => void;
}

export function CommandPalette() {
  const { paletteOpen, closePalette, navigate } = useViewStore();
  const { items: profiles, apply } = useProfilesStore();
  const { items: repos, reveal, scan } = useReposStore();
  const {
    setImportOpen: setSshImportOpen,
    setGenerateOpen: setSshGenerateOpen,
    revealFolder: revealSshFolder,
    openConfig: openSshConfig,
  } = useSshStore();
  const {
    setCreateOpen: setCredentialCreateOpen,
    fetch: fetchCredentials,
    openManager: openCredentialManager,
  } = useCredentialsStore();
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const navCommands: Command[] = [
    {
      id: "nav-dashboard",
      group: "Navigate",
      label: "Go to Dashboard",
      Icon: House,
      action: () => navigate({ name: "dashboard" }),
    },
    {
      id: "nav-profiles",
      group: "Navigate",
      label: "Go to Profiles",
      Icon: Person,
      action: () => navigate({ name: "profiles" }),
    },
    {
      id: "nav-repos",
      group: "Navigate",
      label: "Go to Repositories",
      Icon: Folder,
      action: () => navigate({ name: "repos" }),
    },
    {
      id: "nav-ssh",
      group: "Navigate",
      label: "Go to SSH Keys",
      Icon: Key,
      action: () => navigate({ name: "ssh" }),
    },
    {
      id: "nav-credentials",
      group: "Navigate",
      label: "Go to Credentials",
      Icon: Lock,
      action: () => navigate({ name: "credentials" }),
    },
    {
      id: "nav-activity",
      group: "Navigate",
      label: "Go to Activity",
      Icon: ListTimeline,
      action: () => navigate({ name: "activity" }),
    },
    {
      id: "nav-settings",
      group: "Navigate",
      label: "Go to Settings",
      Icon: Gear,
      kbd: ["⌘", ","],
      action: () => navigate({ name: "settings" }),
    },
    {
      id: "nav-about",
      group: "Navigate",
      label: "Go to About",
      Icon: CircleInfo,
      action: () => navigate({ name: "about" }),
    },
  ];

  const profileCommands: Command[] = profiles.map((p) => ({
    id: `profile-apply-${p.id}`,
    group: "Apply Profile",
    label: `Apply ${p.label}`,
    description: p.identity.email,
    Icon: ArrowRightFromSquare,
    action: () => {
      apply(p.id);
      closePalette();
    },
  }));

  const repoActionCommands: Command[] = [
    {
      id: "repo-scan",
      group: "Repositories",
      label: "Scan folders for repositories",
      Icon: Folder,
      action: () => {
        closePalette();
        scan();
      },
    },
  ];

  const repoCommands: Command[] = repos.map((r) => ({
    id: `repo-reveal-${r.id}`,
    group: "Repositories",
    label: `Reveal ${r.name}`,
    description: r.path,
    Icon: Folder,
    action: () => {
      reveal(r.id);
      closePalette();
    },
  }));

  const sshCommands: Command[] = [
    {
      id: "ssh-import",
      group: "SSH",
      label: "Import SSH Key",
      Icon: FileArrowUp,
      action: () => {
        navigate({ name: "ssh" });
        setSshImportOpen(true);
      },
    },
    {
      id: "ssh-generate",
      group: "SSH",
      label: "Generate SSH Key",
      Icon: Plus,
      action: () => {
        navigate({ name: "ssh" });
        setSshGenerateOpen(true);
      },
    },
    {
      id: "ssh-assign",
      group: "SSH",
      label: "Assign SSH Key",
      Icon: Key,
      action: () => navigate({ name: "ssh" }),
    },
    {
      id: "ssh-reveal-folder",
      group: "SSH",
      label: "Reveal SSH Folder",
      Icon: FolderOpen,
      action: () => {
        revealSshFolder();
        closePalette();
      },
    },
    {
      id: "ssh-config-open",
      group: "SSH",
      label: "Open SSH Config",
      Icon: FileText,
      action: () => {
        openSshConfig();
        closePalette();
      },
    },
  ];

  const credentialCommands: Command[] = [
    {
      id: "credential-create",
      group: "Credentials",
      label: "Create Credential",
      Icon: Plus,
      action: () => {
        navigate({ name: "credentials" });
        setCredentialCreateOpen(true);
      },
    },
    {
      id: "credential-assign",
      group: "Credentials",
      label: "Assign Credential",
      Icon: Lock,
      action: () => navigate({ name: "credentials" }),
    },
    {
      id: "credential-delete",
      group: "Credentials",
      label: "Delete Credential",
      Icon: TrashBin,
      action: () => navigate({ name: "credentials" }),
    },
    {
      id: "credential-refresh",
      group: "Credentials",
      label: "Refresh Credentials",
      Icon: ArrowRotateLeft,
      action: () => {
        navigate({ name: "credentials" });
        fetchCredentials();
      },
    },
    {
      id: "credential-open-manager",
      group: "Credentials",
      label: "Open Windows Credential Manager",
      Icon: Key,
      action: () => {
        openCredentialManager();
        closePalette();
      },
    },
  ];

  const ALL_COMMANDS: Command[] = [
    ...navCommands,
    ...profileCommands,
    ...repoActionCommands,
    ...repoCommands,
    ...sshCommands,
    ...credentialCommands,
  ];

  const filtered = ALL_COMMANDS.filter(
    (c) =>
      query === "" ||
      c.label.toLowerCase().includes(query.toLowerCase()) ||
      (c.description ?? "").toLowerCase().includes(query.toLowerCase()) ||
      c.group.toLowerCase().includes(query.toLowerCase())
  );

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset selection on every query change, query is the trigger
  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  useEffect(() => {
    if (paletteOpen) {
      inputRef.current?.focus();
    } else {
      setQuery("");
      setActiveIndex(0);
    }
  }, [paletteOpen]);

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const cmd = filtered[activeIndex];
      if (cmd) {
        cmd.action();
        closePalette();
      }
    } else if (e.key === "Escape") {
      closePalette();
    }
  };

  useEffect(() => {
    const el = listRef.current?.children[activeIndex] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (!paletteOpen) return null;

  const groups: Map<string, Command[]> = new Map();
  for (const cmd of filtered) {
    if (!groups.has(cmd.group)) groups.set(cmd.group, []);
    groups.get(cmd.group)?.push(cmd);
  }

  let flatIndex = 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      onKeyDown={handleKeyDown}
    >
      <div
        className="absolute inset-0  backdrop-blur-[2px]"
        onClick={closePalette}
        aria-hidden="true"
      />

      <div
        className="relative w-full max-w-lg rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden animate-[slide-up_180ms_ease]"
        style={{ boxShadow: "var(--shadow-md)" }}
      >
        <div className="flex items-center gap-2 px-4 py-3 border-b border-(--color-border)">
          <Magnifier className="size-4 text-(--color-muted) shrink-0" aria-hidden="true" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search commands…"
            className="flex-1 bg-transparent text-sm text-(--color-fg) placeholder:text-(--color-placeholder) outline-none"
            aria-label="Search commands"
          />
          <div className="flex items-center gap-1.5">
            <Kbd className="text-[10px]">Esc</Kbd>
            <button
              type="button"
              onClick={closePalette}
              className="text-(--color-muted) hover:text-(--color-fg) transition-colors"
              aria-label="Close palette"
            >
              <Xmark className="size-4" />
            </button>
          </div>
        </div>

        <div className="max-h-80 overflow-y-auto" ref={listRef}>
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-10 text-center">
              <p className="text-sm text-(--color-muted)">
                No commands found for &ldquo;{query}&rdquo;
              </p>
            </div>
          ) : (
            Array.from(groups.entries()).map(([group, cmds], gi) => (
              <div key={group}>
                {gi > 0 && <Separator />}
                <p className="px-4 pt-2.5 pb-1 text-[10px] font-semibold uppercase tracking-widest text-(--color-muted)">
                  {group}
                </p>
                {cmds.map((cmd) => {
                  const isActive = flatIndex === activeIndex;
                  const currentIndex = flatIndex++;
                  return (
                    <button
                      key={cmd.id}
                      type="button"
                      onMouseEnter={() => setActiveIndex(currentIndex)}
                      onClick={() => {
                        cmd.action();
                        closePalette();
                      }}
                      className={cn(
                        "w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors",
                        isActive
                          ? "bg-(--color-brand-500)/12 text-(--color-fg)"
                          : "text-(--color-secondary) hover:bg-(--color-surface-2)"
                      )}
                    >
                      <cmd.Icon className="size-4 shrink-0" aria-hidden="true" />
                      <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                        <span className="text-sm font-medium">{cmd.label}</span>
                        {cmd.description && (
                          <span className="text-xs text-(--color-muted) truncate">
                            {cmd.description}
                          </span>
                        )}
                      </div>
                      {cmd.kbd && (
                        <div className="flex items-center gap-0.5 shrink-0">
                          {cmd.kbd.map((k) => (
                            <Kbd key={k} className="text-[10px]">
                              {k}
                            </Kbd>
                          ))}
                        </div>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className="border-t border-(--color-border) px-4 py-2 flex items-center gap-3 text-[10px] text-(--color-muted)">
          <span>
            <Kbd className="text-[10px]">↑↓</Kbd> navigate
          </span>
          <span>
            <Kbd className="text-[10px]">↵</Kbd> select
          </span>
          <span>
            <Kbd className="text-[10px]">Esc</Kbd> close
          </span>
        </div>
      </div>
    </div>
  );
}
