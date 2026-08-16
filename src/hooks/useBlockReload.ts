import { useEffect } from "react";

export function useBlockReload(enabled = import.meta.env.PROD): void {
  useEffect(() => {
    if (!enabled) return;
    const onKey = (e: KeyboardEvent) => {
      const isRefresh = e.key === "F5" || (e.ctrlKey && e.key.toLowerCase() === "r");
      if (isRefresh) {
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [enabled]);
}
