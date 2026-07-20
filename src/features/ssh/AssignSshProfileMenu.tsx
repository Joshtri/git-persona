import { Popover } from "@base-ui/react/popover";
import { Check, ChevronDown } from "@gravity-ui/icons";
import { useState } from "react";
import type { SshKey } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";
import { useProfilesStore } from "@/stores/profiles";
import { useSshStore } from "@/stores/ssh";

interface Props {
  sshKey: SshKey;
}

export function AssignSshProfileMenu({ sshKey }: Props) {
  const [open, setOpen] = useState(false);
  const profiles = useProfilesStore((s) => s.items);
  const assignProfile = useSshStore((s) => s.assignProfile);

  const assigned = profiles.find((p) => p.id === sshKey.assigned_profile_id) ?? null;

  const choose = (profileId: string | null) => {
    setOpen(false);
    if (profileId !== sshKey.assigned_profile_id) {
      assignProfile(sshKey.id, profileId);
    }
  };

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger
        className={cn(
          "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-medium transition-colors",
          "cursor-pointer",
          assigned
            ? "border-transparent"
            : "border-(--color-border) text-(--color-muted) hover:text-(--color-fg)"
        )}
        style={
          assigned
            ? ({
                color: assigned.color ?? undefined,
                backgroundColor: `${assigned.color ?? "#6366f1"}18`,
                borderColor: `${assigned.color ?? "#6366f1"}30`,
              } as React.CSSProperties)
            : undefined
        }
        aria-label="Assign profile"
      >
        {assigned ? assigned.label : "Assign profile"}
        <ChevronDown className="size-2.5" aria-hidden="true" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="end">
          <Popover.Popup className="z-50 min-w-44 rounded-(--radius-md) border border-(--color-border) p-1 shadow-(--shadow-md) outline-none bg-(--color-surface)">
            <button
              type="button"
              onClick={() => choose(null)}
              className="flex w-full items-center justify-between gap-2 rounded-(--radius-sm) px-2.5 py-1.5 text-left text-xs text-(--color-secondary) hover:bg-(--color-surface-2)"
            >
              Unassigned
              {sshKey.assigned_profile_id == null && (
                <Check className="size-3.5" aria-hidden="true" />
              )}
            </button>
            {profiles.map((profile) => (
              <button
                key={profile.id}
                type="button"
                onClick={() => choose(profile.id)}
                className="flex w-full items-center justify-between gap-2 rounded-(--radius-sm) px-2.5 py-1.5 text-left text-xs text-(--color-fg) hover:bg-(--color-surface-2)"
              >
                <span className="flex items-center gap-2 min-w-0">
                  <span
                    className="size-2 shrink-0 rounded-full"
                    style={{ backgroundColor: profile.color ?? "#6366f1" }}
                    aria-hidden="true"
                  />
                  <span className="truncate">{profile.label}</span>
                </span>
                {sshKey.assigned_profile_id === profile.id && (
                  <Check className="size-3.5 shrink-0" aria-hidden="true" />
                )}
              </button>
            ))}
            {profiles.length === 0 && (
              <p className="px-2.5 py-1.5 text-xs text-(--color-muted)">No profiles yet</p>
            )}
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
