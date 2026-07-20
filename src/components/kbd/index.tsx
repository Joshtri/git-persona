import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

export function Kbd({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <kbd
      className={cn(
        "inline-flex items-center px-1.5 py-0.5 rounded-(--radius-sm)",
        "text-xs font-mono text-(--color-muted)",
        "bg-(--color-surface-3) border border-(--color-border)",
        className
      )}
    >
      {children}
    </kbd>
  );
}
