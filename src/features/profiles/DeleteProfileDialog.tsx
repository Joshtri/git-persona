import { ConfirmationDialog } from "@/components/confirmation-dialog";
import type { Profile } from "@/ipc/types.gen";
import { useProfilesStore } from "@/stores/profiles";
import { useReposStore } from "@/stores/repos";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  profile: Profile | null;
}

export function DeleteProfileDialog({ open, onOpenChange, profile }: Props) {
  const { delete: deleteProfile } = useProfilesStore();
  const repos = useReposStore((s) => s.items);
  const unassignAll = useReposStore((s) => s.unassignAll);

  const assignedCount = profile
    ? repos.filter((r) => r.active_profile_id === profile.id).length
    : 0;

  const handleDelete = async () => {
    if (!profile) return;
    if (assignedCount > 0) {
      await unassignAll(profile.id);
    }
    const success = await deleteProfile(profile.id);
    if (success) onOpenChange(false);
  };

  const description =
    assignedCount > 0
      ? `This profile is assigned to ${assignedCount} ${
          assignedCount === 1 ? "repository" : "repositories"
        }. They will be unassigned first, then the "${profile?.label}" profile will be permanently removed.`
      : `This will permanently remove the "${profile?.label}" profile. This action cannot be undone.`;

  return (
    <ConfirmationDialog
      open={open}
      onOpenChange={onOpenChange}
      tone="danger"
      title="Delete Profile"
      description={description}
      confirmLabel={assignedCount > 0 ? "Unassign & Delete" : "Delete Profile"}
      onConfirm={handleDelete}
    />
  );
}
