import { Folder } from "@gravity-ui/icons";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { pickDirectory } from "@/ipc";
import { cn } from "@/lib/cn";
import { type SshGenerateFormData, sshGenerateSchema } from "@/lib/zod-schemas";
import { useSshStore } from "@/stores/ssh";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const ALGORITHMS: { value: "Ed25519" | "Rsa"; label: string; hint: string }[] = [
  { value: "Ed25519", label: "ED25519", hint: "Recommended" },
  { value: "Rsa", label: "RSA 4096", hint: "Compatibility" },
];

export function GenerateKeyDialog({ open, onOpenChange }: Props) {
  const generate = useSshStore((s) => s.generate);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<SshGenerateFormData>({
    resolver: zodResolver(sshGenerateSchema),
    defaultValues: {
      label: "",
      algorithm: "Ed25519",
      comment: "",
      outputDir: "",
      fileName: "id_ed25519",
      hostAlias: "",
      hostName: "",
    },
  });

  const algorithm = watch("algorithm");
  const outputDir = watch("outputDir");

  const chooseDir = async () => {
    const picked = await pickDirectory();
    if (picked) {
      setValue("outputDir", picked, { shouldValidate: true });
    }
  };

  const onSubmit = async (data: SshGenerateFormData) => {
    const ok = await generate({
      label: data.label,
      algorithm: data.algorithm,
      comment: data.comment?.trim() || null,
      outputDir: data.outputDir,
      fileName: data.fileName,
      hostAlias: data.hostAlias?.trim() || null,
      hostName: data.hostName?.trim() || null,
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
        title="Generate SSH Key"
        description="Creates a new key pair on disk using platform-native cryptography."
      >
        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <Field label="Label" htmlFor="gen-label" error={errors.label?.message} required>
            <Input
              id="gen-label"
              placeholder="Work, Personal…"
              error={!!errors.label}
              {...register("label")}
            />
          </Field>

          <Field label="Algorithm">
            <div className="flex gap-2">
              {ALGORITHMS.map((a) => (
                <button
                  key={a.value}
                  type="button"
                  onClick={() => setValue("algorithm", a.value)}
                  className={cn(
                    "flex flex-1 flex-col items-start gap-0.5 rounded-(--radius-md) border px-3 py-2 text-left transition-colors",
                    algorithm === a.value
                      ? "border-(--color-brand-500) bg-(--color-brand-500)/10"
                      : "border-(--color-border) hover:bg-(--color-surface-2)"
                  )}
                  aria-pressed={algorithm === a.value}
                >
                  <span className="text-sm font-medium text-(--color-fg)">{a.label}</span>
                  <span className="text-[10px] text-(--color-muted)">{a.hint}</span>
                </button>
              ))}
            </div>
          </Field>

          <Field label="Output folder" error={errors.outputDir?.message} required>
            <div className="flex items-center gap-2">
              <Input
                readOnly
                placeholder="No folder selected"
                value={outputDir}
                error={!!errors.outputDir}
                className="flex-1 font-mono text-xs"
              />
              <Button type="button" variant="secondary" onClick={chooseDir}>
                <Folder className="size-3.5" aria-hidden="true" />
                Choose…
              </Button>
            </div>
          </Field>

          <Field label="File name" htmlFor="gen-file" error={errors.fileName?.message} required>
            <Input
              id="gen-file"
              placeholder="id_ed25519"
              error={!!errors.fileName}
              className="font-mono text-xs"
              {...register("fileName")}
            />
          </Field>

          <Field label="Comment (optional)" htmlFor="gen-comment" error={errors.comment?.message}>
            <Input id="gen-comment" placeholder="you@example.com" {...register("comment")} />
          </Field>

          <div className="grid grid-cols-2 gap-3">
            <Field
              label="Host alias (optional)"
              htmlFor="gen-host-alias"
              error={errors.hostAlias?.message}
            >
              <Input id="gen-host-alias" placeholder="github-work" {...register("hostAlias")} />
            </Field>
            <Field
              label="Host name (optional)"
              htmlFor="gen-host-name"
              error={errors.hostName?.message}
            >
              <Input id="gen-host-name" placeholder="github.com" {...register("hostName")} />
            </Field>
          </div>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={isSubmitting}>
              {isSubmitting ? "Generating…" : "Generate Key"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
