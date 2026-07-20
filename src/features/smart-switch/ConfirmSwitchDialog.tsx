import { Button } from "@/components/button";
import { Dialog } from "@/components/dialog";
import { useSmartSwitchStore } from "@/stores/smartSwitch";

/// Rendered once at the shell level. Surfaces a pending switch when the user has
/// enabled "Confirm before switch" so an automatic switch waits on approval.
export function ConfirmSwitchDialog() {
  const pending = useSmartSwitchStore((s) => s.pending);
  const confirmPending = useSmartSwitchStore((s) => s.confirmPending);
  const dismissPending = useSmartSwitchStore((s) => s.dismissPending);

  return (
    <Dialog.Root
      open={pending !== null}
      onOpenChange={(open) => {
        if (!open) dismissPending();
      }}
    >
      {pending && (
        <Dialog.Content
          title="Switch identity?"
          description={`${pending.repo_name ?? "This repository"} is assigned to a different profile.`}
        >
          <p className="text-sm text-(--color-secondary)">
            Apply{" "}
            <span className="font-medium text-(--color-fg)">
              {pending.profile_label ?? "the assigned profile"}
            </span>{" "}
            for this repository now?
          </p>
          <Dialog.Footer>
            <Button type="button" variant="ghost" onClick={() => dismissPending()}>
              Not now
            </Button>
            <Button type="button" variant="primary" onClick={() => confirmPending()}>
              Switch
            </Button>
          </Dialog.Footer>
        </Dialog.Content>
      )}
    </Dialog.Root>
  );
}
