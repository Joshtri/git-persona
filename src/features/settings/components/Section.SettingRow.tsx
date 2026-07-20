export function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-3 border-b border-(--color-border) last:border-b-0">
      <div className="flex flex-col gap-0.5">
        <span className="text-sm text-(--color-fg)">{label}</span>
        {description && <span className="text-xs text-(--color-muted)">{description}</span>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}
