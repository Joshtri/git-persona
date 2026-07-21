import type { Rule } from "@/ipc/types.gen";

export function makeRule(overrides: Partial<Rule> = {}): Rule {
  return {
    id: "rule-1",
    name: "Company",
    enabled: true,
    priority: 0,
    target_profile_id: "profile-1",
    condition: { subject: "RepoPath", operator: "Contains", value: "/company/" },
    created_at: "2026-07-21T10:00:00Z",
    updated_at: "2026-07-21T10:00:00Z",
    ...overrides,
  };
}
