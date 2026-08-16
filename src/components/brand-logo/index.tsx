import { cn } from "@/lib/cn";

interface Props {
  size?: "sm" | "md" | "lg" | "xl";
  className?: string;
}

const sizes = {
  sm: "size-6",
  md: "size-8",
  lg: "size-10",
  xl: "size-16",
} as const;

export function BrandLogo({ size = "md", className }: Props) {
  return (
    <>
      <img
        src="/logo/logo-dark.png"
        alt="GitPersona"
        className={cn(
          "hidden shrink-0 rounded-(--radius-sm) object-cover dark:block",
          sizes[size],
          className
        )}
      />
      <img
        src="/logo/logo-light-v2.png"
        alt="GitPersona"
        className={cn(
          "block shrink-0 rounded-(--radius-sm) object-cover dark:hidden",
          sizes[size],
          className
        )}
      />
    </>
  );
}
