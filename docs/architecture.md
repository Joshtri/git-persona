# GitPersona — Architecture Overview

## Layers

```
Frontend (React 19 + TypeScript)
  └── src/ipc/          ← sole @tauri-apps/api import boundary
  └── src/stores/       ← Zustand 5 async state owners
  └── src/features/     ← vertical-slice UI modules
  └── src/components/   ← design system + layout

IPC boundary (Tauri invoke / emit)

Backend (Rust)
  commands/             ← thin DTOs → one service call → map error
  services/             ← use-case orchestration via Arc<dyn Trait>
  domain/               ← pure types + port traits (zero I/O)
  infra/                ← port implementations (filesystem, store, gitconfig)
```

## Frontend data flow

```
UI event
  → store action (async)
    → ipc/client.ts invoke
      ← Rust Result<T, AppError>
    → store sets { data, loading, error }
      → component re-renders
```

Stores are the sole async state owners. Components never call `invoke` directly.

## Rust layer rules

- `domain/`: no I/O, no tauri imports. Pure types + traits only.
- `infra/`: sole layer allowed to touch filesystem, gix-config, tauri-plugin-store.
- `services/`: orchestration. No Tauri imports except `AppHandle` for events.
- `commands/`: validate input, call one service method, map `AppError` → Tauri serialisable.

## Error wire format

All IPC errors are serialised as:
```json
{ "code": "NOT_FOUND", "message": "profile abc123 not found" }
```

Frontend branches on `error.code` (typed enum), never on `error.message` text.

## Navigation

No router. Navigation is a Zustand discriminated union:
```typescript
type View = { name: "profiles" } | { name: "repos" } | { name: "rules" } | { name: "settings" } | { name: "onboarding" }
```

## Theme

Tailwind v4 CSS-only config via `@theme {}` in `src/styles/theme.css`.  
Dark mode is `:root` default. Light mode is `[data-theme="light"]`.  
No `tailwind.config.js` — ever.

## Credential Manager (Sprint 5)

GitPersona manages HTTPS Git credentials but is **not a secret vault**. Every
secret lives only in the operating system's secure store (Windows Credential
Manager today; macOS Keychain / Linux Secret Service later). GitPersona stores
**metadata only** and orchestrates the OS store.

### Components

```
domain/credential.rs   Credential (metadata), Protocol, Secret = Zeroizing<String>,
                       VaultSnapshot / VaultSecret / CredentialTxn (rollback types)
domain/ports.rs        CredentialStore (metadata), CredentialVault (OS secrets),
                       ProfileCredentialSync (pipeline port)
services/credential_service.rs   all business logic + ProfileCredentialSync impl
infra/credential_store_tauri.rs  JSON metadata (gitpersona.json, "credentials" key)
infra/credential_vault_windows.rs  Win32 CredRead/Write/Delete (the only `unsafe`)
infra/credential_vault_noop.rs     non-Windows fallback (returns Unsupported)
commands/credentials.rs            thin IPC adapters
```

Frontend: `stores/credentials.ts` owns async state; `features/credentials/*`
renders it; `lib/credential-hosts.ts` holds the supported-host / provider list.

### Credential model

A `Credential` is `{ id, profile_id?, host, protocol, username, created_at,
updated_at, last_used? }`. It **never** carries a token — the `Credential` type
has no secret field, so nothing token-shaped is ever serialized to JSON, logged,
or returned over IPC.

### Two vault targets

The vault is keyed by opaque target strings built by the service:

- **Canonical** `git:https://<host>` — exactly what Git's `wincred` / GCM helper
  reads. Only one credential per host is active here at a time.
- **Backing** `gitpersona:<credential-id>:https://<host>` — where each profile's
  secret is parked. This lets several profiles each hold, say, a `github.com`
  credential while only one is promoted to the canonical target.

Because backing secrets live in the OS store too, GitPersona still holds **zero**
plaintext secrets in its own files.

### Secret lifecycle

```
Create/Update: token enters once as a command arg → Zeroizing<String>
             → vault.store(backing target) → buffer zeroized. Never persisted by us.
Switch/Apply:  vault.promote(backing → canonical) copies the secret inside the
             vault (verified by read-back). Rollback snapshots capture prior
             secrets in Zeroizing buffers, zeroized on drop.
Delete:        removes the backing secret; the canonical target is left untouched
             (it may still be serving another profile).
Read:          there is no "reveal token" path — the UI can only read usernames.
```

### Transactional switching (profile apply)

`ProfileService::apply` is a saga with compensating rollback so the machine never
lands in a half-switched identity:

```
snapshot git identity + active_id
1. git config   (name / email / signingkey)
2. credentials  ProfileCredentialSync::switch_profile → promote each owned
                credential onto its canonical target (empty = no-op)
3. active_id
on any stage failure → compensate completed stages in reverse, return one
friendly AppError.
```

Hosts a profile has no credential for are left untouched.

### Windows Credential Manager integration

`WindowsCredentialVault` calls `CredWriteW` / `CredReadW` / `CredDeleteW` with
`CRED_TYPE_GENERIC`, `CRED_PERSIST_LOCAL_MACHINE`, and UTF-16LE credential blobs —
matching Git's convention so Git actually uses the stored credential. It touches
only `git:` / `gitpersona:` targets and never enumerates unrelated OS credentials.
This is the crate's single `unsafe` module (see ADR 004).

### Security model

- Token is write-only across the IPC boundary — input only, never output.
- Metadata (`gitpersona.json`) never contains secrets.
- All transient secret buffers are `zeroize`d; rollback buffers included.
- Audit records action + host + profile, never token material.

### Extension points

`CredentialVault` is the platform seam. Adding macOS Keychain or Linux Secret
Service means one new `infra/` adapter (using that platform's Git credential-helper
convention) wired in `state.rs` under a `#[cfg(...)]` — no domain, service, or
frontend changes.

## Smart Identity Switching (Sprint 6)

An orchestration layer that applies a repository's assigned profile automatically
when the developer starts working in it. It coordinates existing services only —
it never edits Git, SSH, or the vault itself (see ADR 005).

### Components

```
domain/identity_switch.rs   SwitchStatus, SwitchEvent, SmartSwitchStatus, ResolvedRepo
domain/settings.rs          SmartSwitchingSettings (serde-default, forward compatible)
domain/ports.rs             WorkspaceWatcher, IdentityResolver, IdentitySwitcher, SwitchObserver
services/identity_switch_service.rs   IdentitySwitchService (orchestrator)
                                      + RepoIdentityResolver / ProfileIdentitySwitcher adapters
infra/git_dir_watcher.rs    notify-debouncer-mini WorkspaceWatcher (native, event-driven)
infra/switch_observer_tauri.rs   emits `smart-switch` / `smart-switch-status` events
commands/identity_switch.rs smart_switch_{status,set_enabled,restart,pause,resume,confirm,cancel}
```

### Detection

The watcher observes each tracked repo's resolved `.git` directory
**non-recursively**, reacting only to `HEAD` / `ORIG_HEAD` / `MERGE_HEAD` /
`index` (checkout / commit / merge / reset / rebase / pull). Working-tree edits,
`.git/objects` churn, `node_modules`, and build output never trigger a switch.
Events are debounced (400 ms); the watcher is paused/resumed by a flag and
rebuilt by `reconcile()` on launch, settings change, restart, and post-scan.

### Switching

`IdentitySwitcher` delegates to `ProfileService::apply` — the same atomic saga
(git config → HTTPS credentials → active marker, with rollback) used for manual
apply. SSH follows via the managed `~/.ssh/config` block. The orchestrator only
decides *whether* to switch; it records `identity.auto_switch` /
`identity.same_profile` / `identity.no_assignment` / `identity.switch_cancelled`
to the audit log and emits a display-only event (never secrets) to the UI.

### Extension points

`WorkspaceWatcher` is the activation seam — editor/terminal detectors become new
`infra/` adapters with no orchestrator change. The rules seam is now the
`RuleResolver` port consulted before the manual assignment (see Rule Engine
below). Neither touches the core of `IdentitySwitchService`.

## Rule Engine (Sprint 7)

A declarative **decision layer** that resolves the correct profile for a
repository automatically — from its path, name, remote URL, host, or owner —
*before* Smart Switching applies it. The engine only decides; it never edits Git,
SSH, credentials, or the active marker (see ADR 006).

```
Watcher → Repository → Rule Engine → Resolved Profile → ProfileService::apply
```

### Components

```
domain/rule.rs        Rule, RuleCondition, RuleSubject, RuleOperator, RuleMatch,
                      RulePreviewInput/Result, RuleSummary, EvaluationContext,
                      parse_remote (host/owner) — all pure, no I/O
domain/ports.rs       RuleStore (persistence), RuleResolver (decision port)
services/rule_service.rs   RuleService: CRUD, duplicate, enable/disable, reorder,
                           evaluate/preview, import/export, + RuleResolver impl
infra/rule_store_tauri.rs  JSON persistence (gitpersona.json, "rules" key)
commands/rules.rs     rule_{list,get,create,update,delete,duplicate,set_enabled,
                      set_all_enabled,reorder,preview,summary,export,import}
```

Frontend: `stores/rules.ts` owns async state; `features/rules/*` renders the list,
the visual IF/THEN builder, and the read-only preview; a Rules nav item, command
palette group, and dashboard widget complete the surface.

### Resolution order

Enabled rules are evaluated by `priority` ascending; **first match wins** and
returns its `target_profile_id`. If no rule matches, resolution **falls back** to
the manual `Repo.active_profile_id` — so existing assignments keep working exactly
as in Sprint 6. A rule match is recorded as `rule.matched`. The orchestrator gains
one dependency (`RuleResolver`) and one step (`resolve_profile`); nothing else in
Smart Switching changes.

### Conditions

`subject · operator · value`, deterministic and declarative only. Path / name /
remote-URL support contains · starts_with · ends_with · equals; remote host and
owner (parsed from the remote URL) support equals only. Matching is
case-insensitive. **No regex, glob, DSL, scripting, or plugins** — evaluation is a
linear scan of string comparisons and can never execute code or reach the network.

### Preview & sharing

`rule_preview` evaluates a hypothetical repository and reports the matched rule,
resolved profile, and reason — never applying a switch or updating dashboard state.
`rule_export` / `rule_import` move the rule set as JSON between machines (the
frontend picks the path via `dialog:allow-save`; file I/O stays in Rust).
