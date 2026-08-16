import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { useBootstrapStore } from "@/stores/bootstrap";

type Phase =
  | { kind: "idle" }
  | { kind: "available"; version: string; body: string | null }
  | { kind: "downloading"; progress: number }
  | { kind: "ready" }
  | { kind: "error" };

export function UpdateDialog() {
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });
  const [dismissed, setDismissed] = useState(false);
  const forceUpdate = useBootstrapStore((s) => s.forceUpdate);
  // Held in a ref so the object survives re-renders without going into state.
  const pendingUpdate = useRef<Awaited<ReturnType<typeof check>>>(null);

  useEffect(() => {
    check()
      .then((update) => {
        // console.log("[updater] check() result:", update);
        if (update?.available) {
          pendingUpdate.current = update;
          setPhase({ kind: "available", version: update.version, body: update.body ?? null });
        }
      })
      .catch(() => {});
  }, []);

  const handleDownload = useCallback(async () => {
    const update = pendingUpdate.current;
    if (!update?.available) return;

    let downloaded = 0;
    let total = 0;

    try {
      // console.log("[updater] starting downloadAndInstall, update:", update);
      await update.downloadAndInstall((event) => {
        // console.log("[updater] download event:", event);
        switch (event.event) {
          case "Started":
            total = event.data.contentLength ?? 0;
            setPhase({ kind: "downloading", progress: 0 });
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            setPhase({
              kind: "downloading",
              progress: total > 0 ? Math.round((downloaded / total) * 100) : 0,
            });
            break;
          case "Finished":
            setPhase({ kind: "ready" });
            break;
        }
      });
      await relaunch();
    } catch (_err) {
      // console.error("[updater] downloadAndInstall failed:", err);
      setPhase({ kind: "error" });
    }
  }, []);

  const isVisible =
    (phase.kind === "available" ||
      phase.kind === "downloading" ||
      phase.kind === "ready" ||
      phase.kind === "error") &&
    !dismissed;

  const isLocked = forceUpdate && (phase.kind === "available" || phase.kind === "error");

  return (
    <Dialog.Root
      open={isVisible}
      onOpenChange={(open) => {
        if (!open && phase.kind !== "downloading" && !isLocked) setDismissed(true);
      }}
    >
      {isVisible && (
        <Dialog.Content
          title={phase.kind === "error" ? "Update Failed" : "Update Available"}
          description={
            phase.kind === "available"
              ? `GitPersona ${phase.version} is now available.`
              : phase.kind === "downloading"
                ? "Downloading update, please wait…"
                : phase.kind === "ready"
                  ? "Update installed. Restarting…"
                  : "Could not download the update. Please try again later."
          }
          hideClose={phase.kind === "downloading" || phase.kind === "ready" || isLocked}
          className="max-w-md"
        >
          {phase.kind === "available" && phase.body && (
            <p className="text-sm text-(--color-secondary) leading-relaxed whitespace-pre-line">
              {phase.body}
            </p>
          )}

          {phase.kind === "available" && forceUpdate && (
            <p className="text-sm text-(--color-warning) font-medium">
              This update is required to continue using GitPersona.
            </p>
          )}

          {phase.kind === "downloading" && (
            <div className="flex flex-col gap-2">
              <div className="h-2 rounded-full bg-(--color-surface-3) overflow-hidden">
                <div
                  className="h-full rounded-full bg-(--color-brand-400) transition-[width] duration-200"
                  style={{ width: `${phase.progress}%` }}
                />
              </div>
              <span className="text-xs text-(--color-muted) text-right tabular-nums">
                {phase.progress}%
              </span>
            </div>
          )}

          <Dialog.Footer>
            {phase.kind === "available" && (
              <>
                {!forceUpdate && (
                  <Button type="button" variant="ghost" onClick={() => setDismissed(true)}>
                    Remind me later
                  </Button>
                )}
                <Button type="button" variant="primary" onClick={handleDownload}>
                  Download & Install
                </Button>
              </>
            )}
            {phase.kind === "error" && !forceUpdate && (
              <Button type="button" variant="ghost" onClick={() => setDismissed(true)}>
                Close
              </Button>
            )}
            {phase.kind === "error" && forceUpdate && (
              <Button type="button" variant="primary" onClick={handleDownload}>
                Retry
              </Button>
            )}
          </Dialog.Footer>
        </Dialog.Content>
      )}
    </Dialog.Root>
  );
}
