import { MagnifierMinus } from "@gravity-ui/icons";

interface Props {
  query: string;
}

export function NoResultsState({ query }: Props) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 py-12 text-center">
      <MagnifierMinus className="size-8 text-(--color-muted)" aria-hidden="true" />
      <p className="text-sm font-medium text-(--color-fg)">No results for &ldquo;{query}&rdquo;</p>
      <p className="text-xs text-(--color-muted)">Try a different search term</p>
    </div>
  );
}
