# ADR 003 — IPC boundary isolation: only src/ipc/ imports @tauri-apps/api

**Status**: Accepted  
**Date**: Sprint 0

## Context

Tauri's `invoke` and `listen` calls couple components to the native runtime, making them impossible to test in jsdom and hard to mock in Storybook or isolated component tests.

## Decision

`@tauri-apps/api` is imported **only** in:
- `src/ipc/client.ts` — typed `invoke` wrappers
- `src/ipc/events.ts` — typed `listen` helpers

Feature components and stores import from `@/ipc/client` (project alias), never from `@tauri-apps/api` directly. Biome lint rules and TypeScript paths enforce this at the module level.

## Consequences

- All frontend tests mock `@/ipc/client` with `vi.mock`, not Tauri internals.
- Replacing Tauri with Electron (unlikely) would only require rewriting `src/ipc/`.
- Stores are the single point of truth for loading/error/data from any IPC call.
