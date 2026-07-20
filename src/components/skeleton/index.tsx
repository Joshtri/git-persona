import { cn } from "@/lib/cn";

interface Props {
  className?: string;
  rounded?: boolean;
}

export function Skeleton({ className, rounded = false }: Props) {
  return (
    <div
      className={cn(
        "bg-(--color-surface-3) animate-pulse",
        rounded ? "rounded-full" : "rounded-(--radius-md)",
        className
      )}
      aria-hidden="true"
    />
  );
}
