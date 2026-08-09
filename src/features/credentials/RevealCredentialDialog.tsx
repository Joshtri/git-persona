import { Copy, Eye } from "@gravity-ui/icons";
import { useEffect, useState } from "react";
import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { providerLabel } from "@/lib/credential-hosts";
import { useCredentialsStore } from "@/stores/credentials";
import { useFeedbackStore } from "@/stores/feedback";

// How long a revealed token stays on screen before it is cleared automatically.
const AUTO_HIDE_MS = 30_000;

export function RevealCredentialDialog() {
  const revealing = useCredentialsStore((s) => s.revealing);
  const setRevealing = useCredentialsStore((s) => s.setRevealing);
  const reveal = useCredentialsStore((s) => s.reveal);
  const toast = useFeedbackStore((s) => s.toast);

  const [pin, setPin] = useState("");
  const [token, setToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const close = () => setRevealing(null);

  // Reset transient state whenever a different credential is opened or closed.
  // biome-ignore lint/correctness/useExhaustiveDependencies: reset is keyed off the target credential changing, not read inside.
  useEffect(() => {
    setPin("");
    setToken(null);
    setError(null);
    setSubmitting(false);
  }, [revealing]);

  // Never leave a revealed token lingering on screen.
  useEffect(() => {
    if (token === null) return;
    const timer = setTimeout(() => setToken(null), AUTO_HIDE_MS);
    return () => clearTimeout(timer);
  }, [token]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!revealing || submitting) return;
    setSubmitting(true);
    setError(null);
    const result = await reveal(revealing.id, pin);
    setSubmitting(false);
    if (result.ok) {
      setToken(result.token);
    } else {
      setError(result.message);
    }
  };

  const copyToken = async () => {
    if (token === null) return;
    try {
      await navigator.clipboard.writeText(token);
      toast("Token copied", "success");
    } catch {
      toast("Could not copy token", "error");
    }
  };

  return (
    <Dialog.Root open={revealing !== null} onOpenChange={(o) => !o && close()}>
      <Dialog.Content
        title="Reveal Token"
        description={
          revealing
            ? `${providerLabel(revealing.host)} · @${revealing.username}`
            : "Enter your PIN to view the stored token."
        }
      >
        {token === null ? (
          <form onSubmit={onSubmit} className="flex flex-col gap-4">
            <Field label="PIN" htmlFor="reveal-pin" error={error ?? undefined} required>
              <Input
                id="reveal-pin"
                type="password"
                inputMode="numeric"
                placeholder="••••"
                autoComplete="off"
                autoFocus
                className="font-mono text-xs"
                error={!!error}
                value={pin}
                onChange={(e) => setPin(e.target.value)}
              />
            </Field>

            <Dialog.Footer>
              <Button type="button" variant="ghost" onClick={close}>
                Cancel
              </Button>
              <Button type="submit" variant="primary" disabled={submitting || pin.length === 0}>
                <Eye className="size-3.5" aria-hidden="true" />
                {submitting ? "Verifying…" : "Reveal"}
              </Button>
            </Dialog.Footer>
          </form>
        ) : (
          <div className="flex flex-col gap-4">
            <Field
              label="Token"
              htmlFor="reveal-token"
              hint="Auto-hides after 30 seconds. Copy it somewhere safe."
            >
              <Input
                id="reveal-token"
                readOnly
                className="font-mono text-xs"
                value={token}
                onFocus={(e) => e.target.select()}
              />
            </Field>

            <Dialog.Footer>
              <Button type="button" variant="secondary" onClick={copyToken}>
                <Copy className="size-3.5" aria-hidden="true" />
                Copy token
              </Button>
              <Button type="button" variant="primary" onClick={close}>
                Done
              </Button>
            </Dialog.Footer>
          </div>
        )}
      </Dialog.Content>
    </Dialog.Root>
  );
}
