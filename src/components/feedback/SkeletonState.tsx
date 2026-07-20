import { Skeleton } from "@/components/skeleton";

interface Props {
  rows?: number;
}

const SKELETON_IDS = ["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8"];

export function SkeletonState({ rows = 4 }: Props) {
  return (
    <div className="flex flex-col gap-3 p-1">
      {SKELETON_IDS.slice(0, rows).map((id) => (
        <div
          key={id}
          className="flex items-center gap-3 p-3 rounded-(--radius-lg) bg-(--color-surface) border border-(--color-border)"
        >
          <Skeleton className="size-10 shrink-0" rounded />
          <div className="flex flex-col gap-2 flex-1">
            <Skeleton className="h-3.5 w-1/3" />
            <Skeleton className="h-3 w-1/2" />
          </div>
          <Skeleton className="h-5 w-16" />
        </div>
      ))}
    </div>
  );
}
