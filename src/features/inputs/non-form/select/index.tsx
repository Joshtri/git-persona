// components/ui/select.tsx

import { Select as BaseSelect } from "@base-ui/react/select";
import { Check } from "@gravity-ui/icons";
import type * as React from "react";
import { cn } from "@/lib/cn";

export interface SelectOption {
  label: string;
  value: string;
}

interface Props {
  items: SelectOption[];
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function Select({
  items,
  value,
  defaultValue,
  onValueChange,
  placeholder = "Select…",
  disabled,
  className,
}: Props) {
  return (
    <BaseSelect.Root<string, false>
      items={items}
      value={value}
      defaultValue={defaultValue}
      onValueChange={
        onValueChange
          ? (next) => {
              if (next != null) onValueChange(next);
            }
          : undefined
      }
      disabled={disabled}
    >
      <BaseSelect.Trigger
        className={cn(
          "flex h-8 min-w-28 items-center justify-between gap-2 px-3 text-sm rounded-(--radius-md)",
          "bg-(--color-surface-2) border border-(--color-border) text-(--color-fg)",
          "not-data-disabled:hover:bg-(--color-border-strong) transition-colors",
          "data-popup-open:border-(--color-brand-500)",
          "data-disabled:opacity-50 data-disabled:cursor-not-allowed",
          "outline-none focus-visible:border-(--color-brand-500)",
          className
        )}
      >
        <BaseSelect.Value
          placeholder={placeholder}
          className="truncate data-placeholder:text-(--color-placeholder)"
        />
        <BaseSelect.Icon className="shrink-0 text-(--color-muted)">
          <CaretUpDownIcon className="size-3.5" />
        </BaseSelect.Icon>
      </BaseSelect.Trigger>

      <BaseSelect.Portal>
        <BaseSelect.Positioner
          side="bottom"
          sideOffset={4}
          collisionPadding={8}
          className="z-50 border outline-none select-none"
        >
          <BaseSelect.Popup
            className={cn(
              "fixed top-1/2 left-1/2",
              "transform -translate-x-1/2 -translate-y-1/2",
              "min-w-[var(--anchor-width)] origin-[var(--transform-origin)] rounded-(--radius-md)",
              "border border-(--color-border) text-(--color-fg) py-1",
              "transition-[scale,opacity] duration-100 ease-out",
              "data-starting-style:scale-95 data-starting-style:opacity-0",
              "data-ending-style:scale-95 data-ending-style:opacity-0"
            )}
            style={{
              backgroundColor: "var(--color-surface-2)",
              boxShadow: "var(--shadow-md)",
            }}
          >
            <BaseSelect.List>
              {items.map((item) => (
                <BaseSelect.Item
                  key={item.value}
                  value={item.value}
                  className={cn(
                    "grid grid-cols-[1rem_1fr] items-center gap-2 py-1.5 pr-3 pl-2.5 text-sm",
                    "cursor-default select-none outline-none rounded-sm mx-1",
                    "data-highlighted:bg-(--color-brand-700) data-highlighted:text-gray-700",
                    "data-selected:text-(--color-brand-400)"
                  )}
                >
                  <BaseSelect.ItemIndicator
                    className="col-start-1 flex items-center justify-center"
                    style={{ color: "var(--color-brand-400)" }}
                  >
                    <Check className="size-3.5" />
                  </BaseSelect.ItemIndicator>
                  <BaseSelect.ItemText className="col-start-2 truncate">
                    {item.label}
                  </BaseSelect.ItemText>
                </BaseSelect.Item>
              ))}
            </BaseSelect.List>
          </BaseSelect.Popup>
        </BaseSelect.Positioner>
      </BaseSelect.Portal>
    </BaseSelect.Root>
  );
}

function CaretUpDownIcon(props: React.ComponentProps<"svg">) {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="currentColor"
      aria-hidden="true"
      {...props}
    >
      <path d="M11 10H5l3 3.5zm0-4H5l3-3.5z" />
    </svg>
  );
}
