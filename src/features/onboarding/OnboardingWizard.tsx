import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { Switch } from "@/components/switch";
import type { DetectedSshKey } from "@/ipc/types.gen";
import { useOnboardingStore } from "@/stores/onboarding";

function fileName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function SshRow({ item }: { item: DetectedSshKey }) {
  const selected = useOnboardingStore((s) => s.selectedSsh.includes(item.private_key_path));
  const toggleSsh = useOnboardingStore((s) => s.toggleSsh);
  return (
    <div className="flex items-start justify-between gap-3 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-2) px-3 py-2">
      <div className="min-w-0">
        <p className="truncate text-sm font-medium text-(--color-fg)">
          {fileName(item.private_key_path)}
        </p>
        <p className="truncate text-xs text-(--color-muted)">
          {item.algorithm} · {item.fingerprint}
        </p>
      </div>
      <Switch checked={selected} onCheckedChange={() => toggleSsh(item.private_key_path)} />
    </div>
  );
}

export function OnboardingWizard() {
  const open = useOnboardingStore((s) => s.open);
  const scan = useOnboardingStore((s) => s.scan);
  const applying = useOnboardingStore((s) => s.applying);
  const createProfile = useOnboardingStore((s) => s.createProfile);
  const label = useOnboardingStore((s) => s.label);
  const name = useOnboardingStore((s) => s.name);
  const email = useOnboardingStore((s) => s.email);
  const signingKey = useOnboardingStore((s) => s.signingKey);
  const setCreateProfile = useOnboardingStore((s) => s.setCreateProfile);
  const setField = useOnboardingStore((s) => s.setField);
  const apply = useOnboardingStore((s) => s.apply);
  const skip = useOnboardingStore((s) => s.skip);

  const sshKeys = scan?.ssh_keys ?? [];
  const credentials = scan?.credentials ?? [];
  const nothingFound = !scan?.identity && sshKeys.length === 0 && credentials.length === 0;

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(next) => {
        if (!next) skip();
      }}
    >
      {open && (
        <Dialog.Content
          title="Welcome to GitPersona"
          description="We detected your existing Git setup. Pick what you'd like to import."
          className="max-w-lg"
        >
          <div className="flex max-h-[60vh] flex-col gap-5 overflow-y-auto pr-1">
            {nothingFound && (
              <p className="text-sm text-(--color-secondary)">
                Nothing to import was found on this machine. You can set everything up manually
                whenever you're ready.
              </p>
            )}

            {scan?.identity && (
              <section className="flex flex-col gap-3">
                <div className="flex items-center justify-between gap-3">
                  <h3 className="text-sm font-semibold text-(--color-fg)">Git identity</h3>
                  <Switch
                    id="onboarding-create-profile"
                    checked={createProfile}
                    onCheckedChange={setCreateProfile}
                    label="Create profile"
                  />
                </div>
                {createProfile && (
                  <div className="flex flex-col gap-3">
                    <Field label="Profile name" htmlFor="onb-label">
                      <Input
                        id="onb-label"
                        value={label}
                        placeholder="e.g. Work"
                        onChange={(e) => setField("label", e.target.value)}
                      />
                    </Field>
                    <div className="grid grid-cols-2 gap-3">
                      <Field label="Name" htmlFor="onb-name" required>
                        <Input
                          id="onb-name"
                          value={name}
                          onChange={(e) => setField("name", e.target.value)}
                        />
                      </Field>
                      <Field label="Email" htmlFor="onb-email" required>
                        <Input
                          id="onb-email"
                          value={email}
                          onChange={(e) => setField("email", e.target.value)}
                        />
                      </Field>
                    </div>
                    <Field label="Signing key" htmlFor="onb-key" hint="Optional">
                      <Input
                        id="onb-key"
                        value={signingKey}
                        onChange={(e) => setField("signingKey", e.target.value)}
                      />
                    </Field>
                  </div>
                )}
              </section>
            )}

            {sshKeys.length > 0 && (
              <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold text-(--color-fg)">SSH keys</h3>
                <div className="flex flex-col gap-2">
                  {sshKeys.map((k) => (
                    <SshRow key={k.private_key_path} item={k} />
                  ))}
                </div>
              </section>
            )}

            {credentials.length > 0 && (
              <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold text-(--color-fg)">Saved credentials</h3>
                <ul className="flex flex-col gap-1.5">
                  {credentials.map((c) => (
                    <li
                      key={c.host}
                      className="flex items-center justify-between rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-2) px-3 py-2 text-sm"
                    >
                      <span className="text-(--color-fg)">{c.host}</span>
                      <span className="text-(--color-muted)">@{c.username}</span>
                    </li>
                  ))}
                </ul>
                <p className="text-xs text-(--color-muted)">
                  Usernames only — add the tokens later from the Credentials page.
                </p>
              </section>
            )}
          </div>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => skip()} disabled={applying}>
              Skip
            </Button>
            <Button type="button" variant="primary" onClick={() => apply()} disabled={applying}>
              {nothingFound ? "Get started" : "Import selected"}
            </Button>
          </Dialog.Footer>
        </Dialog.Content>
      )}
    </Dialog.Root>
  );
}
