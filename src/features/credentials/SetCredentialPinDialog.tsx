import { Lock } from "@gravity-ui/icons";
import { useEffect, useState } from "react";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { providerLabel } from "@/lib/credential-hosts";
import { useCredentialsStore } from "@/stores/credentials";

export function SetCredentialPinDialog() {
  const settingPin = useCredentialsStore((s) => s.settingPin);
  const setSettingPin = useCredentialsStore((s) => s.setSettingPin);
  const setPin = useCredentialsStore((s) => s.setPin);

  const [pin, setPinValue] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const close = () => setSettingPin(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset is keyed off the target credential changing, not read inside.
  useEffect(() => {
    setPinValue("");
    setConfirm("");
    setError(null);
    setSubmitting(false);
  }, [settingPin]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!settingPin || submitting) return;
    if (pin.length < 4 || pin.length > 64) {
      setError("PIN must be 4–64 characters");
      return;
    }
    if (pin !== confirm) {
      setError("PINs do not match");
      return;
    }
    setSubmitting(true);
    setError(null);
    const ok = await setPin(settingPin.id, pin);
    setSubmitting(false);
    if (ok) close();
  };

  const existing = settingPin?.has_pin ?? false;

  return (
    <Dialog.Root open={settingPin !== null} onOpenChange={(o) => !o && close()}>
      <Dialog.Content
        title={existing ? "Change Reveal PIN" : "Set Reveal PIN"}
        description={
          settingPin
            ? `${providerLabel(settingPin.host)} · @${settingPin.username}`
            : "Set a PIN to view this token later."
        }
      >
        <form onSubmit={onSubmit} className="flex flex-col gap-4">
          <div className="flex items-start gap-2 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) px-3 py-2">
            <Lock className="size-3.5 text-(--color-muted) shrink-0 mt-0.5" aria-hidden="true" />
            <p className="text-[11px] leading-relaxed text-(--color-secondary)">
              Once set, you can reveal and copy this credential's token by entering the PIN. Keep it
              somewhere safe — it can't be recovered, only replaced.
            </p>
          </div>

          <Field label="New PIN" htmlFor="set-pin" error={error ?? undefined} required>
            <Input
              id="set-pin"
              type="password"
              inputMode="numeric"
              placeholder="4–64 characters"
              autoComplete="off"
              autoFocus
              className="font-mono text-xs"
              error={!!error}
              value={pin}
              onChange={(e) => setPinValue(e.target.value)}
            />
          </Field>

          <Field label="Confirm PIN" htmlFor="set-pin-confirm" required>
            <Input
              id="set-pin-confirm"
              type="password"
              inputMode="numeric"
              placeholder="Re-enter PIN"
              autoComplete="off"
              className="font-mono text-xs"
              error={!!error}
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
            />
          </Field>

          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={close}>
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={submitting || pin.length === 0 || confirm.length === 0}
            >
              {submitting ? "Saving…" : existing ? "Change PIN" : "Set PIN"}
            </Button>
          </Dialog.Footer>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
