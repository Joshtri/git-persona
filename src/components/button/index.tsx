import { Button as BaseButton } from "@base-ui/react/button";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/cn";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost" | "danger" | "menu" | "menu-danger";
  size?: "sm" | "md" | "lg" | "menu";
}

const variants = {
  primary:
    "bg-brand-500 text-white not-data-disabled:hover:bg-brand-400 focus-visible:ring-2 focus-visible:ring-brand-500",
  secondary:
    "bg-(--color-surface-3) text-(--color-fg) border border-(--color-border) not-data-disabled:hover:bg-(--color-border-strong)",
  ghost:
    "text-(--color-secondary) not-data-disabled:hover:text-(--color-fg) not-data-disabled:hover:bg-(--color-surface-2)",
  danger: "bg-danger text-white not-data-disabled:hover:opacity-90",
  menu: "text-(--color-fg) not-data-disabled:hover:bg-(--color-surface-2)",
  "menu-danger": "text-danger not-data-disabled:hover:bg-danger/10",
};

const sizes = {
  sm: "h-7 px-2.5 text-xs rounded-sm justify-center gap-1.5",
  md: "h-8 px-3 text-sm rounded-md justify-center gap-1.5",
  lg: "h-10 px-4 text-sm rounded-md justify-center gap-1.5",
  menu: "w-full justify-start gap-2 px-2.5 py-1.5 text-xs rounded-sm",
};

export function Button({
  variant = "secondary",
  size = "md",
  className,
  children,
  ...props
}: Props) {
  return (
    <BaseButton
      className={cn(
        "inline-flex items-center font-medium transition-colors select-none",
        "focus-visible:outline-2 focus-visible:-outline-offset-1",
        "data-disabled:pointer-events-none data-disabled:opacity-40",
        "dark:focus-visible:ring-offset-(--color-surface)",
        "cursor-pointer",
        variants[variant],
        sizes[size],
        className
      )}
      {...props}
    >
      {children}
    </BaseButton>
  );
}
