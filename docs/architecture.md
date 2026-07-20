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
type View = { name: "profiles" } | { name: "repos" } | { name: "settings" } | { name: "onboarding" }
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
