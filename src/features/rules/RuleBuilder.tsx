import type { UseFormReturn } from "react-hook-form";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import type { SelectOption } from "@/features/inputs/non-form/select";
import { Select } from "@/features/inputs/non-form/select";
import type { RuleFormData } from "@/lib/zod-schemas";
import { EQUALS_ONLY_SUBJECTS, operatorOptionsFor, SUBJECT_OPTIONS } from "./labels";

interface Props {
  form: UseFormReturn<RuleFormData>;
  profileOptions: SelectOption[];
}

/**
 * The visual IF/THEN rule builder. Users never edit JSON — they pick a subject,
 * an operator, and a value, then a target profile. Operators are constrained to
 * the ones legal for the chosen subject.
 */
export function RuleBuilder({ form, profileOptions }: Props) {
  const {
    register,
    watch,
    setValue,
    formState: { errors },
  } = form;

  const subject = watch("subject");
  const operator = watch("operator");
  const targetProfileId = watch("targetProfileId");
  const operatorOptions = operatorOptionsFor(subject);

  const onSubjectChange = (next: string) => {
    const subjectValue = next as RuleFormData["subject"];
    setValue("subject", subjectValue, { shouldValidate: true });
    // Host/owner support only "equals" — snap the operator back if needed.
    if (EQUALS_ONLY_SUBJECTS.includes(subjectValue) && operator !== "Equals") {
      setValue("operator", "Equals", { shouldValidate: true });
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Field label="Name" htmlFor="rule-name" error={errors.name?.message} required>
        <Input
          id="rule-name"
          placeholder="e.g. Company projects"
          autoComplete="off"
          error={!!errors.name}
          {...register("name")}
        />
      </Field>

      <div className="rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) p-3 flex flex-col gap-3">
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-(--color-muted)">
            If
          </span>
          <div className="h-px flex-1 bg-(--color-border)" />
        </div>

        <div className="grid grid-cols-[1fr_auto] gap-2">
          <Field label="Subject" error={errors.subject?.message}>
            <Select
              items={SUBJECT_OPTIONS}
              value={subject}
              onValueChange={onSubjectChange}
              className="w-full"
            />
          </Field>
          <Field label="Operator" error={errors.operator?.message}>
            <Select
              items={operatorOptions}
              value={operator}
              onValueChange={(v) =>
                setValue("operator", v as RuleFormData["operator"], { shouldValidate: true })
              }
            />
          </Field>
        </div>

        <Field label="Value" htmlFor="rule-value" error={errors.value?.message} required>
          <Input
            id="rule-value"
            placeholder="e.g. /company/"
            autoComplete="off"
            error={!!errors.value}
            {...register("value")}
          />
        </Field>

        <div className="flex items-center gap-2 pt-1">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-(--color-muted)">
            Then apply
          </span>
          <div className="h-px flex-1 bg-(--color-border)" />
        </div>

        <Field label="Profile" error={errors.targetProfileId?.message} required>
          <Select
            items={profileOptions}
            value={targetProfileId}
            onValueChange={(v) => setValue("targetProfileId", v, { shouldValidate: true })}
            placeholder="Select profile"
            className="w-full"
          />
        </Field>
      </div>
    </div>
  );
}
