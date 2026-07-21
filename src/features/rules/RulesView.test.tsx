import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithStores, screen } from "@/test/utils";

const hoisted = vi.hoisted(() => {
  const makeHook = <T extends Record<string, unknown>>(state: T) => {
    const hook = (selector?: (s: T) => unknown) => (selector ? selector(state) : state);
    (hook as unknown as { getState: () => T }).getState = () => state;
    return hook;
  };
  const rules = {
    items: [] as unknown[],
    loading: false,
    error: null,
    createOpen: false,
    previewOpen: false,
    editing: null,
    preview: null,
    summary: null,
    fetch: vi.fn(),
    fetchSummary: vi.fn(),
    setCreateOpen: vi.fn(),
    setPreviewOpen: vi.fn(),
    setEditing: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
    duplicate: vi.fn(),
    setEnabled: vi.fn(),
    setAllEnabled: vi.fn(),
    move: vi.fn(),
    runPreview: vi.fn(),
    exportRules: vi.fn(),
    importRules: vi.fn(),
  };
  const profiles = { items: [] as unknown[], fetch: vi.fn() };
  return { makeHook, rules, profiles };
});

vi.mock("@/stores/rules", () => ({ useRulesStore: hoisted.makeHook(hoisted.rules) }));
vi.mock("@/stores/profiles", () => ({ useProfilesStore: hoisted.makeHook(hoisted.profiles) }));

const rulesState = hoisted.rules;

import { RulesView } from "./RulesView";

describe("RulesView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    rulesState.items = [];
  });

  it("renders the empty state and fetches on mount", () => {
    renderWithStores(<RulesView />);
    expect(screen.getByText("No rules yet")).toBeInTheDocument();
    expect(rulesState.fetch).toHaveBeenCalled();
  });

  it("disables export when there are no rules", () => {
    renderWithStores(<RulesView />);
    const exportButton = screen.getByRole("button", { name: /export/i });
    expect(exportButton).toBeDisabled();
  });
});
