import { Avatar as BaseAvatar } from "@base-ui/react/avatar";
import { cn } from "@/lib/cn";
import { initials } from "@/lib/format";

interface Props {
  name: string;
  src?: string;
  color?: string;
  size?: "sm" | "md" | "lg" | "xl";
  className?: string;
}

const sizes = {
  sm: "size-6 text-[10px]",
  md: "size-8 text-xs",
  lg: "size-10 text-sm",
  xl: "size-14 text-lg",
} as const;

export function Avatar({ name, src, color = "#6366f1", size = "md", className }: Props) {
  return (
    <BaseAvatar.Root
      className={cn(
        "inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full font-semibold select-none",
        sizes[size],
        className
      )}
      style={{
        backgroundColor: `${color}22`,
        color,
        border: `1.5px solid ${color}44`,
      }}
    >
      {src && <BaseAvatar.Image src={src} alt={name} className="size-full object-cover" />}
      <BaseAvatar.Fallback
        delay={src ? 300 : 0}
        className="flex size-full items-center justify-center"
      >
        {initials(name)}
      </BaseAvatar.Fallback>
    </BaseAvatar.Root>
  );
}
