import { Popover } from "@base-ui/react/popover";
import { Copy, EllipsisVertical, FolderOpen, TrashBin } from "@gravity-ui/icons";
import { useState } from "react";
import { Button } from "@/components/button";
import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { SshKey } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";
import { useSshStore } from "@/stores/ssh";

interface Props {
  sshKey: SshKey;
}

export function SshActionsMenu({ sshKey }: Props) {
  const [open, setOpen] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const { reveal, remove } = useSshStore();
  const toast = useFeedbackStore((s) => s.toast);

  const run = (fn: () => void) => {
    setOpen(false);
    fn();
  };

  const copyFingerprint = async () => {
    try {
      await navigator.clipboard.writeText(sshKey.fingerprint);
      toast("Fingerprint copied", "success");
    } catch {
      toast("Could not copy fingerprint", "error");
    }
  };

  const assigned = sshKey.assigned_profile_id != null;

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className="flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-2) hover:text-(--color-fg) cursor-pointer"
        aria-label="SSH key actions"
      >
        <EllipsisVertical className="size-4" aria-hidden="true" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="end">
          <Popover.Popup className="z-50 min-w-44 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <Button variant="menu" size="menu" onClick={() => run(() => reveal(sshKey.id))}>
              <FolderOpen className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Reveal in file manager
            </Button>
            <Button variant="menu" size="menu" onClick={() => run(copyFingerprint)}>
              <Copy className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Copy fingerprint
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
        title="Remove SSH Key"
        description={
          assigned
            ? `"${sshKey.label}" is assigned to a profile. Unassign it first — this only removes it from GitPersona and never deletes the key file on disk.`
            : `Remove "${sshKey.label}" from GitPersona? This only removes the reference — the key file on disk is left untouched.`
        }
        confirmLabel="Remove from list"
        onConfirm={async () => {
          const ok = await remove(sshKey.id);
          if (ok) setConfirmRemove(false);
        }}
      />
    </Popover.Root>
  );
}
