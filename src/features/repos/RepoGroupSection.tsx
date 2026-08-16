import { Popover } from "@base-ui/react/popover";
import { ChevronDown, ChevronRight, EllipsisVertical, Pencil, TrashBin } from "@gravity-ui/icons";
import { useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { Repo, RepoGroup } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { deriveGroupHeaderColor } from "@/lib/color";
import { useReposStore } from "@/stores/repos";
import { useSettingsStore } from "@/stores/settings";
import { RepoCard } from "./RepoCard";

interface Props {
  group: RepoGroup | null;
  repos: Repo[];
  collapsed: boolean;
  onToggle: () => void;
}

export function RepoGroupSection({ group, repos, collapsed, onToggle }: Props) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const openGroupDialog = useReposStore((s) => s.openGroupDialog);
  const deleteGroup = useReposStore((s) => s.deleteGroup);

  const theme = useSettingsStore((s) => s.data?.theme);
  const isDark = theme !== "Light";

  const dotColor = group?.color ?? "var(--color-muted)";
  const label = group?.name ?? "Ungrouped";
  const headerBg = group?.color ? deriveGroupHeaderColor(group.color, isDark) : undefined;

  return (
    <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
      <div
        className="flex items-center gap-2 px-3 py-2.5 bg-(--color-surface-2)"
        style={headerBg ? { backgroundColor: headerBg } : undefined}
      >
        <button
          type="button"
          onClick={onToggle}
          className="flex flex-1 items-center gap-2 min-w-0 text-left cursor-pointer"
          aria-expanded={!collapsed}
        >
          {collapsed ? (
            <ChevronRight className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
          ) : (
            <ChevronDown className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
          )}
          <span
            className={cn("size-2.5 shrink-0 rounded-full", !group && "opacity-50")}
            style={{ backgroundColor: dotColor }}
            aria-hidden="true"
          />
          <span className="text-sm font-semibold text-(--color-fg) truncate">{label}</span>
          <Badge variant="default" className="text-[10px]">
            {repos.length}
          </Badge>
        </button>

        {group && (
          <Popover.Root open={menuOpen} onOpenChange={setMenuOpen}>
            <Popover.Trigger
              className="flex size-6 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-2) hover:text-(--color-fg) cursor-pointer"
              aria-label={`Actions for ${label}`}
            >
              <EllipsisVertical className="size-3.5" aria-hidden="true" />
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Positioner sideOffset={6} align="end">
                <Popover.Popup className="z-50 min-w-40 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
                  <Button
                    variant="menu"
                    size="menu"
                    onClick={() => {
                      setMenuOpen(false);
                      openGroupDialog({ editing: group, assignRepoId: null });
                    }}
                  >
                    <Pencil className="size-3.5 text-(--color-muted)" aria-hidden="true" />
                    Rename / recolor
                  </Button>
                  <Button
                    variant="menu-danger"
                    size="menu"
                    onClick={() => {
                      setMenuOpen(false);
                      setConfirmDelete(true);
                    }}
                  >
                    <TrashBin className="size-3.5" aria-hidden="true" />
                    Delete group
                  </Button>
                </Popover.Popup>
              </Popover.Positioner>
            </Popover.Portal>
          </Popover.Root>
        )}
      </div>

      {!collapsed &&
        (repos.length === 0 ? (
          <p className="px-4 py-4 text-xs text-(--color-muted)">
            No repositories here yet. Use a repository's ⋮ menu → “Move to group”.
          </p>
        ) : (
          <div className="border-t border-(--color-border)">
            {repos.map((repo) => (
              <RepoCard key={repo.id} repo={repo} />
            ))}
          </div>
        ))}

      {group && (
        <ConfirmationDialog
          open={confirmDelete}
          onOpenChange={setConfirmDelete}
          tone="danger"
          icon={TrashBin}
          title="Delete Group"
          description={`Delete the "${label}" group? Its repositories are kept and moved back to Ungrouped.`}
          confirmLabel="Delete group"
          onConfirm={async () => {
            await deleteGroup(group.id);
            setConfirmDelete(false);
          }}
        />
      )}
    </div>
  );
}
