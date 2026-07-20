import { Lock, Plus, TrashBin } from "@gravity-ui/icons";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { Tooltip } from "@/components/tooltip";
// import { Badge, Button, Tooltip } from "@/components/ui";
import { getProfileById, MOCK_CREDENTIALS } from "@/lib/mock-data";

export function CredentialsView() {
  return (
    <div className="flex flex-col gap-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">Credentials</h1>
          <Badge variant="default">{MOCK_CREDENTIALS.length}</Badge>
        </div>
        <Tooltip content="Credential management coming in Sprint 2" side="left">
          <Button variant="primary" size="sm" disabled>
            <Plus className="size-3.5" aria-hidden="true" />
            Add Credential
          </Button>
        </Tooltip>
      </div>

      <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
        {MOCK_CREDENTIALS.length === 0 ? (
          <EmptyState
            icon={<Lock className="size-8 text-(--color-muted)" />}
            title="No credentials stored"
            description="Store HTTPS credentials and tokens per profile."
          />
        ) : (
          MOCK_CREDENTIALS.map((cred) => {
            const profile = getProfileById(cred.profile_id ?? "");
            return (
              <div
                key={cred.id}
                className="flex items-start gap-4 px-4 py-3.5 border-b border-(--color-border) last:border-b-0"
              >
                <div className="size-9 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) flex items-center justify-center shrink-0">
                  <Lock className="size-4 text-(--color-muted)" aria-hidden="true" />
                </div>

                <div className="flex flex-col gap-1 flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-(--color-fg)">{cred.name}</span>
                    <Badge variant="default" className="text-[10px]">
                      {cred.type}
                    </Badge>
                    {profile && (
                      <Badge
                        variant="default"
                        className="text-[10px]"
                        style={
                          {
                            color: profile.color,
                            backgroundColor: `${profile.color}18`,
                          } as React.CSSProperties
                        }
                      >
                        {profile.label}
                      </Badge>
                    )}
                  </div>
                  <span className="text-xs text-(--color-muted)">{cred.host}</span>
                  <div className="flex items-center gap-2 mt-0.5">
                    <span className="text-xs text-(--color-muted) font-mono">@{cred.username}</span>
                    <span className="text-[10px] text-(--color-muted)">·</span>
                    <span className="text-[10px] text-(--color-muted)">Added {cred.created}</span>
                  </div>
                </div>

                <button
                  type="button"
                  disabled
                  className="p-1.5 rounded-(--radius-sm) text-(--color-muted) opacity-40 cursor-not-allowed"
                  aria-label="Remove credential"
                >
                  <TrashBin className="size-3.5" />
                </button>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
