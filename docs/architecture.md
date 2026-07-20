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
