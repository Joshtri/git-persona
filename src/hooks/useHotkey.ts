import { useEffect } from "react";
import { isInputTarget } from "@/lib/keyboard";

interface Options {
  ignoreInputs?: boolean;
  enabled?: boolean;
}

export function useHotkey(
  key: string,
  handler: (e: KeyboardEvent) => void,
  { ignoreInputs = true, enabled = true }: Options = {}
): void {
  useEffect(() => {
    if (!enabled) return;
    const onKey = (e: KeyboardEvent) => {
      if (ignoreInputs && isInputTarget(e)) return;
      if (e.key === key) handler(e);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [key, handler, ignoreInputs, enabled]);
}
