import { Switch as BaseSwitch } from "@base-ui/react";
import { cn } from "@/lib/cn";

interface Props {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  id?: string;
}

export function Switch({ checked, onCheckedChange, disabled, label, id }: Props) {
  return (
    <div className="flex items-center gap-2">
      <BaseSwitch.Root
        id={id}
        checked={checked}
        onCheckedChange={onCheckedChange}
        disabled={disabled}
        className="flex h-5 w-9 shrink-0 border border-neutral-950 bg-white p-0.5 transition-colors duration-150 ease-[ease] dark:border-white dark:bg-neutral-950 data-checked:bg-neutral-950 dark:data-checked:bg-white focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white"
      >
        <BaseSwitch.Thumb
          className={cn(
            "size-3.5 bg-neutral-950 transition-[translate,background-color] duration-150 ease-[ease] data-checked:translate-x-4 data-checked:bg-white dark:bg-white dark:data-checked:bg-neutral-950"
          )}
        />
      </BaseSwitch.Root>
      {label && (
        <label htmlFor={id} className="text-sm text-(--color-fg) cursor-pointer select-none">
          {label}
        </label>
      )}
    </div>
  );
}
