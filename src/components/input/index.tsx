import { Input as BaseInput } from "@base-ui/react/input";
import type { InputHTMLAttributes } from "react";
import { cn } from "@/lib/cn";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

export function Input({ error, className, ...props }: Props) {
  return (
    <BaseInput
      className={cn(
        "w-full h-8 px-3 text-sm rounded-(--radius-md)",
        "bg-(--color-surface-2) border text-(--color-fg)",
        "placeholder:text-(--color-placeholder)",
        "transition-colors outline-none",
        "data-disabled:cursor-not-allowed data-disabled:opacity-50",
        error
          ? "border-(--color-danger) focus:border-(--color-danger)"
          : "border-(--color-border) focus:border-(--color-brand-500)",
        className,
      )}
      {...props}
    />
  );
}
