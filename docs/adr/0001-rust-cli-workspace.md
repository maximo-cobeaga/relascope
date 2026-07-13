# ADR 0001: Rust CLI Workspace Foundation

## Status

Accepted for Relascope 0.0.1.

## Context

Relascope needs a fast local-first foundation for workspace initialization, repository registration, scan orchestration, and persisted status. The product architecture recommends Rust for the core engine and CLI because the long-term product needs efficient indexing, safe concurrency, and distributable binaries.

## Decision

Use a Rust workspace with three initial crates:

- `relascope-core` for domain types and deterministic inventory logic.
- `relascope-storage-sqlite` for local SQLite persistence.
- `relascope-cli` for the `relascope` binary and user-facing commands.

Use `clap` for command parsing and `cargo test` as the initial test command.

## Consequences

This keeps 0.0.1 small while preserving room for later crates such as graph, analyzer runtime, impact engine, local API, MCP, and desktop integration. It avoids introducing UI or agent concerns before the local inventory proof works.
