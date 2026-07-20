import { AlertDialog as BaseAlertDialog } from "@base-ui/react/alert-dialog";
import { CircleCheck, CircleInfo, TriangleExclamation } from "@gravity-ui/icons";
import type { ComponentType, ReactNode } from "react";
import { useState } from "react";
import { Button } from "@/components/button";
import { Spinner } from "@/components/spinner";
import { cn } from "@/lib/cn";

type Tone = "danger" | "warning" | "info" | "success";

interface ToneConfig {
  icon: ComponentType<{ className?: string }>;
  iconWrap: string;
  iconColor: string;
  confirmVariant: "primary" | "danger";
}

const tones: Record<Tone, ToneConfig> = {
  danger: {
    icon: TriangleExclamation,
    iconWrap: "bg-(--color-danger)/12",
    iconColor: "text-(--color-danger)",
    confirmVariant: "danger",
  },
  warning: {
    icon: TriangleExclamation,
    iconWrap: "bg-(--color-warning)/12",
    iconColor: "text-(--color-warning)",
    confirmVariant: "primary",
  },
  info: {
    icon: CircleInfo,
    iconWrap: "bg-(--color-brand-500)/12",
    iconColor: "text-(--color-brand-500)",
    confirmVariant: "primary",
  },
  success: {
    icon: CircleCheck,
    iconWrap: "bg-(--color-success)/12",
    iconColor: "text-(--color-success)",
    confirmVariant: "primary",
  },
};

interface ConfirmationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: ReactNode;
  tone?: Tone;
  icon?: ComponentType<{ className?: string }>;
  confirmLabel?: string;
  cancelLabel?: string;
  loading?: boolean;
  onConfirm: () => void | Promise<void>;
}

export function ConfirmationDialog({
  open,
  onOpenChange,
  title,
  description,
  tone = "danger",
  icon,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  loading,
  onConfirm,
}: ConfirmationDialogProps) {
  const [busy, setBusy] = useState(false);
  const isBusy = loading ?? busy;
  const config = tones[tone];
  const Icon = icon ?? config.icon;

  const handleConfirm = async () => {
    if (loading === undefined) {
      setBusy(true);
      try {
        await onConfirm();
      } finally {
        setBusy(false);
      }
      return;
    }
    await onConfirm();
  };

  return (
    <BaseAlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <BaseAlertDialog.Portal>
        <BaseAlertDialog.Backdrop className="fixed inset-0 z-40 bg-black/75 animate-[fade-in_150ms_ease]" />
        <BaseAlertDialog.Popup
          className={cn(
            "fixed z-50 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
            "w-full max-w-md rounded-(--radius-xl) bg-(--color-surface)",
            "border border-(--color-border) p-6 flex flex-col gap-4",
            "animate-[slide-up_200ms_ease]"
          )}
          style={{ boxShadow: "var(--shadow-md)" }}
        >
          <div className="flex items-start gap-4">
            <span
              aria-hidden="true"
              className={cn(
                "flex size-10 shrink-0 items-center justify-center rounded-full",
                config.iconWrap,
                config.iconColor
              )}
            >
              <Icon className="size-5" />
            </span>
            <div className="flex flex-col gap-1 pt-0.5">
              <BaseAlertDialog.Title className="text-base font-semibold text-(--color-fg)">
                {title}
              </BaseAlertDialog.Title>
              {description && (
                <BaseAlertDialog.Description className="text-sm text-(--color-secondary)">
                  {description}
                </BaseAlertDialog.Description>
              )}
            </div>
          </div>
          <div className="flex items-center justify-end gap-2 pt-2">
            <Button variant="ghost" onClick={() => onOpenChange(false)} disabled={isBusy}>
              {cancelLabel}
            </Button>
            <Button variant={config.confirmVariant} onClick={handleConfirm} disabled={isBusy}>
              {isBusy && <Spinner size="sm" />}
              {confirmLabel}
            </Button>
          </div>
        </BaseAlertDialog.Popup>
      </BaseAlertDialog.Portal>
    </BaseAlertDialog.Root>
  );
}
