import type { SelectOption } from "@/features/inputs/non-form/select";
import type { Rule, RuleOperator, RuleSubject } from "@/ipc/types.gen";

export const SUBJECT_LABELS: Record<RuleSubject, string> = {
  RepoPath: "Repository path",
  RepoName: "Repository name",
  RemoteUrl: "Remote URL",
  RemoteHost: "Remote host",
  Owner: "Owner / Organization",
};

export const OPERATOR_LABELS: Record<RuleOperator, string> = {
  Contains: "contains",
  StartsWith: "starts with",
  EndsWith: "ends with",
  Equals: "equals",
};

/** Subjects that only support the `Equals` operator (see backend `RuleSubject::allows`). */
export const EQUALS_ONLY_SUBJECTS: RuleSubject[] = ["RemoteHost", "Owner"];

export const SUBJECT_OPTIONS: SelectOption[] = (Object.keys(SUBJECT_LABELS) as RuleSubject[]).map(
  (value) => ({ value, label: SUBJECT_LABELS[value] })
);

const ALL_OPERATOR_OPTIONS: SelectOption[] = (Object.keys(OPERATOR_LABELS) as RuleOperator[]).map(
  (value) => ({ value, label: OPERATOR_LABELS[value] })
);

const EQUALS_ONLY_OPTIONS: SelectOption[] = [{ value: "Equals", label: OPERATOR_LABELS.Equals }];

/** Operator options legal for the given subject. */
export function operatorOptionsFor(subject: RuleSubject): SelectOption[] {
  return EQUALS_ONLY_SUBJECTS.includes(subject) ? EQUALS_ONLY_OPTIONS : ALL_OPERATOR_OPTIONS;
}

/** A one-line, human-readable summary of a rule's condition. */
export function conditionSummary(rule: Rule): string {
  const { subject, operator, value } = rule.condition;
  return `${SUBJECT_LABELS[subject]} ${OPERATOR_LABELS[operator]} "${value}"`;
}
