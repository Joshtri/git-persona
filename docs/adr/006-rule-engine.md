# ADR 006 — Rule Engine: a declarative decision layer above Smart Switching

**Status**: Accepted
**Date**: Sprint 7

## Context

After Sprint 6, Smart Identity Switching applies a repository's identity
automatically — but only when the repository already carries a **manual profile
assignment** (`Repo.active_profile_id`). A freshly cloned repository has no
assignment, so nothing happens until the developer assigns a profile by hand.

The recurring real-world pattern is structural: "everything under `~/work/` is my
work identity", "anything cloned from `github.com/my-company` is the company
identity", "repos owned by `open-source` use my OSS identity." Encoding that once,
declaratively, removes the per-repository busywork.

ADR 005 anticipated this ("a richer rule engine slots in behind
`IdentityResolver`"). Sprint 7 delivers it, with one refinement: rather than
replacing the resolver, the engine is a **separate decision port the orchestrator
consults first**, keeping the two concerns — *which* profile vs. *how* to apply it
— cleanly separated.

## Decision

### A new bounded context — decision only, never execution

A new `rule` context is added across every layer
(`domain/ → services/ → infra/ + commands/`), exactly mirroring the credential and
repo contexts. Its single job is to **choose a profile id**. It never edits Git
config, SSH, credentials, or the active marker — execution stays entirely in the
Sprint-5 `ProfileService::apply` saga.

```
Watcher → Repository → Rule Engine → Resolved Profile → ProfileService::apply
```

`RuleService` implements a narrow `RuleResolver` port
(`resolve(&EvaluationContext) -> Option<RuleMatch>`), just as `CredentialService`
implements `ProfileCredentialSync`. `IdentitySwitchService` gains one dependency on
that port and calls it inside a new `resolve_profile` step; nothing else in the
orchestrator changes.

### Resolution order — rules first, manual assignment as fallback

```
resolve_profile(repo):
  ctx  = facts(path, name, remote_url → host, owner)
  hit  = rules.resolve(ctx)          # enabled rules, priority asc, first match wins
  if hit: record "rule.matched"; return hit.profile_id
  else:   return repo.active_profile_id   # unchanged Sprint-6 behaviour
```

- **First match wins**, ordered by `priority` ascending (lower = higher priority).
- **No rule matches → fall back** to the manual assignment. A repository with an
  assignment and no matching rule switches exactly as before — full backward
  compatibility, verified by test.
- Neither a rule nor an assignment → `NoAssignment`, as today.

Rules therefore *override* a manual assignment when they match; the manual
assignment is the safety net, not the other way around. This is what makes the
headline scenario work: clone into a matching folder, run a Git action, and the
correct identity is applied with **no** manual assignment.

### Declarative conditions only — deterministic, no code execution

A condition is `subject · operator · value`:

| Subject                     | Operators                                    |
| --------------------------- | -------------------------------------------- |
| Repository path             | contains · starts_with · ends_with · equals  |
| Repository name             | contains · starts_with · ends_with · equals  |
| Remote URL                  | contains · starts_with · ends_with · equals  |
| Remote host                 | equals                                       |
| Owner / organization        | equals                                       |

Host and owner are exact identifiers derived by parsing the remote URL
(`parse_remote`, handling both `git@host:owner/repo.git` and
`https://host/owner/repo.git`), so substring matching on them would be
surprising — they are constrained to `equals`, enforced in the service and mirrored
in the frontend zod schema. Matching is **case-insensitive** (Windows paths and
hosts are). There is deliberately **no regex, glob, DSL, scripting, or plugin
system** (see Security).

### Preview is strictly read-only

`rule_preview` evaluates a hypothetical `(path, name, remote_url)` and reports the
matched rule, resolved profile, and a human reason. It shares the pure `evaluate`
path with the resolver but — unlike the resolver — never updates the dashboard's
`last_match` and never triggers a switch. This separation is asserted by test.

### Import / export for teams

`rule_export` writes the rule set as pretty JSON to a path the user picks via the
dialog plugin; `rule_import` reads one back (replace-all or merge). File I/O stays
in Rust; the frontend only chooses the path (a new `dialog:allow-save` capability).
Imported rules receive fresh ids and re-normalized priorities and are validated to
reference profiles that exist on the importing machine.

## Consequences

- **Separation of concerns preserved.** The Rule Engine decides; Smart Switching
  executes. `ProfileService::apply` is untouched and remains the only path that
  mutates identity state.
- **Backward compatible.** With no rules defined, behaviour is identical to
  Sprint 6. The change to `IdentitySwitchService` is additive (one port, one step).
- **Security.** Rules are pure data. Evaluation is a linear scan of string
  comparisons — it cannot execute shell commands, evaluate scripts, load plugins,
  or touch the network. The store holds no secrets. `dialog:allow-save` is the only
  new capability, scoped to a user-initiated save dialog; the actual write happens
  in a Rust command.
- **Performance.** Evaluation is O(n) over 10–100 rules — a trivial in-memory scan
  on each Git activation. No caching, indexing, or precompilation is warranted.
- **Determinism.** Priority ordering + first-match + case-folded exact string ops
  make resolution fully predictable and reproducible across machines.
- **`last_match` is in-memory** (resets on restart), mirroring
  `SmartSwitchStatus.last_switch`; the dashboard's active/disabled counts come from
  the store and are always accurate.
- **Extensibility.** New subjects/operators are additive enum variants; a future
  activation source or richer matcher slots in without touching the orchestrator.
