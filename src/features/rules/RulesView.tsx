import { FileArrowDown, FileArrowUp, Flask, Plus, Sliders } from "@gravity-ui/icons";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { EmptyState } from "@/components/feedback/EmptyState";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { NoResultsState } from "@/components/feedback/NoResultsState";
import { SearchInput } from "@/components/search-input";
import { Switch } from "@/components/switch";
import type { Rule } from "@/ipc/types.gen";
import { useProfilesStore } from "@/stores/profiles";
import { useRulesStore } from "@/stores/rules";
import { CreateRuleDialog } from "./CreateRuleDialog";
import { EditRuleDialog } from "./EditRuleDialog";
import { conditionSummary, SUBJECT_LABELS } from "./labels";
import { PreviewRuleDialog } from "./PreviewRuleDialog";
import { RuleActionsMenu } from "./RuleActionsMenu";

function matchesQuery(rule: Rule, q: string): boolean {
  if (q === "") return true;
  const needle = q.toLowerCase();
  return (
    rule.name.toLowerCase().includes(needle) ||
    rule.condition.value.toLowerCase().includes(needle) ||
    SUBJECT_LABELS[rule.condition.subject].toLowerCase().includes(needle)
  );
}

export function RulesView() {
  const {
    items,
    loading,
    error,
    fetch,
    createOpen,
    setCreateOpen,
    previewOpen,
    setPreviewOpen,
    setEnabled,
    exportRules,
    importRules,
  } = useRulesStore();
  const profiles = useProfilesStore((s) => s.items);
  const [query, setQuery] = useState("");

  useEffect(() => {
    fetch();
    if (useProfilesStore.getState().items.length === 0) {
      useProfilesStore.getState().fetch();
    }
  }, [fetch]);

  const profileLabel = (id: string) =>
    profiles.find((p) => p.id === id)?.label ?? "Unknown profile";

  const visible = useMemo(() => items.filter((r) => matchesQuery(r, query)), [items, query]);

  return (
    <div className="flex flex-col gap-4 max-w-2xl">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <h1 className="text-lg font-semibold text-(--color-fg)">Rules</h1>
          <Badge variant="default">{items.length}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={() => setPreviewOpen(true)}>
            <Flask className="size-3.5" aria-hidden="true" />
            Test
          </Button>
          <Button variant="secondary" size="sm" onClick={() => importRules(false)}>
            <FileArrowUp className="size-3.5" aria-hidden="true" />
            Import
          </Button>
          <Button variant="secondary" size="sm" onClick={exportRules} disabled={items.length === 0}>
            <FileArrowDown className="size-3.5" aria-hidden="true" />
            Export
          </Button>
          <Button variant="primary" size="sm" onClick={() => setCreateOpen(true)}>
            <Plus className="size-3.5" aria-hidden="true" />
            Add Rule
          </Button>
        </div>
      </div>

      <SearchInput value={query} onValueChange={setQuery} placeholder="Search rules…" />

      <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
        {error ? (
          <ErrorState error={error} onRetry={fetch} />
        ) : loading && items.length === 0 ? (
          <LoadingState />
        ) : items.length === 0 ? (
          <EmptyState
            icon={<Sliders className="size-8 text-(--color-muted)" aria-hidden="true" />}
            title="No rules yet"
            description="Create a rule to automatically resolve the right profile from a repository's path, name, or remote — no manual assignment needed."
            action={
              <Button variant="primary" size="sm" onClick={() => setCreateOpen(true)}>
                <Plus className="size-3.5" aria-hidden="true" />
                Add rule
              </Button>
            }
          />
        ) : visible.length === 0 ? (
          <NoResultsState query={query} />
        ) : (
          visible.map((rule, index) => (
            <div
              key={rule.id}
              className="flex items-start gap-4 px-4 py-3.5 border-b border-(--color-border) last:border-b-0"
            >
              <div className="flex size-9 shrink-0 items-center justify-center rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) text-xs font-semibold text-(--color-muted)">
                {index + 1}
              </div>

              <div className="flex flex-col gap-1 flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-medium text-(--color-fg) truncate">
                    {rule.name}
                  </span>
                  <Badge variant={rule.enabled ? "success" : "default"}>
                    {rule.enabled ? "Enabled" : "Disabled"}
                  </Badge>
                </div>
                <span className="text-xs text-(--color-muted) truncate">
                  {conditionSummary(rule)}
                </span>
                <span className="text-[11px] text-(--color-secondary) mt-0.5">
                  → {profileLabel(rule.target_profile_id)}
                </span>
              </div>

              <Switch
                checked={rule.enabled}
                onCheckedChange={(checked) => setEnabled(rule.id, checked)}
              />
              <RuleActionsMenu
                rule={rule}
                isFirst={index === 0}
                isLast={index === visible.length - 1}
              />
            </div>
          ))
        )}
      </div>

      <CreateRuleDialog open={createOpen} onOpenChange={setCreateOpen} />
      <EditRuleDialog />
      <PreviewRuleDialog open={previewOpen} onOpenChange={setPreviewOpen} />
    </div>
  );
}
