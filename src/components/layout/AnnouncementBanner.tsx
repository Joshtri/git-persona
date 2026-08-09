import { CircleCheckFill, CircleExclamationFill, CircleInfoFill, Xmark } from "@gravity-ui/icons";
import { cn } from "@/lib/cn";
import { useBootstrapStore } from "@/stores/bootstrap";

const typeStyles: Record<string, string> = {
  info: "bg-(--color-brand-400)/10 border-(--color-brand-400)/30 text-(--color-brand-300)",
  success: "bg-(--color-success)/10 border-(--color-success)/30 text-(--color-success)",
  warning: "bg-(--color-warning)/10 border-(--color-warning)/30 text-(--color-warning)",
};

const TypeIcon: Record<string, typeof CircleInfoFill> = {
  info: CircleInfoFill,
  success: CircleCheckFill,
  warning: CircleExclamationFill,
};

export function AnnouncementBanner() {
  const { announcements, dismissedIds, dismissAnnouncement } = useBootstrapStore();

  const active = announcements.find((a) => !dismissedIds.has(a.id));
  if (!active) return null;

  const Icon = TypeIcon[active.type] ?? CircleInfoFill;
  const styles = typeStyles[active.type] ?? typeStyles.info;

  return (
    <div
      role="status"
      aria-live="polite"
      className={cn("flex items-center gap-2.5 px-4 py-2 text-sm border-b", styles)}
    >
      <Icon className="size-4 shrink-0" aria-hidden="true" />
      <span className="font-medium mr-0.5">{active.title}:</span>
      <span className="flex-1 text-(--color-fg)">{active.message}</span>

      {active.action && (
        <a
          href={active.action.url}
          target="_blank"
          rel="noopener noreferrer"
          className="underline underline-offset-2 shrink-0"
        >
          {active.action.label}
        </a>
      )}

      {active.dismissible && (
        <button
          type="button"
          onClick={() => dismissAnnouncement(active.id)}
          className="text-(--color-muted) hover:text-(--color-fg) transition-colors shrink-0 ml-1"
          aria-label="Dismiss announcement"
        >
          <Xmark className="size-3.5" />
        </button>
      )}
    </div>
  );
}
