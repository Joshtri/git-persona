import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { hostGuidance, providerLabel } from "@/lib/credential-hosts";
import { type CredentialEditFormData, credentialEditSchema } from "@/lib/zod-schemas";
import { useCredentialsStore } from "@/stores/credentials";

export function EditCredentialDialog() {
  const editing = useCredentialsStore((s) => s.editing);
  const setEditing = useCredentialsStore((s) => s.setEditing);
  const update = useCredentialsStore((s) => s.update);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<CredentialEditFormData>({
    resolver: zodResolver(credentialEditSchema),
    defaultValues: { username: "", token: "" },
  });

  // Re-seed the form each time a different credential is opened for editing.
  useEffect(() => {
    if (editing) {
      reset({ username: editing.username, token: "" });
    }
  }, [editing, reset]);

  const close = () => setEditing(null);

  const onSubmit = async (data: CredentialEditFormData) => {
    if (!editing) return;
    const token = data.token?.trim() ? data.token : null;
    const ok = await update(editing.id, data.username.trim(), token);
    if (ok) close();
  };

  return (
    <Dialog.Root open={editing !== null} onOpenChange={(o) => !o && close()}>
      <Dialog.Content
        title="Edit Credential"
        description={
          editing
            ? `${providerLabel(editing.host)} · ${editing.host}`
            : "Update credential metadata."
        }
      >
        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <Field
            label="Username"
            htmlFor="cred-edit-username"
            error={errors.username?.message}
            required
          >
            <Input
              id="cred-edit-username"
              autoComplete="off"
              error={!!errors.username}
              {...register("username")}
            />
          </Field>

          <Field
            label={`Rotate ${editing ? hostGuidance(editing.host).tokenLabel : "token"} (optional)`}
            htmlFor="cred-edit-token"
            error={errors.token?.message}
            hint="Leave blank to keep the current token. A new token is written to the OS vault; apply or switch to make it active."
          >
            <Input
              id="cred-edit-token"
              type="password"
              placeholder="••••••••••••••••"
              autoComplete="off"
              className="font-mono text-xs"
              error={!!errors.token}
              {...register("token")}
            />
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={close}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={isSubmitting}>
              {isSubmitting ? "Saving…" : "Save Changes"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
