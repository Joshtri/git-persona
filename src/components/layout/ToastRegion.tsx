import { CircleCheckFill, CircleExclamationFill, CircleInfoFill, Xmark } from "@gravity-ui/icons";
import { cn } from "@/lib/cn";
import { type ToastVariant, useFeedbackStore } from "@/stores/feedback";

const icons: Record<ToastVariant, typeof CircleCheckFill> = {
  success: CircleCheckFill,
  error: CircleExclamationFill,
  info: CircleInfoFill,
};

const iconColors: Record<ToastVariant, string> = {
  success: "text-(--color-success)",
  error: "text-(--color-danger)",
  info: "text-(--color-brand-400)",
};

export function ToastRegion() {
  const { toasts, dismiss } = useFeedbackStore();

  if (toasts.length === 0) return null;

  return (
    <div
      aria-live="polite"
      className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none"
    >
      {toasts.map((t) => {
        const Icon = icons[t.variant];
        return (
          <div
            key={t.id}
            className={cn(
              "flex items-center gap-2.5 px-3.5 py-2.5 rounded-(--radius-lg)",
              "bg-(--color-surface-2) border border-(--color-border)",
              "text-sm text-(--color-fg) pointer-events-auto",
              "animate-[slide-up_200ms_ease]",
              "min-w-56 max-w-80"
            )}
            style={{ boxShadow: "var(--shadow-md)" }}
          >
            <Icon className={cn("size-4 shrink-0", iconColors[t.variant])} aria-hidden="true" />
            <span className="flex-1">{t.message}</span>
            <button
              type="button"
              onClick={() => dismiss(t.id)}
              className="text-(--color-muted) hover:text-(--color-fg) transition-colors ml-1"
              aria-label="Dismiss"
            >
              <Xmark className="size-3.5" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
