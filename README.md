# GitPersona

A premium cross-platform **Tauri 2** desktop app — the Developer Identity Manager.
Manage multiple Git identities (name, email, signing keys, SSH keys, HTTPS
credentials) and switch between them globally or per repository.

## Features

- **Profiles** — named Git identities with optional GPG signing keys.
- **Repository discovery** and per-repo profile assignment.
- **SSH & HTTPS credential management** (secrets live only in the OS vault).
- **Smart Identity Switching** — applies the right identity automatically when you
  start working in a repository.
- **Rule Engine** (Sprint 7) — resolve the correct profile automatically from a
  repository's path, name, remote URL, host, or owner, so a freshly cloned repo
  gets the right identity with **no manual assignment**. Rules are declarative and
  deterministic (no scripting), support live **preview**, and can be **exported /
  imported as JSON** to share across a team. See
  [docs/adr/006-rule-engine.md](docs/adr/006-rule-engine.md) and
  [docs/architecture.md](docs/architecture.md).

## Development

```bash
npm run tauri dev     # run the app
npm run check         # tsc + Biome
npm test              # frontend unit tests (Vitest)
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
