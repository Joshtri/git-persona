import { zodResolver } from "@hookform/resolvers/zod";
import { useMemo } from "react";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { type RuleFormData, ruleSchema } from "@/lib/zod-schemas";
import { useProfilesStore } from "@/stores/profiles";
import { useRulesStore } from "@/stores/rules";
import { RuleBuilder } from "./RuleBuilder";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreateRuleDialog({ open, onOpenChange }: Props) {
  const create = useRulesStore((s) => s.create);
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

  const profileOptions = useMemo(
    () => profiles.map((p) => ({ value: p.id, label: p.label })),
    [profiles]
  );

  const onSubmit = async (data: RuleFormData) => {
    const ok = await create(data);
    if (ok) {
      form.reset();
      onOpenChange(false);
    }
  };

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(o) => {
        if (!o) form.reset();
        onOpenChange(o);
      }}
    >
      <Dialog.Content
        title="Add Rule"
        description="Automatically resolve a profile when a repository matches this condition."
      >
        <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <RuleBuilder form={form} profileOptions={profileOptions} />
          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? "Saving…" : "Add Rule"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
