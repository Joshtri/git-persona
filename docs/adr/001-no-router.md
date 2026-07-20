# ADR 001 — No router; navigation via Zustand discriminated union

**Status**: Accepted  
**Date**: Sprint 0

## Context

GitPersona is a single-window desktop app with ~4 top-level views (profiles, repos, settings, onboarding). Traditional SPA routers (React Router, TanStack Router) add URL management, history stack, and code-splitting concerns that are irrelevant for a desktop app with no deep-linking requirements.

## Decision

Navigation state is a Zustand store holding a discriminated union `View` type. Components call `navigate({ name: "settings" })`. The `AppShell` renders the active view imperatively.

## Consequences

- Zero routing library overhead in the bundle.
- TypeScript exhaustiveness checking on `View` variants at compile time.
- No back/forward history — not needed for this use case.
- Adding a new view requires: (1) add a variant to the union, (2) handle it in AppShell. That's it.
