import { type RenderOptions, render } from "@testing-library/react";
import type { ReactElement } from "react";

export function renderWithStores(ui: ReactElement, options?: Omit<RenderOptions, "wrapper">) {
  return render(ui, options);
}

export * from "@testing-library/react";
