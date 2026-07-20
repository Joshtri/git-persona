import { Input as BaseInput } from "@base-ui/react/input";
import { Magnifier, Xmark } from "@gravity-ui/icons";
import type { InputHTMLAttributes } from "react";
import { Button } from "@/components/button";
import { cn } from "@/lib/cn";

interface Props extends Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "value" | "onChange"> {
  value: string;
  onValueChange: (v: string) => void;
  containerClassName?: string;
}

export function SearchInput({
  value,
  onValueChange,
  containerClassName,
  className,
  ...props
}: Props) {
  return (
    <div className={cn("relative flex items-center", containerClassName)}>
      <Magnifier
        className="absolute left-2.5 size-3.5 text-(--color-muted) pointer-events-none shrink-0"
        aria-hidden="true"
      />
      <BaseInput
        type="search"
        value={value}
        onValueChange={onValueChange}
        className={cn(
          "w-full h-8 pl-8 pr-8 text-sm rounded-(--radius-md)",
          "bg-(--color-surface-2) border border-(--color-border) text-(--color-fg)",
          "placeholder:text-(--color-placeholder)",
          "outline-none focus:border-(--color-brand-500) transition-colors",
          "[&::-webkit-search-cancel-button]:hidden",
          className
        )}
        {...props}
      />
      {value && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => onValueChange("")}
          aria-label="Clear search"
          className="absolute right-1 h-6 w-6 rounded-full px-0"
        >
          <Xmark className="size-3.5" />
        </Button>
      )}
    </div>
  );
}
