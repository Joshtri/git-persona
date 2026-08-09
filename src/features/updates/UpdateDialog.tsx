import { CircleExclamationFill } from "@gravity-ui/icons";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { openUrl } from "@/ipc/client";
import type { ReleaseNote } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { useBootstrapStore } from "@/stores/bootstrap";
import { useFeedbackStore } from "@/stores/feedback";

const NOTE_TYPE_META: Record<string, { label: string; className: string }> = {
  security: {
    label: "Security",
    className: "bg-danger/15 text-danger",
  },
  feature: {
    label: "Feature",
    className: "bg-brand-500/15 text-(--color-brand-300)",
  },
  improvement: {
    label: "Improvement",
    className: "bg-(--color-success)/15 text-(--color-success)",
  },
  fix: {
    label: "Fix",
    className: "bg-(--color-warning)/15 text-(--color-warning)",
  },
};

const NOTE_TYPE_ORDER = ["security", "feature", "improvement", "fix"];

function groupNotes(notes: ReleaseNote[]): Record<string, ReleaseNote[]> {
  const groups: Record<string, ReleaseNote[]> = {};
  for (const note of notes) {
    if (!groups[note.type]) groups[note.type] = [];
    groups[note.type].push(note);
  }
  return groups;
}

async function handleDownload(url: string) {
  try {
    await openUrl(url);
  } catch {
    useFeedbackStore
      .getState()
      .toast("Could not open download link. Please visit the website manually.", "error");
  }
}

export function UpdateDialog() {
  const update = useBootstrapStore((s) => s.update);
  const updateDismissed = useBootstrapStore((s) => s.updateDismissed);
  const dismissUpdate = useBootstrapStore((s) => s.dismissUpdate);

  const isOpen = Boolean(update?.update_available && !updateDismissed);
  const latest = update?.latest ?? null;
  const isForced = update?.force_update ?? false;

  const groups = latest ? groupNotes(latest.release_notes) : {};
  const orderedTypes = NOTE_TYPE_ORDER.filter((t) => groups[t]);

  return (
    <Dialog.Root
      open={isOpen}
      onOpenChange={(open) => {
        if (!open && !isForced) dismissUpdate();
      }}
    >
      {isOpen && latest && (
        <Dialog.Content
          title={isForced ? "Critical Update Required" : "Update Available"}
          description={
            isForced
              ? "This version is no longer supported. You must update to continue."
              : `GitPersona ${latest.version} is now available.`
          }
          hideClose={isForced}
          className="max-w-lg"
        >
          {isForced && (
            <div className="flex items-start gap-2.5 p-3 rounded-(--radius-md) bg-danger/10 border border-danger/30 text-sm text-danger">
              <CircleExclamationFill className="size-4 shrink-0 mt-0.5" aria-hidden="true" />
              <span>
                GitPersona {latest.version} introduces critical changes. You must update before
                continuing.
              </span>
            </div>
          )}

          {/* Release card */}
          <div className="flex flex-col gap-1.5 p-3 rounded-(--radius-lg) bg-(--color-surface-2) border border-(--color-border)">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-sm font-semibold text-(--color-fg)">{latest.title}</span>
              <span className="text-xs font-medium px-1.5 py-0.5 rounded bg-brand-500/15 text-(--color-brand-300)">
                v{latest.version}
              </span>
              {latest.channel !== "stable" && (
                <span className="text-xs font-medium px-1.5 py-0.5 rounded bg-(--color-warning)/15 text-(--color-warning)">
                  {latest.channel}
                </span>
              )}
            </div>
            {latest.summary && (
              <p className="text-sm text-(--color-secondary) leading-relaxed">{latest.summary}</p>
            )}
          </div>

          {/* Release notes */}
          {orderedTypes.length > 0 && (
            <div className="flex flex-col gap-4 max-h-52 overflow-y-auto pr-0.5">
              {orderedTypes.map((type) => {
                const meta = NOTE_TYPE_META[type] ?? {
                  label: type,
                  className: "bg-(--color-surface-3) text-(--color-secondary)",
                };
                return (
                  <div key={type}>
                    <span
                      className={cn("text-xs font-semibold px-1.5 py-0.5 rounded", meta.className)}
                    >
                      {meta.label}
                    </span>
                    <ul className="mt-2 flex flex-col gap-1.5">
                      {groups[type].map((note) => (
                        <li
                          key={`${type}-${note.title}`}
                          className="flex items-start gap-2.5 text-sm text-(--color-secondary)"
                        >
                          <span className="mt-2 size-1 rounded-full bg-(--color-muted) shrink-0" />
                          {note.title}
                        </li>
                      ))}
                    </ul>
                  </div>
                );
              })}
            </div>
          )}

          <Dialog.Footer>
            {!isForced && (
              <Button type="button" variant="ghost" onClick={dismissUpdate}>
                Remind me later
              </Button>
            )}
            <Button
              type="button"
              variant="primary"
              onClick={() => handleDownload(latest.download_url)}
            >
              Download Update
            </Button>
          </Dialog.Footer>
        </Dialog.Content>
      )}
    </Dialog.Root>
  );
}
