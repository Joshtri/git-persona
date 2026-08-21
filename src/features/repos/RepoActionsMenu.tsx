import { Popover } from "@base-ui/react/popover";
import {
  ArrowsRotateRight,
  Check,
  Copy,
  EllipsisVertical,
  FolderOpen,
  Plus,
  TrashBin,
} from "@gravity-ui/icons";
import { useState } from "react";
import { Button } from "@/components/button";
import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { Repo } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";
import { useReposStore } from "@/stores/repos";

interface Props {
  repo: Repo;
}

export function RepoActionsMenu({ repo }: Props) {
  const [open, setOpen] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const { refresh, remove, reveal, groups, setGroup, openGroupDialog } = useReposStore();
  const toast = useFeedbackStore((s) => s.toast);

  const run = (fn: () => void) => {
    setOpen(false);
    fn();
  };

  const copyPath = async () => {
    try {
      await navigator.clipboard.writeText(repo.path);
      toast("Path copied", "success");
    } catch {
      toast("Could not copy path", "error");
    }
  };

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className="flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-2) hover:text-(--color-fg) cursor-pointer"
        aria-label="Repository actions"
      >
        <EllipsisVertical className="size-4" aria-hidden="true" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="end">
          <Popover.Popup className="z-50 min-w-44 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <Button variant="menu" size="menu" onClick={() => run(() => reveal(repo.id))}>
              <FolderOpen className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Reveal in file manager
            </Button>
            <Button variant="menu" size="menu" onClick={() => run(copyPath)}>
              <Copy className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Copy path
            </Button>
            <Button variant="menu" size="menu" onClick={() => run(() => refresh(repo.id))}>
              <ArrowsRotateRight className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Refresh metadata
            </Button>

            <div className="my-1 h-px bg-(--color-border)" />
            <p className="px-2.5 pb-1 pt-1 text-[10px] font-medium uppercase tracking-wide text-(--color-muted)">
              Move to group
            </p>
            <div className="max-h-44 overflow-y-auto">
              <button
                type="button"
                onClick={() => run(() => setGroup(repo.id, null))}
                className="flex w-full items-center justify-between gap-2 rounded-(--radius-sm) px-2.5 py-1.5 text-left text-xs text-(--color-secondary) hover:bg-(--color-surface-2)"
              >
                Ungrouped
                {repo.group_id == null && <Check className="size-3.5" aria-hidden="true" />}
              </button>
              {groups.map((group) => (
                <button
                  key={group.id}
                  type="button"
                  onClick={() => run(() => setGroup(repo.id, group.id))}
                  className="flex w-full items-center justify-between gap-2 rounded-(--radius-sm) px-2.5 py-1.5 text-left text-xs text-(--color-fg) hover:bg-(--color-surface-2)"
                >
                  <span className="flex min-w-0 items-center gap-2">
                    <span
                      className="size-2 shrink-0 rounded-full"
                      style={{ backgroundColor: group.color ?? "#6366f1" }}
                      aria-hidden="true"
                    />
                    <span className="truncate">{group.name}</span>
                  </span>
                  {repo.group_id === group.id && (
                    <Check className="size-3.5 shrink-0" aria-hidden="true" />
                  )}
                </button>
              ))}
            </div>
            <Button
              variant="menu"
              size="menu"
              onClick={() =>
                run(() => openGroupDialog({ editing: null, assignRepoIds: [repo.id] }))
              }
            >
              <Plus className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              New group…
            </Button>

            <div className="my-1 h-px bg-(--color-border)" />
            <Button
              variant="menu-danger"
              size="menu"
              onClick={() => run(() => setConfirmRemove(true))}
            >
              <TrashBin className="size-3.5" aria-hidden="true" />
              Remove from list
            </Button>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>

      <ConfirmationDialog
        open={confirmRemove}
        onOpenChange={setConfirmRemove}
        tone="danger"
        icon={TrashBin}
        title="Remove Repository"
        description={`Remove "${repo.name}" from the list? This only removes it from GitPersona — the repository on disk is not touched.`}
        confirmLabel="Remove from list"
        onConfirm={async () => {
          await remove(repo.id);
          setConfirmRemove(false);
        }}
      />
    </Popover.Root>
  );
}
