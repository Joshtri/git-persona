# ADR 002 — Ports & Adapters (Hexagonal) for Rust backend

**Status**: Accepted  
**Date**: Sprint 0

## Context

The Rust backend needs to: read/write ~/.gitconfig, persist profile data, and write an audit log. These are I/O concerns that should be replaceable (for testing and future platforms) without changing business logic.

## Decision

Three port traits in `domain/ports.rs`:
- `GitConfigBackend` — read/write global gitconfig
- `ProfileStore` — CRUD for profiles + settings persistence  
- `AuditSink` — append-only audit log writes

Concrete implementations live in `infra/`. Services receive `Arc<dyn Trait>` and never import `infra` directly.

## Consequences

- Domain and services are testable with mock implementations.
- `infra/` is the only layer that can fail at integration boundaries.
- A future "dry-run" mode can swap in a no-op `AuditSink` at startup.
- A Clock port was evaluated and cut — `chrono::Utc::now()` inline is sufficient; the abstraction would only matter for deterministic testing of time-sensitive logic, which doesn't exist in Sprint 0.
