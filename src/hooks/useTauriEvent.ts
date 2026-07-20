import type { UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function useTauriEvent(
  subscribe: (cb: (payload: unknown) => void) => Promise<UnlistenFn>,
  handler: (payload: unknown) => void
): void {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    subscribe(handler).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [subscribe, handler]);
}
