import { useEffect, useState } from "react";
import { Avatar } from "@/components/avatar";
import { Badge } from "@/components/badge";
import { EmptyState } from "@/components/feedback/EmptyState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
// import { Avatar, Badge, SearchInput } from "@/components/ui";
import { useActivityStore } from "@/stores/activity";
import { useProfilesStore } from "@/stores/profiles";

function actionLabel(action: string): string {
  const map: Record<string, string> = {
    "profile.apply": "Applied profile globally",
    "profile.create": "Created profile",
    "profile.update": "Updated profile",
    "profile.delete": "Deleted profile",
    "ssh.import": "Imported SSH key",
    "ssh.generate": "Generated SSH key",
    "ssh.assign": "Assigned SSH key to profile",
    "ssh.remove": "Removed SSH key",
    "ssh.config.updated": "Updated SSH config",
  };
  return map[action] ?? action;
}

function actionVariant(action: string): "default" | "success" | "warning" | "danger" {
  if (action === "profile.apply") return "success";
  if (action === "profile.create") return "success";
  if (action === "profile.update") return "default";
  if (action === "profile.delete") return "danger";
  if (action === "ssh.import" || action === "ssh.generate") return "success";
  if (action === "ssh.remove") return "danger";
  if (action === "ssh.assign" || action === "ssh.config.updated") return "default";
  return "default";
}

function formatTimestamp(iso: string): string {
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    new Date(iso)
  );
}

function dayLabel(iso: string): string {
  const d = new Date(iso);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);
  if (d.toDateString() === today.toDateString()) return "Today";
  if (d.toDateString() === yesterday.toDateString()) return "Yesterday";
  return new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
  }).format(d);
}

export function ActivityView() {
  const { items, fetch } = useActivityStore();
  const { items: profiles } = useProfilesStore();
  const [query, setQuery] = useState("");

  useEffect(() => {
    fetch();
  }, [fetch]);

  const getProfileById = (id: string | undefined) =>
    id ? profiles.find((p) => p.id === id) : undefined;

  const filtered = items.filter((entry) => {
    if (!query) return true;
    const profile = getProfileById(entry.profile_id);
    return (
      actionLabel(entry.action).toLowerCase().includes(query.toLowerCase()) ||
      (profile?.label ?? "").toLowerCase().includes(query.toLowerCase()) ||
      (entry.repo_path ?? "").toLowerCase().includes(query.toLowerCase())
    );
  });

  const groups: Map<string, typeof filtered> = new Map();
  for (const entry of filtered) {
    const label = dayLabel(entry.timestamp);
    if (!groups.has(label)) groups.set(label, []);
    groups.get(label)?.push(entry);
  }

  return (
    <div className="flex flex-col gap-4 max-w-xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">Activity</h1>
          <Badge variant="default">{items.length}</Badge>
        </div>
      </div>

      <SearchInput value={query} onValueChange={setQuery} placeholder="Search activity…" />

      {items.length === 0 ? (
        <EmptyState
          title="No activity yet"
          description="Apply or modify profiles to start building an activity history."
        />
      ) : filtered.length === 0 ? (
        <NoResultsState query={query} />
      ) : (
        <div className="flex flex-col gap-4">
          {Array.from(groups.entries()).map(([day, entries]) => (
            <div key={day}>
              <p className="text-xs font-semibold text-(--color-muted) uppercase tracking-wider mb-2 px-1">
                {day}
              </p>
              <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
                {entries.map((entry) => {
                  const profile = getProfileById(entry.profile_id);
                  return (
                    <div
                      key={entry.id}
                      className="flex items-start gap-3.5 px-4 py-3 border-b border-(--color-border) last:border-b-0"
                    >
                      {profile ? (
                        <Avatar
                          name={profile.identity.name}
                          color={profile.color}
                          size="sm"
                          className="mt-0.5 shrink-0"
                        />
                      ) : (
                        <div className="size-6 rounded-full bg-(--color-surface-3) border border-(--color-border) shrink-0 mt-0.5" />
                      )}

                      <div className="flex flex-col gap-1 flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="text-sm text-(--color-fg)">
                            {actionLabel(entry.action)}
                          </span>
                          <Badge variant={actionVariant(entry.action)} className="text-[10px]">
                            {entry.action}
                          </Badge>
                        </div>
                        {profile && (
                          <div className="flex items-center gap-1.5">
                            <div
                              className="size-2 rounded-full shrink-0"
                              style={{ backgroundColor: profile.color }}
                            />
                            <span className="text-xs text-(--color-secondary)">
                              {profile.label}
                            </span>
                            <span className="text-xs text-(--color-muted)">·</span>
                            <span className="text-xs text-(--color-muted)">
                              {profile.identity.email}
                            </span>
                          </div>
                        )}
                        {entry.repo_path && (
                          <span className="text-xs text-(--color-muted) font-mono truncate">
                            {entry.repo_path}
                          </span>
                        )}
                      </div>

                      <span className="text-[10px] text-(--color-muted) shrink-0 mt-0.5">
                        {formatTimestamp(entry.timestamp)}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
