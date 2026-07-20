import { Spinner } from "@/components/spinner";

export function LoadingState() {
  return (
    <div className="flex items-center justify-center py-16">
      <Spinner size="lg" className="text-(--color-brand-500)" />
    </div>
  );
}
