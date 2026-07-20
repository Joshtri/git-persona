import { cn } from "@/lib/cn";

interface Props {
  size?: "sm" | "md" | "lg";
  className?: string;
}

const sizes = { sm: "size-4", md: "size-5", lg: "size-8" };

export function Spinner({ size = "md", className }: Props) {
  return (
    <span
      role="status"
      aria-label="Loading"
      className={cn(
        "inline-block rounded-full border-2 border-current border-t-transparent animate-spin",
        sizes[size],
        className
      )}
    />
  );
}
