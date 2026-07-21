import { CircleCheck, CircleXmark } from "@gravity-ui/icons";
import { useState } from "react";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { useProfilesStore } from "@/stores/profiles";
import { useRulesStore } from "@/stores/rules";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Test a hypothetical repository against the rules. This is preview-only — it
 * never applies a profile or touches Git configuration.
 */
export function PreviewRuleDialog({ open, onOpenChange }: Props) {
  const { preview, runPreview } = useRulesStore();
  const profiles = useProfilesStore((s) => s.items);
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [remoteUrl, setRemoteUrl] = useState("");

  const profileLabel = (id: string) => profiles.find((p) => p.id === id)?.label ?? id;

  const close = () => {
    onOpenChange(false);
    setPath("");
    setName("");
    setRemoteUrl("");
  };

  const onCheck = () => {
    void runPreview({ path, name, remoteUrl });
  };

  return (
    <Dialog.Root open={open} onOpenChange={(o) => (o ? onOpenChange(o) : close())}>
      <Dialog.Content
        title="Test Rules"
        description="See which rule would match a repository — without changing any Git configuration."
      >
        <div className="flex flex-col gap-4">
          <Field label="Repository path" htmlFor="preview-path">
            <Input
              id="preview-path"
              placeholder="/home/dev/company/api"
              autoComplete="off"
              value={path}
              onChange={(e) => setPath(e.target.value)}
            />
          </Field>
          <Field label="Repository name" htmlFor="preview-name">
            <Input
              id="preview-name"
              placeholder="api"
              autoComplete="off"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </Field>
          <Field label="Remote URL (optional)" htmlFor="preview-remote">
            <Input
              id="preview-remote"
              placeholder="git@github.com:my-company/api.git"
              autoComplete="off"
              className="font-mono text-xs"
              value={remoteUrl}
              onChange={(e) => setRemoteUrl(e.target.value)}
            />
          </Field>

          {preview &&
            (preview.matched ? (
              <div className="flex items-start gap-3 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) px-3 py-3">
                <CircleCheck
                  className="size-4 text-(--color-success) shrink-0 mt-0.5"
                  aria-hidden="true"
                />
                <div className="flex flex-col gap-1 min-w-0">
                  <span className="text-sm font-medium text-(--color-fg)">
                    Matched “{preview.matched.rule_name}”
                  </span>
                  <span className="text-xs text-(--color-secondary)">
                    Applies profile{" "}
                    <span className="font-medium text-(--color-fg)">
                      {profileLabel(preview.matched.profile_id)}
                    </span>
                  </span>
                  <span className="text-[11px] text-(--color-muted)">{preview.matched.reason}</span>
                </div>
              </div>
            ) : (
              <div className="flex items-start gap-3 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) px-3 py-3">
                <CircleXmark
                  className="size-4 text-(--color-muted) shrink-0 mt-0.5"
                  aria-hidden="true"
                />
                <div className="flex flex-col gap-1">
                  <span className="text-sm font-medium text-(--color-fg)">No rule matched</span>
                  <span className="text-[11px] text-(--color-muted)">
                    The repository would fall back to its manual profile assignment.
                  </span>
                </div>
              </div>
            ))}

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={close}>
              Close
            </Button>
            <Button type="button" variant="primary" onClick={onCheck} disabled={!path && !name}>
              Test
            </Button>
          </Dialog.Footer>
        </div>
      </Dialog.Content>
    </Dialog.Root>
  );
}
