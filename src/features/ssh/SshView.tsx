import { FileArrowUp, Key, Plus } from "@gravity-ui/icons";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
import type { SshKey } from "@/ipc/types.gen";
import { useProfilesStore } from "@/stores/profiles";
import { useSshStore } from "@/stores/ssh";
import { AssignSshProfileMenu } from "./AssignSshProfileMenu";
import { GenerateKeyDialog } from "./GenerateKeyDialog";
import { ImportKeyDialog } from "./ImportKeyDialog";
import { SshActionsMenu } from "./SshActionsMenu";

function lastUsedLabel(iso: string | null): string {
  if (!iso) return "Never used";
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "Used just now";
  if (mins < 60) return `Used ${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `Used ${hours}h ago`;
  return `Used ${Math.floor(hours / 24)}d ago`;
}

function matchesQuery(key: SshKey, q: string): boolean {
  if (q === "") return true;
  const needle = q.toLowerCase();
  return (
    key.label.toLowerCase().includes(needle) ||
    key.fingerprint.toLowerCase().includes(needle) ||
    key.private_key_path.toLowerCase().includes(needle) ||
    (key.comment?.toLowerCase().includes(needle) ?? false)
  );
}

export function SshView() {
  const {
    items,
    loading,
    error,
    fetch,
    scan,
    importOpen,
    generateOpen,
    setImportOpen,
    setGenerateOpen,
  } = useSshStore();
  const [query, setQuery] = useState("");

  // Load persisted keys, discover any new ones in ~/.ssh, and ensure profiles
  // are available so each key's assigned-profile badge resolves.
  useEffect(() => {
    scan();
    if (useProfilesStore.getState().items.length === 0) {
      useProfilesStore.getState().fetch();
    }
  }, [scan]);

  const visible = useMemo(() => items.filter((k) => matchesQuery(k, query)), [items, query]);

  return (
    <div className="flex flex-col gap-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">SSH Keys</h1>
          <Badge variant="default">{items.length}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={() => setImportOpen(true)}>
            <FileArrowUp className="size-3.5" aria-hidden="true" />
            Import
          </Button>
          <Button variant="primary" size="sm" onClick={() => setGenerateOpen(true)}>
            <Plus className="size-3.5" aria-hidden="true" />
            Generate
          </Button>
        </div>
      </div>

      <SearchInput value={query} onValueChange={setQuery} placeholder="Search SSH keys…" />

      <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
        {error ? (
          <ErrorState error={error} onRetry={fetch} />
        ) : loading && items.length === 0 ? (
          <LoadingState />
        ) : items.length === 0 ? (
          <EmptyState
            icon={<Key className="size-8 text-(--color-muted)" aria-hidden="true" />}
            title="No SSH keys"
            description="Import an existing key or generate a new one to use with your Git profiles."
            action={
              <Button variant="primary" size="sm" onClick={() => setGenerateOpen(true)}>
                <Plus className="size-3.5" aria-hidden="true" />
                Generate key
              </Button>
            }
          />
        ) : visible.length === 0 ? (
          <NoResultsState query={query} />
        ) : (
          visible.map((key) => (
            <div
              key={key.id}
              className="flex items-start gap-4 px-4 py-3.5 border-b border-(--color-border) last:border-b-0"
            >
              <div className="size-9 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) flex items-center justify-center shrink-0">
                <Key className="size-4 text-(--color-muted)" aria-hidden="true" />
              </div>

              <div className="flex flex-col gap-1 flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-medium text-(--color-fg) truncate">
                    {key.label}
                  </span>
                  <Badge variant="default" className="text-[10px]">
                    {key.algorithm === "Ed25519" ? "ED25519" : "RSA"}
                  </Badge>
                  {key.imported && (
                    <Badge variant="default" className="text-[10px]">
                      Imported
                    </Badge>
                  )}
                  <AssignSshProfileMenu sshKey={key} />
                </div>
                <span className="text-xs text-(--color-muted) font-mono truncate">
                  {key.fingerprint}
                </span>
                <div className="flex items-center gap-2 mt-0.5 min-w-0">
                  <span className="text-[10px] text-(--color-muted) font-mono truncate">
                    {key.private_key_path}
                  </span>
                  <span className="text-[10px] text-(--color-muted) shrink-0">·</span>
                  <span className="text-[10px] text-(--color-muted) shrink-0">
                    {lastUsedLabel(key.last_used)}
                  </span>
                </div>
              </div>

              <SshActionsMenu sshKey={key} />
            </div>
          ))
        )}
      </div>

      <ImportKeyDialog open={importOpen} onOpenChange={setImportOpen} />
      <GenerateKeyDialog open={generateOpen} onOpenChange={setGenerateOpen} />
    </div>
  );
}
