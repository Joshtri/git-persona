export function SectionHeader({ title, description }: { title: string; description?: string }) {
  return (
    <div className="flex flex-col gap-0.5 mb-4">
      <h2 className="text-sm font-semibold text-(--color-fg)">{title}</h2>
      {description && <p className="text-xs text-(--color-muted)">{description}</p>}
    </div>
  );
}
