import { CircleCheck, Key, Pencil, TrashBin } from "@gravity-ui/icons";
import { Avatar } from "@/components/avatar";
import { Badge } from "@/components/badge";
// import { Avatar, Badge } from "@/components/ui";
import type { Profile } from "@/ipc/types.gen";
import { cn } from "@/lib/cn";

interface Props {
  profile: Profile;
  isActive: boolean;
  onEdit: (profile: Profile) => void;
  onDelete: (profile: Profile) => void;
  onApply: (profile: Profile) => void;
}

export function ProfileCard({ profile, isActive, onEdit, onDelete, onApply }: Props) {
  return (
    <div
      className={cn(
        "group relative flex flex-col gap-3 p-4 rounded-(--radius-xl) border transition-all",
        "bg-(--color-surface) cursor-default",
        isActive
          ? "border-(--color-brand-500)/50 bg-(--color-brand-500)/8 shadow-[0_0_0_1px_oklch(62%_0.19_260_/_0.15)]"
          : "border-(--color-border) hover:border-(--color-border-strong) hover:bg-(--color-surface-2)"
      )}
    >
      <div
        className="absolute left-0 top-4 bottom-4 w-0.5 rounded-full"
        style={{ backgroundColor: profile.color ?? "#6366f1" }}
        aria-hidden="true"
      />

      <div className="flex items-start justify-between pl-2.5">
        <div className="flex items-center gap-3">
          <Avatar name={profile.identity.name} color={profile.color} size="lg" />
          <div className="flex flex-col gap-0.5">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-(--color-fg)">{profile.label}</span>
              {isActive && (
                <Badge variant="success" className="text-[10px]">
                  Active
                </Badge>
              )}
            </div>
            <span className="text-sm text-(--color-secondary)">{profile.identity.name}</span>
          </div>
        </div>

        <div className="flex items-center gap-1 transition-opacity opacity-0 group-hover:opacity-100">
          {!isActive && (
            <button
              type="button"
              onClick={() => onApply(profile)}
              className="p-1.5 rounded-(--radius-sm) text-(--color-muted) hover:text-(--color-success) hover:bg-(--color-success)/10 transition-colors"
              aria-label={`Apply ${profile.label}`}
              title="Apply this profile globally"
            >
              <CircleCheck className="size-3.5" />
            </button>
          )}
          <button
            type="button"
            onClick={() => onEdit(profile)}
            className="p-1.5 rounded-(--radius-sm) text-(--color-muted) hover:text-(--color-fg) hover:bg-(--color-surface-2) transition-colors"
            aria-label={`Edit ${profile.label}`}
          >
            <Pencil className="size-3.5" />
          </button>
          <button
            type="button"
            onClick={() => onDelete(profile)}
            disabled={isActive}
            className="p-1.5 rounded-(--radius-sm) text-(--color-muted) hover:text-(--color-danger) hover:bg-(--color-danger)/10 transition-colors disabled:opacity-30 disabled:pointer-events-none"
            aria-label={`Delete ${profile.label}`}
          >
            <TrashBin className="size-3.5" />
          </button>
        </div>
      </div>

      <div className="pl-2.5 flex flex-col gap-1">
        <span className="text-xs text-(--color-muted) font-mono truncate">
          {profile.identity.email}
        </span>
        {profile.identity.signing_key && (
          <div className="flex items-center gap-1.5">
            <Key className="size-3 text-(--color-muted)" aria-hidden="true" />
            <span className="text-xs text-(--color-muted) font-mono">
              {profile.identity.signing_key}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
