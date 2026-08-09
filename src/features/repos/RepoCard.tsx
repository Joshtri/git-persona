import {
  BranchesDown,
  Clock,
  Folder,
  Star,
  StarFill,
  TriangleExclamation,
} from "@gravity-ui/icons";
import type { Repo } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { useReposStore } from "@/stores/repos";
import { AssignProfileMenu } from "./AssignProfileMenu";
import { RepoActionsMenu } from "./RepoActionsMenu";

interface Props {
  repo: Repo;
}

function timeAgo(iso: string | null): string | null {
  if (!iso) return null;
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function remoteHost(remote: string | null): string | null {
  if (!remote) return null;
  const match = remote.match(/(?:@|\/\/)([^/:]+)[/:]/);
  return match?.[1] ?? null;
}

export function RepoCard({ repo }: Props) {
  const toggleFavorite = useReposStore((s) => s.toggleFavorite);
  const updated = timeAgo(repo.last_opened ?? repo.detected_at);
  const host = remoteHost(repo.remote_origin);
  const missing = !repo.path_exists;

  return (
    <div
      className={cn(
        "flex items-center gap-4 px-4 py-3 transition-colors border-b border-(--color-border) last:border-b-0",
        missing
          ? "border-l-2 border-l-(--color-warning) bg-(--color-warning)/5 hover:bg-(--color-warning)/10"
          : "hover:bg-(--color-surface-2)"
      )}
      title={missing ? `Folder not found: ${repo.path}` : undefined}
    >
      <button
        type="button"
        onClick={() => toggleFavorite(repo.id)}
        className={cn(
          "flex size-8 items-center justify-center rounded-(--radius-md) border shrink-0 transition-colors",
          repo.favorite
            ? "border-(--color-warning)/40 bg-(--color-warning)/10 text-(--color-warning)"
            : "border-(--color-border) bg-(--color-surface-2) text-(--color-muted) hover:text-(--color-fg)"
        )}
        aria-label={repo.favorite ? "Unfavorite repository" : "Favorite repository"}
        aria-pressed={repo.favorite}
      >
        {repo.favorite ? (
          <StarFill className="size-4" aria-hidden="true" />
        ) : (
          <Star className="size-4" aria-hidden="true" />
        )}
      </button>

      <div className="flex flex-col gap-0.5 flex-1 min-w-0">
        <div className="flex items-center gap-2 min-w-0">
          <Folder className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
          <span className="text-sm font-medium text-(--color-fg) truncate">{repo.name}</span>
          {missing && (
            <span className="flex items-center gap-1 shrink-0 rounded-full bg-(--color-warning)/10 px-1.5 py-0.5 text-[10px] font-medium text-(--color-warning)">
              <TriangleExclamation className="size-3" aria-hidden="true" />
              Folder not found
            </span>
          )}
        </div>
        <span
          className={cn(
            "text-xs truncate font-mono",
            missing ? "text-(--color-warning) line-through" : "text-(--color-muted)"
          )}
        >
          {repo.path}
        </span>
        {host && <span className="text-[10px] text-(--color-secondary) truncate">{host}</span>}
      </div>

      <div className="flex flex-col items-end gap-1.5 shrink-0">
        <AssignProfileMenu repo={repo} />
        <div className="flex items-center gap-1 text-[10px] text-(--color-muted)">
          <BranchesDown className="size-3" aria-hidden="true" />
          <span className="font-mono">{repo.active_branch ?? "detached"}</span>
        </div>
        {updated && (
          <div className="flex items-center gap-1 text-[10px] text-(--color-muted)">
            <Clock className="size-3" aria-hidden="true" />
            <span>{updated}</span>
          </div>
        )}
      </div>

      <RepoActionsMenu repo={repo} />
    </div>
  );
}
