import { Dialog as BaseDialog } from "@base-ui/react";
import { Xmark } from "@gravity-ui/icons";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
}

interface DialogContentProps {
  title: string;
  description?: string;
  children: ReactNode;
  className?: string;
  hideClose?: boolean;
}

function Root({ open, onOpenChange, children }: DialogProps) {
  return (
    <BaseDialog.Root open={open} onOpenChange={onOpenChange}>
      {children}
    </BaseDialog.Root>
  );
}

function Trigger({ children }: { children: ReactNode }) {
  return <BaseDialog.Trigger render={<span />}>{children}</BaseDialog.Trigger>;
}

function Content({ title, description, children, className, hideClose }: DialogContentProps) {
  return (
    <BaseDialog.Portal>
      <BaseDialog.Backdrop className="fixed inset-0  bg-black/75 z-40 animate-[fade-in_150ms_ease]" />
      <BaseDialog.Popup
        className={cn(
          "fixed z-50 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
          "w-full max-w-md rounded-(--radius-xl) bg-(--color-surface)",
          "border border-(--color-border) p-6 flex flex-col gap-4",
          "animate-[slide-up_200ms_ease]",
          className
        )}
        style={{ boxShadow: "var(--shadow-md)" }}
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex flex-col gap-1">
            <BaseDialog.Title className="text-base font-semibold text-(--color-fg)">
              {title}
            </BaseDialog.Title>
            {description && (
              <BaseDialog.Description className="text-sm text-(--color-secondary)">
                {description}
              </BaseDialog.Description>
            )}
          </div>
          {!hideClose && (
            <BaseDialog.Close
              className="text-(--color-muted) hover:text-(--color-fg) transition-colors mt-0.5"
              aria-label="Close"
            >
              <Xmark className="size-4" />
            </BaseDialog.Close>
          )}
        </div>
        {children}
      </BaseDialog.Popup>
    </BaseDialog.Portal>
  );
}

function Footer({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn("flex items-center justify-end gap-2 pt-2", className)}>{children}</div>
  );
}

export const Dialog = { Root, Trigger, Content, Footer };
