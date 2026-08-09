import { Popover } from "@base-ui/react/popover";
import {
  ArrowRightFromSquare,
  Copy,
  EllipsisVertical,
  Eye,
  Lock,
  Pencil,
  TrashBin,
} from "@gravity-ui/icons";
import { useState } from "react";
import { Button } from "@/components/button";
import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { Credential } from "@/ipc/types.gen";
import { providerLabel } from "@/lib/credential-hosts";
import { useCredentialsStore } from "@/stores/credentials";
import { useFeedbackStore } from "@/stores/feedback";

interface Props {
  credential: Credential;
}

export function CredentialActionsMenu({ credential }: Props) {
  const [open, setOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const { switchCredential, remove, setEditing, setRevealing, setSettingPin } =
    useCredentialsStore();
  const toast = useFeedbackStore((s) => s.toast);

  const run = (fn: () => void) => {
    setOpen(false);
    fn();
  };

  const copyUsername = async () => {
    try {
      await navigator.clipboard.writeText(credential.username);
      toast("Username copied", "success");
    } catch {
      toast("Could not copy username", "error");
    }
  };

  const assigned = credential.profile_id != null;

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className="flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-2) hover:text-(--color-fg) cursor-pointer"
        aria-label="Credential actions"
      >
        <EllipsisVertical className="size-4" aria-hidden="true" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="end">
          <Popover.Popup className="z-50 min-w-48 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <Button
              variant="menu"
              size="menu"
              onClick={() => run(() => switchCredential(credential.id))}
            >
              <ArrowRightFromSquare className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Make active for host
            </Button>
            <Button variant="menu" size="menu" onClick={() => run(() => setEditing(credential))}>
              <Pencil className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Edit
            </Button>
            {credential.has_pin ? (
              <>
                <Button
                  variant="menu"
                  size="menu"
                  onClick={() => run(() => setRevealing(credential))}
                >
                  <Eye className="size-3.5 text-(--color-muted)" aria-hidden="true" />
                  Reveal token
                </Button>
                <Button
                  variant="menu"
                  size="menu"
                  onClick={() => run(() => setSettingPin(credential))}
                >
                  <Lock className="size-3.5 text-(--color-muted)" aria-hidden="true" />
                  Change PIN
                </Button>
              </>
            ) : (
              <Button
                variant="menu"
                size="menu"
                onClick={() => run(() => setSettingPin(credential))}
              >
                <Lock className="size-3.5 text-(--color-muted)" aria-hidden="true" />
                Set reveal PIN
              </Button>
            )}
            <Button variant="menu" size="menu" onClick={() => run(copyUsername)}>
              <Copy className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Copy username
            </Button>
            <div className="my-1 h-px bg-(--color-border)" />
            <Button
              variant="menu-danger"
              size="menu"
              onClick={() => run(() => setConfirmDelete(true))}
            >
              <TrashBin className="size-3.5" aria-hidden="true" />
              Delete
            </Button>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>

      <ConfirmationDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        tone="danger"
        icon={TrashBin}
        title="Delete Credential"
        description={
          assigned
            ? `This credential is assigned to a profile. Unassign it first — deleting removes its ${providerLabel(credential.host)} secret from the OS vault.`
            : `Delete the ${providerLabel(credential.host)} credential for "${credential.username}"? Its secret is removed from the OS vault. This cannot be undone.`
        }
        confirmLabel="Delete"
        onConfirm={async () => {
          const ok = await remove(credential.id);
          if (ok) setConfirmDelete(false);
        }}
      />
    </Popover.Root>
  );
}
