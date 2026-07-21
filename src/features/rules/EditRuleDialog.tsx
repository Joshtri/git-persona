import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect, useMemo } from "react";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { type RuleFormData, ruleSchema } from "@/lib/zod-schemas";
import { useProfilesStore } from "@/stores/profiles";
import { useRulesStore } from "@/stores/rules";
import { RuleBuilder } from "./RuleBuilder";

export function EditRuleDialog() {
  const editing = useRulesStore((s) => s.editing);
  const setEditing = useRulesStore((s) => s.setEditing);
  const update = useRulesStore((s) => s.update);
  const profiles = useProfilesStore((s) => s.items);

  const form = useForm<RuleFormData>({
    resolver: zodResolver(ruleSchema),
    defaultValues: {
      name: "",
      subject: "RepoPath",
      operator: "Contains",
      value: "",
      targetProfileId: "",
    },
  });

  useEffect(() => {
    if (editing) {
      form.reset({
        name: editing.name,
        subject: editing.condition.subject,
        operator: editing.condition.operator,
        value: editing.condition.value,
        targetProfileId: editing.target_profile_id,
      });
    }
  }, [editing, form]);

  const profileOptions = useMemo(
    () => profiles.map((p) => ({ value: p.id, label: p.label })),
    [profiles]
  );

  const close = () => setEditing(null);

  const onSubmit = async (data: RuleFormData) => {
    if (!editing) return;
    const ok = await update(editing.id, data);
    if (ok) close();
  };

  return (
    <Dialog.Root open={editing !== null} onOpenChange={(o) => !o && close()}>
      <Dialog.Content
        title="Edit Rule"
        description="Update this rule's condition or target profile."
      >
        <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <RuleBuilder form={form} profileOptions={profileOptions} />
          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={close}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? "Saving…" : "Save Changes"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
