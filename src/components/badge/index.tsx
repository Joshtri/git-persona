import type { CSSProperties, ReactNode } from "react";
import { cn } from "@/lib/cn";

interface Props {
  variant?: "default" | "success" | "warning" | "danger";
  children: ReactNode;
  className?: string;
  style?: CSSProperties;
}

const variants = {
  default: "bg-(--color-surface-3) text-(--color-secondary)",
  success: "bg-(--color-success)/15 text-(--color-success)",
  warning: "bg-(--color-warning)/15 text-(--color-warning)",
  danger: "bg-(--color-danger)/15 text-(--color-danger)",
};

export function Badge({ variant = "default", children, className, style }: Props) {
  return (
    <span
      style={style}
      className={cn(
        "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium",
        variants[variant],
        className
      )}
    >
      {children}
    </span>
  );
}
