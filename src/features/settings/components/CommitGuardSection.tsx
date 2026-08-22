import { useEffect } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { Switch } from "@/components/switch";
import type { GuardMode, HookState } from "@/ipc/types.gen";
import { useCommitGuardStore } from "@/stores/commitGuard";
import { Select, type SelectOption } from "../../inputs/non-form/select";
import { SectionHeader } from "./Section.Header";
import { SettingRow } from "./Section.SettingRow";

const MODE_OPTIONS: SelectOption[] = [
  { label: "Warn only", value: "Warn" },
  { label: "Block commit", value: "Block" },
];

function HookBadge({ state }: { state: HookState }) {
  switch (state) {
    case "Managed":
    case "ManagedChained":
      return <Badge variant="success">Protected</Badge>;
    case "Foreign":
      return <Badge variant="default">Existing hook</Badge>;
    case "Unsupported":
      return <Badge variant="warning">Unsupported</Badge>;
    default:
      return <Badge variant="default">Not protected</Badge>;
  }
}

/** Commit Guard settings + per-repository protection manager. Presented as the
 *  "final safety layer" that complements Smart Switching — never a duplicate. */
export function CommitGuardSection() {
  const { status, init, setEnabled, setMode, setAutoProtect, install, repair, uninstall } =
    useCommitGuardStore();

  useEffect(() => {
    init();
  }, [init]);

  const enabled = status?.enabled ?? false;
  const mode = status?.mode ?? "Warn";
  const autoProtect = status?.auto_protect ?? false;
  const repos = status?.repos ?? [];

  return (
    <section>
      <SectionHeader
        title="Commit Guard"
        description="A final safety check before each commit. Smart Switching keeps the right identity active; Commit Guard verifies it at commit time and warns — or blocks — on a mismatch."
      />
      <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
        <SettingRow
          label="Enable Commit Guard"
          description="Install a managed pre-commit hook in protected repositories. Any existing hook is preserved and chained."
        >
          <Switch checked={enabled} onCheckedChange={(v) => setEnabled(v)} />
        </SettingRow>
        <SettingRow
          label="On identity mismatch"
          description="Warn only lets the commit proceed; Block stops it (override once with git commit --no-verify)."
        >
          <Select
            items={MODE_OPTIONS}
            value={mode}
            onValueChange={(v) => setMode(v as GuardMode)}
            disabled={!enabled}
          />
        </SettingRow>
        <SettingRow
          label="Auto-protect assigned repositories"
          description="Install the hook automatically when a profile is assigned to a repository."
        >
          <Switch
            checked={autoProtect}
            onCheckedChange={(v) => setAutoProtect(v)}
            disabled={!enabled}
          />
        </SettingRow>
        <SettingRow
          label="Protected repositories"
          description={`${status?.protected_count ?? 0} of ${repos.length} tracked ${
            repos.length === 1 ? "repository" : "repositories"
          } protected.`}
        >
          <Badge variant="default">{status?.protected_count ?? 0}</Badge>
        </SettingRow>
      </div>

      {enabled && repos.length > 0 && (
        <div className="mt-3 max-h-72 overflow-y-auto rounded-(--radius-xl) border border-(--color-border) bg-(--color-surface) divide-y divide-(--color-border)">
          {repos.map((r) => (
            <div key={r.repo_id} className="flex items-center justify-between gap-3 px-4 py-3">
              <div className="min-w-0">
                <div className="flex items-center gap-2">
                  <span className="truncate text-sm font-medium">{r.repo_name}</span>
                  {r.identity_matches === false && (
                    <span
                      className="size-1.5 shrink-0 rounded-full bg-(--color-warning)"
                      title="Current identity does not match the expected profile"
                    />
                  )}
                  {r.identity_matches === true && (
                    <span
                      className="size-1.5 shrink-0 rounded-full bg-(--color-success)"
                      title="Current identity matches the expected profile"
                    />
                  )}
                </div>
                <div className="truncate text-xs text-(--color-secondary)">
                  {r.expected_label ? `Expected: ${r.expected_label}` : "No expected profile"}
                </div>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <HookBadge state={r.hook_state} />
                {r.hook_state === "Unsupported" ? null : r.protected ? (
                  <>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => repair(r.repo_id)}
                    >
                      Repair
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => uninstall(r.repo_id)}
                    >
                      Remove
                    </Button>
                  </>
                ) : (
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => install(r.repo_id)}
                  >
                    Protect
                  </Button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
