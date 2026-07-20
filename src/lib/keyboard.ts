export function isMod(e: KeyboardEvent): boolean {
  return e.metaKey || e.ctrlKey;
}

export function isInputTarget(e: KeyboardEvent): boolean {
  const tag = (e.target as HTMLElement)?.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}
