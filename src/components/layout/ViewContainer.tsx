import type { ReactNode } from "react";

export function ViewContainer({ children }: { children: ReactNode }) {
  return <main className="flex-1 overflow-y-auto p-5">{children}</main>;
}
