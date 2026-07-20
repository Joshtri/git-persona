# ADR 004 — HTTPS Credential Manager: OS vault, not our own

**Status**: Accepted
**Date**: Sprint 5

## Context

A profile is a complete developer identity (Git config + SSH + HTTPS credential).
Sprint 5 adds the HTTPS credential so that applying a profile also switches the
Git credential a host authenticates with. Storing authentication secrets demands
the highest security bar: GitPersona must never become a secret vault, never log
or serialize tokens, and must only touch Git-related OS credentials.

## Decision

### Secrets live only in the OS secure store

GitPersona persists **metadata only** (`gitpersona.json`, `credentials` key). Every
secret lives in the OS credential store, reached through the `CredentialVault`
port. The `Credential` domain type has no secret field, so no token can be
serialized, logged, or returned over IPC. Tokens are `Zeroizing<String>` for their
short in-memory lifetime.

### Windows: native Win32, not the `keyring` crate

`WindowsCredentialVault` calls `CredWriteW` / `CredReadW` / `CredDeleteW` directly
with target `git:https://<host>`, `CRED_TYPE_GENERIC`, `CRED_PERSIST_LOCAL_MACHINE`,
and UTF-16LE blobs. This matches Git's `wincred`/GCM convention so Git actually
uses the credential. The `keyring` crate was rejected because its Windows target
naming would not match Git's, so Git would ignore the stored secret.

### Two vault targets so multiple profiles can hold the same host

Each credential's secret is parked under a namespaced **backing** target
`gitpersona:<id>:https://<host>`. Applying/switching a profile **promotes** that
secret onto the **canonical** `git:https://<host>` target. All secrets — active or
not — stay in the OS store; GitPersona holds no plaintext.

### `unsafe_code` relaxed from `forbid` to `deny`

The Win32 FFI requires `unsafe`, which `forbid` cannot override. The crate lint was
changed to `deny` so exactly one module, `infra/credential_vault_windows.rs`,
carries a scoped, audited `#![allow(unsafe_code)]` with a SAFETY note on every
block. Unsafe remains denied everywhere else.

### Applying a profile is a transactional saga

`ProfileService::apply` runs git config → credential switch → active id, snapshots
prior state, and compensates completed stages in reverse on any failure. Credential
access stays inside `CredentialService` via the `ProfileCredentialSync` port; the
pipeline only carries an opaque `CredentialTxn` of rollback snapshots.

## Consequences

- No plaintext credential ever exists inside GitPersona's own files.
- Git picks up switched credentials because targets match its helper convention.
- macOS Keychain / Linux Secret Service are additive: one new `CredentialVault`
  adapter each, wired by `#[cfg(...)]` in `state.rs`. No other layer changes.
- The codebase now contains one audited `unsafe` module; all other code is
  `unsafe`-free.
- WCM and the filesystem are not truly transactional, so rollback is best-effort
  compensation (a saga), not ACID — acceptable for local identity switching.
