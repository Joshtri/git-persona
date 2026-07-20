import { Button } from "@/components/button";
import type { AppError } from "@/ipc/errors";

interface Props {
  error: AppError;
  onRetry?: () => void;
}

export function ErrorState({ error, onRetry }: Props) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <p className="text-sm font-medium text-(--color-danger)">Something went wrong</p>
      <p className="text-xs text-(--color-muted) max-w-xs font-mono">{error.message}</p>
      {onRetry && (
        <Button size="sm" onClick={onRetry}>
          Retry
        </Button>
      )}
    </div>
  );
}
