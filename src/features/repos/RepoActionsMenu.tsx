import { Popover } from "@base-ui/react/popover";
import { ArrowsRotateRight, Copy, EllipsisVertical, FolderOpen, TrashBin } from "@gravity-ui/icons";
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
  const { refresh, remove, reveal } = useReposStore();
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
