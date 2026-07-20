import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

interface Props {
  label: string;
  htmlFor?: string;
  error?: string;
  hint?: string;
  required?: boolean;
  children: ReactNode;
  className?: string;
}

export function Field({ label, htmlFor, error, hint, required, children, className }: Props) {
  return (
    <div className={cn("flex flex-col gap-1", className)}>
      <label htmlFor={htmlFor} className="text-xs font-medium text-(--color-secondary)">
        {label}
        {required && <span className="text-(--color-danger) ml-0.5">*</span>}
      </label>
      {children}
      {hint && !error && <p className="text-xs text-(--color-muted)">{hint}</p>}
      {error && <p className="text-xs text-(--color-danger)">{error}</p>}
    </div>
  );
}
