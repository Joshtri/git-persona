import { Plus } from "@gravity-ui/icons";
import { useEffect, useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
import type { Profile } from "@/ipc/types.gen";
import { useProfilesStore } from "@/stores/profiles";
import { DeleteProfileDialog } from "./DeleteProfileDialog";
import { ProfileCard } from "./ProfileCard";
import { ProfileFormDialog } from "./ProfileFormDialog";

export function ProfilesView() {
  const { items, activeProfileId, loading, error, fetch, apply } = useProfilesStore();
  const [query, setQuery] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<Profile | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Profile | null>(null);

  useEffect(() => {
    fetch();
  }, [fetch]);

  const filtered = items.filter(
    (p) =>
      query === "" ||
      p.label.toLowerCase().includes(query.toLowerCase()) ||
      p.identity.name.toLowerCase().includes(query.toLowerCase()) ||
      p.identity.email.toLowerCase().includes(query.toLowerCase())
  );

  if (loading && items.length === 0) return <LoadingState />;
  if (error) return <ErrorState error={error} onRetry={fetch} />;

  return (
    <>
      <div className="flex flex-col gap-4 max-w-2xl">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <h1 className="text-lg font-semibold text-(--color-fg)">Profiles</h1>
            <Badge variant="default">{items.length}</Badge>
          </div>
          <Button variant="primary" size="sm" onClick={() => setCreateOpen(true)}>
            <Plus className="size-3.5" aria-hidden="true" />
            New Profile
          </Button>
        </div>

        <SearchInput value={query} onValueChange={setQuery} placeholder="Search profiles…" />

        {items.length === 0 ? (
          <EmptyState
            title="No profiles yet"
            description="Create your first profile to manage separate Git identities."
            action={
              <Button variant="primary" onClick={() => setCreateOpen(true)}>
                <Plus className="size-3.5" />
                Create Profile
              </Button>
            }
          />
        ) : filtered.length === 0 ? (
          <NoResultsState query={query} />
        ) : (
          <div className="grid grid-cols-2 gap-3">
            {filtered.map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                isActive={profile.id === activeProfileId}
                onEdit={(p) => setEditTarget(p)}
                onDelete={(p) => setDeleteTarget(p)}
                onApply={(p) => apply(p.id)}
              />
            ))}
          </div>
        )}
      </div>

      <ProfileFormDialog
        key={editTarget?.id ?? "new"}
        open={createOpen || !!editTarget}
        onOpenChange={(o) => {
          if (!o) {
            setCreateOpen(false);
            setEditTarget(null);
          } else setCreateOpen(true);
        }}
        editing={editTarget}
      />

      <DeleteProfileDialog
        open={!!deleteTarget}
        onOpenChange={(o) => {
          if (!o) setDeleteTarget(null);
        }}
        profile={deleteTarget}
      />
    </>
  );
}
