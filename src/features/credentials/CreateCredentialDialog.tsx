import { CircleInfo } from "@gravity-ui/icons";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { Select } from "@/features/inputs/non-form/select";
import { HOST_SELECT_OPTIONS, hostGuidance } from "@/lib/credential-hosts";
import { type CredentialFormData, credentialSchema } from "@/lib/zod-schemas";
import { useCredentialsStore } from "@/stores/credentials";
import { useProfilesStore } from "@/stores/profiles";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreateCredentialDialog({ open, onOpenChange }: Props) {
  const create = useCredentialsStore((s) => s.create);
  const profiles = useProfilesStore((s) => s.items);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<CredentialFormData>({
    resolver: zodResolver(credentialSchema),
    defaultValues: {
      host: "github.com",
      username: "",
      token: "",
      profileId: "",
    },
  });

  const host = watch("host");
  const profileId = watch("profileId");
  const guidance = hostGuidance(host);

  const profileOptions = [
    { value: "", label: "Unassigned" },
    ...profiles.map((p) => ({ value: p.id, label: p.label })),
  ];

  const onSubmit = async (data: CredentialFormData) => {
    const ok = await create({
      profileId: data.profileId ? data.profileId : null,
      host: data.host,
      username: data.username.trim(),
      token: data.token,
    });
    if (ok) {
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
        title="Add HTTPS Credential"
        description="Stored securely in the Windows Credential Manager. GitPersona keeps only metadata — never the token."
      >
        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <Field label="Host" error={errors.host?.message} required>
            <Select
              items={HOST_SELECT_OPTIONS}
              value={host}
              onValueChange={(v) => setValue("host", v as CredentialFormData["host"])}
              className="w-full"
            />
          </Field>

          <Field label="Username" htmlFor="cred-username" error={errors.username?.message} required>
            <Input
              id="cred-username"
              placeholder="your-git-username"
              autoComplete="off"
              error={!!errors.username}
              {...register("username")}
            />
          </Field>

          <Field
            label={guidance.tokenLabel}
            htmlFor="cred-token"
            error={errors.token?.message}
            hint="Written straight to the OS vault — GitPersona never keeps a copy."
            required
          >
            <Input
              id="cred-token"
              type="password"
              placeholder="••••••••••••••••"
              autoComplete="off"
              className="font-mono text-xs"
              error={!!errors.token}
              {...register("token")}
            />
          </Field>

          {guidance.note && (
            <div className="flex items-start gap-2 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) px-3 py-2 -mt-1">
              <CircleInfo
                className="size-3.5 text-(--color-muted) shrink-0 mt-0.5"
                aria-hidden="true"
              />
              <p className="text-[11px] leading-relaxed text-(--color-secondary)">
                {guidance.note}
              </p>
            </div>
          )}

          <Field label="Assign to profile (optional)">
            <Select
              items={profileOptions}
              value={profileId ?? ""}
              onValueChange={(v) => setValue("profileId", v)}
              placeholder="Unassigned"
              className="w-full"
            />
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={isSubmitting}>
              {isSubmitting ? "Saving…" : "Add Credential"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
