# ADR 005 — Smart Identity Switching: watch `.git`, orchestrate, never re-implement

**Status**: Accepted
**Date**: Sprint 6

## Context

Sprints 0–5 made identity management complete but manual: a developer opens a
repository and applies its profile by hand. Sprint 6 automates that last step —
open a repo, and the assigned identity is applied for you — without redesigning
any existing layer or duplicating the apply pipeline.

Two hard questions shaped the design:

1. **What filesystem signal means "this repository is now current"?** There is no
   OS event for "developer opened a repo in a terminal," and we ship no editor
   integration (out of scope).
2. **How do we switch without re-implementing git/SSH/credential logic** that
   already lives in `ProfileService`, `SshService`, and `CredentialService`?

## Decision

### A new orchestration domain — thin, coordinating only

One new domain, **Identity Orchestrator**, sits above the existing services and
owns no I/O of its own. It coordinates four ports and records outcomes:

```
WorkspaceWatcher → IdentitySwitchService → IdentityResolver
                                          → IdentitySwitcher  → ProfileService.apply
                                          → SwitchObserver    → Tauri events
                                          → AuditSink
```

The full layering is preserved: `commands/ → services/ → domain/ ← infra/`. The
watcher and observer are infra adapters; the resolver and switcher are thin
service-layer adapters over the repo store and `ProfileService`.

### Detection = Git state changes, not working-tree activity

The `GitDirWatcher` watches each tracked repo's resolved `.git` directory
**non-recursively** and reacts only to `HEAD`, `ORIG_HEAD`, `MERGE_HEAD`, and
`index`. These move on checkout / commit / merge / reset / rebase / pull — a
strong "I am working here now" signal — while `.git/objects`, editor auto-saves,
`node_modules`, and build output are never seen. This keeps CPU flat across 1000+
repositories (one debounced, filtered, non-recursive watch each) and avoids the
false positives a "watch the whole tree" approach would cause (e.g. formatters or
auto-save churning an inactive repo). Detection lives behind the
`WorkspaceWatcher` port so future sources (editor/terminal integrations) can be
added in a later sprint without touching the orchestrator.

### Switching reuses the existing atomic apply — no duplication

The `IdentitySwitcher` port delegates to `ProfileService::apply`, the Sprint-5
saga that switches git config → HTTPS credentials → active marker atomically with
rollback. SSH follows automatically through the managed `~/.ssh/config` block, so
no per-switch SSH step is needed. The orchestrator therefore never edits Git or
the vault directly — it decides *whether* to switch and lets the proven pipeline
do the *how*. `auto_ssh` / `auto_credential` are surfaced as settings (defaulting
on) and reserved for a future per-stage gate; today the apply is all-or-nothing by
design.

### The decision flow

```
activity(git_root) → resolve assignment
  ├─ untracked            → ignore
  ├─ no assignment        → record identity.no_assignment
  ├─ already active       → record identity.same_profile (no-op)
  ├─ confirm_before_switch→ emit PendingConfirmation (await UI)
  └─ otherwise            → apply, record identity.auto_switch, notify
```

Errors on the watcher thread are swallowed (never panic): a removed repo, missing
key, deleted profile, or busy Git leaves the app running and the next event
retries.

### Watcher lifecycle is settings-driven and reconciled

`reconcile()` is the single source of truth: it starts the watcher over every
tracked repo when `enabled`, stops it otherwise. It is called on launch (gated by
`start_on_launch`), on `settings_set`, on explicit enable/restart, and after a
scan discovers new repositories. `pause`/`resume` flip a flag without releasing
OS watches. New `SmartSwitchingSettings` carry `#[serde(default)]` so settings
files written before Sprint 6 still load.

## Consequences

- Zero changes to `ProfileService`/`SshService`/`CredentialService` behaviour;
  the orchestrator is purely additive.
- A new dependency, `notify-debouncer-mini`, provides native, event-driven
  watching (`ReadDirectoryChangesW` on Windows) — no polling, no busy loops.
- A passive `cd` into a repo with no Git action will not switch; this is an
  accepted trade-off of the HEAD/index model, chosen over the noise of
  activity-watching. A future detector port can cover that case.
- Notifications are in-app toasts (respecting `show_notification`), avoiding a new
  OS-notification capability and its security review.
- Extensibility: new activation sources implement `WorkspaceWatcher`; a richer
  rule engine (wildcards/regex — explicitly out of scope now) slots in behind
  `IdentityResolver`. Neither touches `IdentitySwitchService`.
