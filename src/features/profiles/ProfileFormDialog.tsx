import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
// import { Button, Dialog, Field, Input } from "@/components/ui";
import type { Profile } from "@/ipc/types.gen";
import { type ProfileFormData, profileSchema } from "@/lib/zod-schemas";
import { useProfilesStore } from "@/stores/profiles";

const PRESET_COLORS = [
  "#6366f1",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
  "#14b8a6",
];

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editing?: Profile | null;
}

export function ProfileFormDialog({ open, onOpenChange, editing }: Props) {
  const { create, update } = useProfilesStore();
  const isEdit = !!editing;

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<ProfileFormData>({
    resolver: zodResolver(profileSchema),
    defaultValues: editing
      ? { label: editing.label, identity: editing.identity, color: editing.color }
      : { label: "", identity: { name: "", email: "" }, color: "#6366f1" },
  });

  const selectedColor = watch("color") ?? "#6366f1";

  const onSubmit = async (data: ProfileFormData) => {
    const { label, identity, color } = data;
    let success: boolean;
    if (editing) {
      success = await update(
        editing.id,
        label,
        identity.name,
        identity.email,
        identity.signing_key,
        color
      );
    } else {
      success = await create(label, identity.name, identity.email, identity.signing_key, color);
    }
    if (success) {
      reset();
      onOpenChange(false);
    }
  };

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(o) => {
        if (!o) reset();
        onOpenChange(o);
      }}
    >
      <Dialog.Content
        title={isEdit ? `Edit ${editing?.label}` : "New Profile"}
        description="Profiles store a Git identity you can switch between instantly."
      >
        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <Field label="Label" htmlFor="label" error={errors.label?.message} required>
            <Input
              id="label"
              placeholder="Work, Personal, OSS…"
              error={!!errors.label}
              {...register("label")}
            />
          </Field>

          <Field
            label="Full Name"
            htmlFor="identity-name"
            error={errors.identity?.name?.message}
            required
          >
            <Input
              id="identity-name"
              placeholder="Jane Doe"
              error={!!errors.identity?.name}
              {...register("identity.name")}
            />
          </Field>

          <Field
            label="Email"
            htmlFor="identity-email"
            error={errors.identity?.email?.message}
            required
          >
            <Input
              id="identity-email"
              type="email"
              placeholder="jane@example.com"
              error={!!errors.identity?.email}
              {...register("identity.email")}
            />
          </Field>

          <Field label="Signing Key (optional)" htmlFor="signing-key">
            <Input
              id="signing-key"
              placeholder="GPG key ID or fingerprint"
              {...register("identity.signing_key")}
            />
          </Field>

          <Field label="Color">
            <div className="flex gap-2 flex-wrap">
              {PRESET_COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setValue("color", c)}
                  className="size-6 rounded-full transition-transform hover:scale-110 focus-visible:outline-2 focus-visible:outline-(--color-fg)"
                  style={{
                    backgroundColor: c,
                    boxShadow:
                      selectedColor === c
                        ? `0 0 0 2px var(--color-surface), 0 0 0 4px ${c}`
                        : undefined,
                  }}
                  aria-label={c}
                  aria-pressed={selectedColor === c}
                />
              ))}
            </div>
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={isSubmitting}>
              {isSubmitting ? "Saving…" : isEdit ? "Save Changes" : "Create Profile"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
