import { Popover } from "@base-ui/react/popover";
import {
  ArrowDown,
  ArrowUp,
  Copy,
  EllipsisVertical,
  Pencil,
  ToggleOff,
  ToggleOn,
  TrashBin,
} from "@gravity-ui/icons";
import { useState } from "react";
import { Button } from "@/components/button";
import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { Rule } from "@/ipc/types.gen";
import { useRulesStore } from "@/stores/rules";

interface Props {
  rule: Rule;
  isFirst: boolean;
  isLast: boolean;
}

export function RuleActionsMenu({ rule, isFirst, isLast }: Props) {
  const [open, setOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const { setEditing, duplicate, setEnabled, move, remove } = useRulesStore();

  const run = (fn: () => void) => {
    setOpen(false);
    fn();
  };

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className="flex size-7 items-center justify-center rounded-(--radius-md) text-(--color-muted) hover:bg-(--color-surface-2) hover:text-(--color-fg) cursor-pointer"
        aria-label="Rule actions"
      >
        <EllipsisVertical className="size-4" aria-hidden="true" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="end">
          <Popover.Popup className="z-50 min-w-48 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <Button variant="menu" size="menu" onClick={() => run(() => setEditing(rule))}>
              <Pencil className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Edit
            </Button>
            <Button
              variant="menu"
              size="menu"
              onClick={() => run(() => setEnabled(rule.id, !rule.enabled))}
            >
              {rule.enabled ? (
                <ToggleOff className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              ) : (
                <ToggleOn className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              )}
              {rule.enabled ? "Disable" : "Enable"}
            </Button>
            <Button variant="menu" size="menu" onClick={() => run(() => duplicate(rule.id))}>
              <Copy className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Duplicate
            </Button>
            <Button
              variant="menu"
              size="menu"
              disabled={isFirst}
              onClick={() => run(() => move(rule.id, "up"))}
            >
              <ArrowUp className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Move up
            </Button>
            <Button
              variant="menu"
              size="menu"
              disabled={isLast}
              onClick={() => run(() => move(rule.id, "down"))}
            >
              <ArrowDown className="size-3.5 text-(--color-muted)" aria-hidden="true" />
              Move down
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
        title="Delete Rule"
        description={`Delete the rule "${rule.name}"? Repositories will fall back to their manual assignment. This cannot be undone.`}
        confirmLabel="Delete"
        onConfirm={async () => {
          const ok = await remove(rule.id);
          if (ok) setConfirmDelete(false);
        }}
      />
    </Popover.Root>
  );
}
