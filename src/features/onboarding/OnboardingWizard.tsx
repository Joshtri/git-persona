import { Dialog as BaseDialog } from "@base-ui/react";
import { CircleCheckFill, Key, Lock, Person } from "@gravity-ui/icons";
import { AnimatePresence, domAnimation, LazyMotion, m } from "motion/react";
import { useState } from "react";
import { BrandLogo } from "@/components/brand-logo";
import { Button } from "@/components/button";
import { Field } from "@/components/field";
import { Input } from "@/components/input";
import { Switch } from "@/components/switch";
import type { DetectedSshKey } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { useOnboardingStore } from "@/stores/onboarding";

const STEP_NUMS = [1, 2, 3, 4] as const;

const stepVariants = {
  enter: (dir: number) => ({ x: dir > 0 ? 48 : -48, opacity: 0 }),
  center: { x: 0, opacity: 1 },
  exit: (dir: number) => ({ x: dir > 0 ? -48 : 48, opacity: 0 }),
};

function ProgressDots({ current }: { current: number }) {
  return (
    <div className="flex items-center justify-center gap-2 mb-7">
      {STEP_NUMS.map((n) => (
        <div
          key={n}
          className={cn(
            "h-1.5 rounded-full transition-all duration-300",
            n === current
              ? "w-6 bg-brand-500"
              : n < current
                ? "w-3 bg-brand-500/40"
                : "w-1.5 bg-(--color-border)"
          )}
        />
      ))}
    </div>
  );
}

function StepWelcome({ onNext, onSkip }: { onNext: () => void; onSkip: () => void }) {
  const scan = useOnboardingStore((s) => s.scan);
  const hasData =
    scan && (scan.identity || scan.ssh_keys.length > 0 || scan.credentials.length > 0);

  return (
    <div className="flex flex-col items-center text-center gap-6">
      <div className="w-full rounded-(--radius-xl) bg-gradient-to-br from-brand-500/20 via-brand-500/10 to-transparent border border-brand-500/20 py-10 flex flex-col items-center gap-3">
        <BrandLogo size="xl" />
        <div>
          <h2 className="text-xl font-bold text-(--color-fg)">GitPersona</h2>
          <p className="text-xs text-(--color-muted) mt-0.5 tracking-wide uppercase">
            Developer Identity Manager
          </p>
        </div>
        <div className="flex gap-1.5 mt-1">
          {["bg-brand-500", "bg-brand-400/60", "bg-brand-300/40"].map((c) => (
            <div key={c} className={cn("size-1.5 rounded-full", c)} />
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-2 px-2">
        <h3 className="text-lg font-semibold text-(--color-fg)">Welcome!</h3>
        <p className="text-sm text-(--color-secondary) leading-relaxed">
          {hasData
            ? "We detected your existing Git setup. Let's import it in a few quick steps."
            : "Let's get you set up. You can configure everything in just a few steps."}
        </p>
      </div>

      <div className="flex w-full items-center justify-between pt-1">
        <Button variant="ghost" onClick={onSkip}>
          Skip setup
        </Button>
        <Button variant="primary" size="lg" onClick={onNext}>
          Get Started →
        </Button>
      </div>
    </div>
  );
}

function StepIdentity({ onNext, onBack }: { onNext: () => void; onBack: () => void }) {
  const scan = useOnboardingStore((s) => s.scan);
  const createProfile = useOnboardingStore((s) => s.createProfile);
  const label = useOnboardingStore((s) => s.label);
  const name = useOnboardingStore((s) => s.name);
  const email = useOnboardingStore((s) => s.email);
  const signingKey = useOnboardingStore((s) => s.signingKey);
  const setCreateProfile = useOnboardingStore((s) => s.setCreateProfile);
  const setField = useOnboardingStore((s) => s.setField);

  return (
    <div className="flex flex-col gap-5">
      <div>
        <h3 className="text-base font-semibold text-(--color-fg)">Git Identity</h3>
        <p className="text-sm text-(--color-secondary) mt-0.5">
          {scan?.identity
            ? "We found your global git config."
            : "No git identity found on this machine."}
        </p>
      </div>

      {scan?.identity ? (
        <>
          <div className="flex items-center gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-2) px-4 py-3">
            <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-brand-500/15">
              <Person className="size-4 text-brand-400" />
            </div>
            <div className="min-w-0">
              <p className="text-sm font-medium text-(--color-fg) truncate">
                {scan.identity.name || "—"}
              </p>
              <p className="text-xs text-(--color-muted) truncate">{scan.identity.email || "—"}</p>
            </div>
          </div>

          <div className="flex items-center justify-between rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-2) px-3 py-2.5">
            <span className="text-sm text-(--color-fg)">Create a profile from this</span>
            <Switch checked={createProfile} onCheckedChange={setCreateProfile} />
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
        </>
      ) : (
        <div className="flex flex-col items-center gap-2 py-8 text-center rounded-(--radius-lg) border border-dashed border-(--color-border)">
          <Person className="size-8 text-(--color-muted)" />
          <p className="text-sm text-(--color-muted)">
            No git identity found. You can set this up manually later.
          </p>
        </div>
      )}

      <div className="flex items-center justify-between pt-2">
        <Button variant="ghost" onClick={onBack}>
          ← Back
        </Button>
        <Button variant="primary" onClick={onNext}>
          Next →
        </Button>
      </div>
    </div>
  );
}

function fileName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function SshCard({ item }: { item: DetectedSshKey }) {
  const selected = useOnboardingStore((s) => s.selectedSsh.includes(item.private_key_path));
  const toggleSsh = useOnboardingStore((s) => s.toggleSsh);

  return (
    <button
      type="button"
      onClick={() => toggleSsh(item.private_key_path)}
      className={cn(
        "flex items-center gap-3 rounded-(--radius-lg) border px-3 py-2.5 text-left w-full transition-colors cursor-pointer",
        selected
          ? "border-brand-500/50 bg-brand-500/10"
          : "border-(--color-border) bg-(--color-surface-2) hover:bg-(--color-surface-3)"
      )}
    >
      <div
        className={cn(
          "flex size-8 shrink-0 items-center justify-center rounded-full",
          selected ? "bg-brand-500/20" : "bg-(--color-surface-3)"
        )}
      >
        <Key className={cn("size-4", selected ? "text-brand-400" : "text-(--color-muted)")} />
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-(--color-fg)">
          {fileName(item.private_key_path)}
        </p>
        <p className="truncate text-xs text-(--color-muted)">
          {item.algorithm} · {item.fingerprint}
        </p>
      </div>
      <div
        className={cn(
          "size-4 rounded-full border flex items-center justify-center shrink-0 transition-colors",
          selected ? "border-brand-500 bg-brand-500" : "border-(--color-border)"
        )}
      >
        {selected && <div className="size-1.5 rounded-full bg-white" />}
      </div>
    </button>
  );
}

function StepSshCredentials({ onNext, onBack }: { onNext: () => void; onBack: () => void }) {
  const scan = useOnboardingStore((s) => s.scan);
  const sshKeys = scan?.ssh_keys ?? [];
  const credentials = scan?.credentials ?? [];
  const nothingHere = sshKeys.length === 0 && credentials.length === 0;

  return (
    <div className="flex flex-col gap-5">
      <div>
        <h3 className="text-base font-semibold text-(--color-fg)">SSH Keys & Credentials</h3>
        <p className="text-sm text-(--color-secondary) mt-0.5">
          {nothingHere ? "Nothing found on this machine." : "Select what you'd like to import."}
        </p>
      </div>

      {nothingHere ? (
        <div className="flex flex-col items-center gap-2 py-8 text-center rounded-(--radius-lg) border border-dashed border-(--color-border)">
          <Key className="size-8 text-(--color-muted)" />
          <p className="text-sm text-(--color-muted)">No SSH keys or saved credentials found.</p>
        </div>
      ) : (
        <div className="flex flex-col gap-4 max-h-60 overflow-y-auto pr-1">
          {sshKeys.length > 0 && (
            <section className="flex flex-col gap-2">
              <h4 className="text-xs font-semibold uppercase tracking-wide text-(--color-muted)">
                SSH Keys
              </h4>
              <div className="flex flex-col gap-2">
                {sshKeys.map((k) => (
                  <SshCard key={k.private_key_path} item={k} />
                ))}
              </div>
            </section>
          )}
          {credentials.length > 0 && (
            <section className="flex flex-col gap-2">
              <h4 className="text-xs font-semibold uppercase tracking-wide text-(--color-muted)">
                Saved Credentials
              </h4>
              <ul className="flex flex-col gap-2">
                {credentials.map((c) => (
                  <li
                    key={c.host}
                    className="flex items-center gap-3 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-2) px-3 py-2.5"
                  >
                    <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-(--color-surface-3)">
                      <Lock className="size-4 text-(--color-muted)" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="text-sm font-medium text-(--color-fg)">{c.host}</p>
                      <p className="text-xs text-(--color-muted)">@{c.username}</p>
                    </div>
                  </li>
                ))}
              </ul>
              <p className="text-xs text-(--color-muted) px-1">
                Usernames only — add tokens later from the Credentials page.
              </p>
            </section>
          )}
        </div>
      )}

      <div className="flex items-center justify-between pt-2">
        <Button variant="ghost" onClick={onBack}>
          ← Back
        </Button>
        <Button variant="primary" onClick={onNext}>
          Next →
        </Button>
      </div>
    </div>
  );
}

function StepDone({ onFinish, applying }: { onFinish: () => void; applying: boolean }) {
  const scan = useOnboardingStore((s) => s.scan);
  const createProfile = useOnboardingStore((s) => s.createProfile);
  const selectedSsh = useOnboardingStore((s) => s.selectedSsh);
  const credentials = scan?.credentials ?? [];

  const items = [
    createProfile && scan?.identity && "1 profile",
    selectedSsh.length > 0 && `${selectedSsh.length} SSH key${selectedSsh.length > 1 ? "s" : ""}`,
    credentials.length > 0 &&
      `${credentials.length} credential${credentials.length > 1 ? "s" : ""}`,
  ].filter(Boolean) as string[];

  return (
    <div className="flex flex-col items-center text-center gap-6 py-2">
      <div className="flex size-16 items-center justify-center rounded-full bg-success/15 ring-4 ring-success/10">
        <CircleCheckFill className="size-8 text-(--color-success)" />
      </div>

      <div>
        <h3 className="text-lg font-semibold text-(--color-fg)">You're all set!</h3>
        <p className="text-sm text-(--color-secondary) mt-1 max-w-xs">
          {items.length > 0
            ? "Here's what will be imported into GitPersona:"
            : "Nothing to import. You can configure everything manually whenever you're ready."}
        </p>
      </div>

      {items.length > 0 && (
        <ul className="flex flex-col gap-2 text-left w-full">
          {items.map((item) => (
            <li
              key={item}
              className="flex items-center gap-2.5 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) px-3 py-2.5"
            >
              <CircleCheckFill className="size-4 text-(--color-success) shrink-0" />
              <span className="text-sm text-(--color-fg)">{item}</span>
            </li>
          ))}
        </ul>
      )}

      <Button variant="primary" size="lg" className="w-full" onClick={onFinish} disabled={applying}>
        {applying
          ? "Importing…"
          : items.length > 0
            ? "Import & Open GitPersona →"
            : "Open GitPersona →"}
      </Button>
    </div>
  );
}

export function OnboardingWizard() {
  const open = useOnboardingStore((s) => s.open);
  const applying = useOnboardingStore((s) => s.applying);
  const apply = useOnboardingStore((s) => s.apply);
  const skip = useOnboardingStore((s) => s.skip);
  const scan = useOnboardingStore((s) => s.scan);
  const createProfile = useOnboardingStore((s) => s.createProfile);
  const selectedSsh = useOnboardingStore((s) => s.selectedSsh);

  const [step, setStep] = useState(1);
  const [direction, setDirection] = useState(1);

  const goTo = (next: number) => {
    setDirection(next > step ? 1 : -1);
    setStep(next);
  };

  const handleFinish = () => {
    const credentials = scan?.credentials ?? [];
    const hasAnything =
      (createProfile && scan?.identity) || selectedSsh.length > 0 || credentials.length > 0;
    if (hasAnything) {
      apply();
    } else {
      skip();
    }
  };

  const stepContent = [
    <StepWelcome key="welcome" onNext={() => goTo(2)} onSkip={skip} />,
    <StepIdentity key="identity" onNext={() => goTo(3)} onBack={() => goTo(1)} />,
    <StepSshCredentials key="ssh" onNext={() => goTo(4)} onBack={() => goTo(2)} />,
    <StepDone key="done" onFinish={handleFinish} applying={applying} />,
  ];

  return (
    <BaseDialog.Root
      open={open}
      onOpenChange={(next) => {
        if (!next) skip();
      }}
    >
      {open && (
        <BaseDialog.Portal>
          <BaseDialog.Backdrop className="fixed inset-0 bg-black/75 z-40 animate-[fade-in_150ms_ease]" />
          <BaseDialog.Popup
            className="fixed z-50 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg rounded-(--radius-2xl) bg-(--color-surface) border border-(--color-border) p-7 overflow-hidden animate-[slide-up_200ms_ease]"
            style={{ boxShadow: "var(--shadow-md)" }}
          >
            <BaseDialog.Title className="sr-only">GitPersona Setup</BaseDialog.Title>

            <ProgressDots current={step} />

            <LazyMotion features={domAnimation} strict>
              <AnimatePresence mode="wait" custom={direction}>
                <m.div
                  key={step}
                  custom={direction}
                  variants={stepVariants}
                  initial="enter"
                  animate="center"
                  exit="exit"
                  transition={{ duration: 0.2, ease: "easeInOut" }}
                >
                  {stepContent[step - 1]}
                </m.div>
              </AnimatePresence>
            </LazyMotion>
          </BaseDialog.Popup>
        </BaseDialog.Portal>
      )}
    </BaseDialog.Root>
  );
}
