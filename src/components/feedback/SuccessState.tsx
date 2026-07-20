import { CircleCheckFill } from "@gravity-ui/icons";
import type { ReactNode } from "react";

interface Props {
  title: string;
  description?: string;
  action?: ReactNode;
}

export function SuccessState({ title, description, action }: Props) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <CircleCheckFill className="size-10 text-(--color-success)" aria-hidden="true" />
      <p className="text-sm font-medium text-(--color-fg)">{title}</p>
      {description && <p className="text-xs text-(--color-muted) max-w-xs">{description}</p>}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
