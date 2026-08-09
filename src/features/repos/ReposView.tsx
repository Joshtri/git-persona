import { Folder, FolderPlus, MagnifierPlus } from "@gravity-ui/icons";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
import { Spinner } from "@/components/spinner";
import type { Repo } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { useProfilesStore } from "@/stores/profiles";
import { useReposStore } from "@/stores/repos";
import { GroupFormDialog } from "./GroupFormDialog";
import { RepoGroupSection } from "./RepoGroupSection";

const UNGROUPED_KEY = "__ungrouped__";

type SortKey = "recent" | "name" | "opened";
type FilterKey = "all" | "favorites" | "assigned" | "unassigned";

const SORTS: { key: SortKey; label: string }[] = [
  { key: "recent", label: "Recently detected" },
  { key: "name", label: "Name" },
  { key: "opened", label: "Recently opened" },
];

const FILTERS: { key: FilterKey; label: string }[] = [
  { key: "all", label: "All" },
  { key: "favorites", label: "Favorites" },
  { key: "assigned", label: "Assigned" },
  { key: "unassigned", label: "Unassigned" },
];

function matchesFilter(repo: Repo, filter: FilterKey): boolean {
  switch (filter) {
    case "favorites":
      return repo.favorite;
    case "assigned":
      return repo.active_profile_id != null;
    case "unassigned":
      return repo.active_profile_id == null;
    default:
      return true;
  }
}

function matchesQuery(repo: Repo, q: string): boolean {
  if (q === "") return true;
  const needle = q.toLowerCase();
  return (
    repo.name.toLowerCase().includes(needle) ||
    repo.path.toLowerCase().includes(needle) ||
    (repo.remote_origin?.toLowerCase().includes(needle) ?? false) ||
    (repo.active_branch?.toLowerCase().includes(needle) ?? false)
  );
}

function sortRepos(repos: Repo[], sort: SortKey): Repo[] {
  const copy = [...repos];
  switch (sort) {
    case "name":
      return copy.sort((a, b) => a.name.localeCompare(b.name));
    case "opened":
      return copy.sort((a, b) => (b.last_opened ?? "").localeCompare(a.last_opened ?? ""));
    default:
      return copy.sort((a, b) => b.detected_at.localeCompare(a.detected_at));
  }
}

export function ReposView() {
  const { items, groups, loading, scanning, progress, error, scan, fetch, openGroupDialog } =
    useReposStore();
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortKey>("recent");
  const [filter, setFilter] = useState<FilterKey>("all");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

  // Reload persisted repos on open, and ensure profiles are loaded so each
  // repo's assigned-profile badge resolves to a label.
  useEffect(() => {
    fetch();
    if (useProfilesStore.getState().items.length === 0) {
      useProfilesStore.getState().fetch();
    }
  }, [fetch]);

  const visible = useMemo(
    () =>
      sortRepos(
        items.filter((r) => matchesFilter(r, filter) && matchesQuery(r, query)),
        sort
      ),
    [items, filter, query, sort]
  );

  // Only hide empty group sections while the user is actively narrowing the
  // list; otherwise keep every group visible so it stays manageable.
  const narrowing = query !== "" || filter !== "all";

  const sections = useMemo(() => {
    const result: { key: string; group: (typeof groups)[number] | null; repos: Repo[] }[] = [];
    for (const group of groups) {
      const groupRepos = visible.filter((r) => r.group_id === group.id);
      if (narrowing && groupRepos.length === 0) continue;
      result.push({ key: group.id, group, repos: groupRepos });
    }
    const ungrouped = visible.filter((r) => r.group_id == null);
    if (ungrouped.length > 0 || (!narrowing && groups.length === 0)) {
      result.push({ key: UNGROUPED_KEY, group: null, repos: ungrouped });
    }
    return result;
  }, [groups, visible, narrowing]);

  const toggle = (key: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });

  const showScanning = scanning && items.length === 0;

  return (
    <div className="flex flex-col gap-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">Repositories</h1>
          <Badge variant="default">{items.length}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => openGroupDialog({ editing: null, assignRepoId: null })}
          >
            <FolderPlus className="size-3.5" aria-hidden="true" />
            New group
          </Button>
          <Button variant="primary" size="sm" onClick={scan} disabled={scanning}>
            {scanning ? <Spinner size="sm" /> : <Folder className="size-3.5" aria-hidden="true" />}
            {scanning ? "Scanning…" : "Scan folders"}
          </Button>
        </div>
      </div>

      {scanning && progress && (
        <div className="flex items-center gap-2 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-2) px-3 py-2 text-xs text-(--color-secondary)">
          <Spinner size="sm" />
          <span>
            Scanned {progress.scanned_dirs.toLocaleString()} folders · found {progress.found_repos}{" "}
            repositories
          </span>
        </div>
      )}

      <div className="flex items-center gap-2">
        <SearchInput
          value={query}
          onValueChange={setQuery}
          placeholder="Search repositories…"
          containerClassName="flex-1"
        />
        <select
          value={sort}
          onChange={(e) => setSort(e.target.value as SortKey)}
          className="h-8 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-2) px-2 text-xs text-(--color-fg) outline-none focus:border-(--color-brand-500)"
          aria-label="Sort repositories"
        >
          {SORTS.map((s) => (
            <option key={s.key} value={s.key}>
              {s.label}
            </option>
          ))}
        </select>
      </div>

      <div className="flex items-center gap-1">
        {FILTERS.map((f) => (
          <button
            key={f.key}
            type="button"
            onClick={() => setFilter(f.key)}
            className={cn(
              "rounded-full px-3 py-1 text-xs font-medium transition-colors",
              filter === f.key
                ? "bg-(--color-brand-500)/15 text-(--color-brand-500)"
                : "text-(--color-secondary) hover:bg-(--color-surface-2) hover:text-(--color-fg)"
            )}
          >
            {f.label}
          </button>
        ))}
      </div>

      {error ? (
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          <ErrorState error={error} onRetry={fetch} />
        </div>
      ) : loading || showScanning ? (
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          <LoadingState />
        </div>
      ) : items.length === 0 ? (
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          <EmptyState
            icon={<Folder className="size-8" aria-hidden="true" />}
            title="No repositories yet"
            description="Scan one or more folders to automatically discover the Git repositories on your machine."
            action={
              <Button variant="primary" size="sm" onClick={scan} disabled={scanning}>
                <MagnifierPlus className="size-3.5" aria-hidden="true" />
                Scan folders
              </Button>
            }
          />
        </div>
      ) : visible.length === 0 ? (
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          <NoResultsState query={query} />
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {sections.map((section) => (
            <RepoGroupSection
              key={section.key}
              group={section.group}
              repos={section.repos}
              collapsed={collapsed.has(section.key)}
              onToggle={() => toggle(section.key)}
            />
          ))}
        </div>
      )}

      <GroupFormDialog />
    </div>
  );
}
