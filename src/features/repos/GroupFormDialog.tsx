import { useEffect, useState } from "react";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { useReposStore } from "@/stores/repos";

const PRESET_COLORS = [
  "#10b981",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
  "#f59e0b",
  "#ef4444",
  "#14b8a6",
  "#6366f1",
];

export function GroupFormDialog() {
  const groupDialog = useReposStore((s) => s.groupDialog);
  const closeGroupDialog = useReposStore((s) => s.closeGroupDialog);
  const createGroup = useReposStore((s) => s.createGroup);
  const updateGroup = useReposStore((s) => s.updateGroup);
  const setGroup = useReposStore((s) => s.setGroup);

  const editing = groupDialog?.editing ?? null;
  const isEdit = editing !== null;

  const [name, setName] = useState("");
  const [color, setColor] = useState<string>(PRESET_COLORS[0]);
  const [submitting, setSubmitting] = useState(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: reseed keyed off the dialog opening, not read inside.
  useEffect(() => {
    setName(editing?.name ?? "");
    setColor(editing?.color ?? PRESET_COLORS[0]);
    setSubmitting(false);
  }, [groupDialog]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!groupDialog || submitting || name.trim().length === 0) return;
    setSubmitting(true);
    if (isEdit && editing) {
      const ok = await updateGroup(editing.id, name.trim(), color);
      setSubmitting(false);
      if (ok) closeGroupDialog();
      return;
    }
    const newId = await createGroup(name.trim(), color);
    if (newId && groupDialog.assignRepoId) {
      await setGroup(groupDialog.assignRepoId, newId);
    }
    setSubmitting(false);
    if (newId) closeGroupDialog();
  };

  return (
    <Dialog.Root open={groupDialog !== null} onOpenChange={(o) => !o && closeGroupDialog()}>
      <Dialog.Content
        title={isEdit ? "Edit Group" : "New Group"}
        description="Organize repositories into an ecosystem like “Dolphin Modules”."
      >
        <form onSubmit={onSubmit} className="flex flex-col gap-4">
          <Field label="Group name" htmlFor="group-name" required>
            <Input
              id="group-name"
              placeholder="Dolphin Modules, Joshtri Modules…"
              autoComplete="off"
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </Field>

          <Field label="Color">
            <div className="flex gap-2 flex-wrap">
              {PRESET_COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setColor(c)}
                  className="size-6 rounded-full transition-transform hover:scale-110 focus-visible:outline-2 focus-visible:outline-(--color-fg)"
                  style={{
                    backgroundColor: c,
                    boxShadow:
                      color === c ? `0 0 0 2px var(--color-surface), 0 0 0 4px ${c}` : undefined,
                  }}
                  aria-label={c}
                  aria-pressed={color === c}
                />
              ))}
            </div>
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={closeGroupDialog}>
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={submitting || name.trim().length === 0}
            >
              {submitting ? "Saving…" : isEdit ? "Save Changes" : "Create Group"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
