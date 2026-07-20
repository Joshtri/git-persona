import { AntennaSignal } from "@gravity-ui/icons";
import type { ReactNode } from "react";

interface Props {
  onRetry?: () => void;
  action?: ReactNode;
}

export function OfflineState({ action }: Props) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <AntennaSignal className="size-10 text-(--color-muted)" aria-hidden="true" />
      <p className="text-sm font-medium text-(--color-fg)">No connection</p>
      <p className="text-xs text-(--color-muted) max-w-xs">
        GitPersona cannot reach the Git configuration. Check your environment.
      </p>
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
