import { zodResolver } from "@hookform/resolvers/zod";
import { useMemo } from "react";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { type RuleFormData, ruleSchema } from "@/lib/zod-schemas";
import { useProfilesStore } from "@/stores/profiles";
import { useRulesStore } from "@/stores/rules";
import { RuleBuilder } from "./RuleBuilder";

const BLANK_RULE: RuleFormData = {
  name: "",
  subject: "RepoPath",
  operator: "Contains",
  value: "",
  targetProfileId: "",
};

export function RuleFormDialog() {
  const createOpen = useRulesStore((s) => s.createOpen);
  const setCreateOpen = useRulesStore((s) => s.setCreateOpen);
  const editing = useRulesStore((s) => s.editing);
  const setEditing = useRulesStore((s) => s.setEditing);
  const create = useRulesStore((s) => s.create);
  const update = useRulesStore((s) => s.update);
  const profiles = useProfilesStore((s) => s.items);

  const isEdit = editing !== null;

  const form = useForm<RuleFormData>({
    resolver: zodResolver(ruleSchema),
    defaultValues: editing
      ? {
          name: editing.name,
          subject: editing.condition.subject,
          operator: editing.condition.operator,
          value: editing.condition.value,
          targetProfileId: editing.target_profile_id,
        }
      : BLANK_RULE,
  });

  const profileOptions = useMemo(
    () => profiles.map((p) => ({ value: p.id, label: p.label })),
    [profiles]
  );

  const close = () => {
    if (isEdit) setEditing(null);
    else setCreateOpen(false);
  };

  const onSubmit = async (data: RuleFormData) => {
    const ok = editing ? await update(editing.id, data) : await create(data);
    if (ok) close();
  };

  return (
    <Dialog.Root open={createOpen || isEdit} onOpenChange={(o) => !o && close()}>
      <Dialog.Content
        title={isEdit ? "Edit Rule" : "Add Rule"}
        description={
          isEdit
            ? "Update this rule's condition or target profile."
            : "Automatically resolve a profile when a repository matches this condition."
        }
      >
        <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <RuleBuilder form={form} profileOptions={profileOptions} />
          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={close}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? "Saving…" : isEdit ? "Save Changes" : "Add Rule"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
