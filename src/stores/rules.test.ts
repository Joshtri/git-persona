import { beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import { makeRule } from "@/test/fixtures/rules";

vi.mock("@/ipc", () => ({
  ruleList: vi.fn(),
  ruleCreate: vi.fn(),
  ruleUpdate: vi.fn(),
  ruleDelete: vi.fn(),
  ruleDuplicate: vi.fn(),
  ruleExport: vi.fn(),
  ruleImport: vi.fn(),
  rulePreview: vi.fn(),
  ruleReorder: vi.fn(),
  ruleSetAllEnabled: vi.fn(),
  ruleSetEnabled: vi.fn(),
  ruleSummary: vi.fn(),
}));
vi.mock("@/ipc/dialog", () => ({
  pickRulesExportPath: vi.fn(),
  pickRulesImportFile: vi.fn(),
}));
vi.mock("@/stores/feedback", () => ({
  useFeedbackStore: { getState: () => ({ toast: vi.fn() }) },
}));
vi.mock("@/stores/activity", () => ({
  useActivityStore: { getState: () => ({ fetch: vi.fn() }) },
}));

import * as ipc from "@/ipc";
import { useRulesStore } from "./rules";

const initial = useRulesStore.getState();

describe("useRulesStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRulesStore.setState(initial, true);
  });

  it("fetch populates items", async () => {
    const rule = makeRule();
    (ipc.ruleList as Mock).mockResolvedValue([rule]);
    await useRulesStore.getState().fetch();
    expect(useRulesStore.getState().items).toEqual([rule]);
    expect(useRulesStore.getState().loading).toBe(false);
  });

  it("create calls the command then refetches", async () => {
    (ipc.ruleCreate as Mock).mockResolvedValue(makeRule());
    (ipc.ruleList as Mock).mockResolvedValue([makeRule()]);
    const ok = await useRulesStore.getState().create({
      name: "Company",
      subject: "RepoPath",
      operator: "Contains",
      value: "/company/",
      targetProfileId: "profile-1",
    });
    expect(ok).toBe(true);
    expect(ipc.ruleCreate).toHaveBeenCalledWith(
      "Company",
      "RepoPath",
      "Contains",
      "/company/",
      "profile-1"
    );
    expect(ipc.ruleList).toHaveBeenCalled();
  });

  it("move down swaps a rule with its next neighbour and persists the order", async () => {
    const a = makeRule({ id: "a", priority: 0 });
    const b = makeRule({ id: "b", priority: 1 });
    const c = makeRule({ id: "c", priority: 2 });
    useRulesStore.setState({ items: [a, b, c] });
    (ipc.ruleReorder as Mock).mockResolvedValue([]);
    (ipc.ruleList as Mock).mockResolvedValue([]);

    await useRulesStore.getState().move("a", "down");

    expect(ipc.ruleReorder).toHaveBeenCalledWith(["b", "a", "c"]);
  });

  it("move up at the top is a no-op", async () => {
    const a = makeRule({ id: "a" });
    useRulesStore.setState({ items: [a] });
    await useRulesStore.getState().move("a", "up");
    expect(ipc.ruleReorder).not.toHaveBeenCalled();
  });

  it("runPreview stores the result and passes null for a blank remote", async () => {
    (ipc.rulePreview as Mock).mockResolvedValue({ matched: null });
    await useRulesStore.getState().runPreview({ path: "/x", name: "x", remoteUrl: "  " });
    expect(ipc.rulePreview).toHaveBeenCalledWith({ path: "/x", name: "x", remote_url: null });
    expect(useRulesStore.getState().preview).toEqual({ matched: null });
  });
});
