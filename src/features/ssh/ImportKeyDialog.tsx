import { FileArrowUp } from "@gravity-ui/icons";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { pickPrivateKeyFile } from "@/ipc";
import { type SshImportFormData, sshImportSchema } from "@/lib/zod-schemas";
import { useSshStore } from "@/stores/ssh";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ImportKeyDialog({ open, onOpenChange }: Props) {
  const importKey = useSshStore((s) => s.importKey);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<SshImportFormData>({
    resolver: zodResolver(sshImportSchema),
    defaultValues: { privateKeyPath: "", hostAlias: "", hostName: "" },
  });

  const path = watch("privateKeyPath");

  const chooseFile = async () => {
    const picked = await pickPrivateKeyFile();
    if (picked) {
      setValue("privateKeyPath", picked, { shouldValidate: true });
    }
  };

  const onSubmit = async (data: SshImportFormData) => {
    const ok = await importKey(
      data.privateKeyPath,
      data.hostAlias?.trim() || null,
      data.hostName?.trim() || null
    );
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
        title="Import SSH Key"
        description="GitPersona stores a reference to your key — the file is never copied or modified."
      >
        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <Field label="Private key file" error={errors.privateKeyPath?.message} required>
            <div className="flex items-center gap-2">
              <Input
                readOnly
                placeholder="No file selected"
                value={path}
                error={!!errors.privateKeyPath}
                className="flex-1 font-mono text-xs"
              />
              <Button type="button" variant="secondary" onClick={chooseFile}>
                <FileArrowUp className="size-3.5" aria-hidden="true" />
                Choose…
              </Button>
            </div>
          </Field>

          <Field
            label="Host alias (optional)"
            htmlFor="host-alias"
            error={errors.hostAlias?.message}
          >
            <Input id="host-alias" placeholder="github-work" {...register("hostAlias")} />
          </Field>

          <Field label="Host name (optional)" htmlFor="host-name" error={errors.hostName?.message}>
            <Input id="host-name" placeholder="github.com" {...register("hostName")} />
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={isSubmitting}>
              {isSubmitting ? "Importing…" : "Import Key"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
