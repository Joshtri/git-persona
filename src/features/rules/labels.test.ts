import { describe, expect, it } from "vitest";
import { makeRule } from "@/test/fixtures/rules";
import { conditionSummary, operatorOptionsFor } from "./labels";

describe("rule labels", () => {
  it("offers every operator for path/name/remote subjects", () => {
    expect(operatorOptionsFor("RepoPath")).toHaveLength(4);
    expect(operatorOptionsFor("RepoName")).toHaveLength(4);
    expect(operatorOptionsFor("RemoteUrl")).toHaveLength(4);
  });

  it("restricts host and owner to equals only", () => {
    const host = operatorOptionsFor("RemoteHost");
    const owner = operatorOptionsFor("Owner");
    expect(host).toHaveLength(1);
    expect(host[0]?.value).toBe("Equals");
    expect(owner).toHaveLength(1);
    expect(owner[0]?.value).toBe("Equals");
  });

  it("renders a human-readable condition summary", () => {
    const rule = makeRule({
      condition: { subject: "RepoName", operator: "StartsWith", value: "api-" },
    });
    expect(conditionSummary(rule)).toBe('Repository name starts with "api-"');
  });
});
