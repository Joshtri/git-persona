import { Key, Lock, Plus } from "@gravity-ui/icons";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
import type { Credential } from "@/ipc/types.gen";
import { providerLabel } from "@/lib/credential-hosts";
import { useCredentialsStore } from "@/stores/credentials";
import { useProfilesStore } from "@/stores/profiles";
import { AssignCredentialProfileMenu } from "./AssignCredentialProfileMenu";
import { CreateCredentialDialog } from "./CreateCredentialDialog";
import { CredentialActionsMenu } from "./CredentialActionsMenu";
import { EditCredentialDialog } from "./EditCredentialDialog";

function updatedLabel(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "Updated just now";
  if (mins < 60) return `Updated ${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `Updated ${hours}h ago`;
  return `Updated ${Math.floor(hours / 24)}d ago`;
}

function matchesQuery(credential: Credential, q: string): boolean {
  if (q === "") return true;
  const needle = q.toLowerCase();
  return (
    credential.host.toLowerCase().includes(needle) ||
    credential.username.toLowerCase().includes(needle) ||
    providerLabel(credential.host).toLowerCase().includes(needle)
  );
}

export function CredentialsView() {
  const { items, loading, error, fetch, createOpen, setCreateOpen, openManager } =
    useCredentialsStore();
  const [query, setQuery] = useState("");

  // Load credentials and make sure profiles exist so assignment badges resolve.
  useEffect(() => {
    fetch();
    if (useProfilesStore.getState().items.length === 0) {
      useProfilesStore.getState().fetch();
    }
  }, [fetch]);

  const visible = useMemo(() => items.filter((c) => matchesQuery(c, query)), [items, query]);

  return (
    <div className="flex flex-col gap-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">Credentials</h1>
          <Badge variant="default">{items.length}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={openManager}>
            <Key className="size-3.5" aria-hidden="true" />
            Credential Manager
          </Button>
          <Button variant="primary" size="sm" onClick={() => setCreateOpen(true)}>
            <Plus className="size-3.5" aria-hidden="true" />
            Add Credential
          </Button>
        </div>
      </div>

      <SearchInput value={query} onValueChange={setQuery} placeholder="Search credentials…" />

      <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
        {error ? (
          <ErrorState error={error} onRetry={fetch} />
        ) : loading && items.length === 0 ? (
          <LoadingState />
        ) : items.length === 0 ? (
          <EmptyState
            icon={<Lock className="size-8 text-(--color-muted)" aria-hidden="true" />}
            title="No credentials"
            description="Add an HTTPS token so applying a profile can switch your Git credential automatically."
            action={
              <Button variant="primary" size="sm" onClick={() => setCreateOpen(true)}>
                <Plus className="size-3.5" aria-hidden="true" />
                Add credential
              </Button>
            }
          />
        ) : visible.length === 0 ? (
          <NoResultsState query={query} />
        ) : (
          visible.map((credential) => (
            <div
              key={credential.id}
              className="flex items-start gap-4 px-4 py-3.5 border-b border-(--color-border) last:border-b-0"
            >
              <div className="size-9 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) flex items-center justify-center shrink-0">
                <Lock className="size-4 text-(--color-muted)" aria-hidden="true" />
              </div>

              <div className="flex flex-col gap-1 flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-medium text-(--color-fg) truncate">
                    {providerLabel(credential.host)}
                  </span>
                  <Badge variant="default" className="text-[10px]">
                    HTTPS
                  </Badge>
                  <AssignCredentialProfileMenu credential={credential} />
                </div>
                <span className="text-xs text-(--color-muted)">{credential.host}</span>
                <div className="flex items-center gap-2 mt-0.5 min-w-0">
                  <span className="text-xs text-(--color-muted) font-mono truncate">
                    @{credential.username}
                  </span>
                  <span className="text-[10px] text-(--color-muted) shrink-0">·</span>
                  <span className="text-[10px] text-(--color-muted) shrink-0">
                    {updatedLabel(credential.updated_at)}
                  </span>
                </div>
              </div>

              <CredentialActionsMenu credential={credential} />
            </div>
          ))
        )}
      </div>

      <CreateCredentialDialog open={createOpen} onOpenChange={setCreateOpen} />
      <EditCredentialDialog />
    </div>
  );
}
