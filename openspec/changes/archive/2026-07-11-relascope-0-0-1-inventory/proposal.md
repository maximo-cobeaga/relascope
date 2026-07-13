# Change Proposal: Relascope 0.0.1 Inventory

## Status

Draft for approval before implementation.

## Problem Statement

Relascope needs a narrow first milestone that proves it can create a durable local workspace, register multiple existing local repositories, inventory their files without executing repository code, and reopen later without losing identity or configuration.

The product vision depends on a trustworthy incremental knowledge base, but 0.0.1 must avoid premature graph, AI, desktop, MCP, and impact-analysis scope. The first useful proof is persistence and inventory across heterogeneous local repositories.

## Goals

- Create a Rust-based Relascope workspace and CLI foundation.
- Support `relascope init` and `relascope init <path>`.
- Support adding existing local repositories only.
- Persist workspace configuration, repository metadata, and scan inventory locally.
- Generate stable internal UUIDs for workspaces and repositories.
- Generate unique visible repository IDs as slugs, with optional `--id <id>` override.
- Inventory all non-excluded files, including unknown files.
- Classify files at a shallow inventory level: recognized code, unknown, configuration, documentation, binary, generated, or similar.
- Detect unavailable registered repositories without deleting them silently.
- Provide `relascope status` from persisted state without running a new scan.
- Include a small heterogeneous polyrepo fixture.
- Establish Rust test execution via `cargo test`.
- Record initial ADRs for foundational technology and persistence choices.

## Non-Goals

- No Git URL registration or cloning.
- No semantic code analysis.
- No dependency graph, cross-repository resolution, or impact analysis.
- No AI usage, provider configuration, prompt execution, or network requirement.
- No MCP server.
- No desktop application.
- No web UI.
- No execution of code from registered repositories.
- No automatic removal of missing repositories.
- No fully portable workspace persistence design yet; 0.0.1 documents limitations and preserves enough path metadata to evolve.

## User-Facing Commands

```bash
relascope init
relascope init <path>
relascope repo add <path>
relascope repo add <path> --id <id>
relascope scan
relascope status
```

## Key Decisions

- The workspace is local-first and stored under `.relascope/`.
- The visible workspace configuration is stored in `relascope.yaml`.
- The local database is stored at `.relascope/graph.db`.
- Internal identity uses UUIDs, not filesystem paths.
- Repository visible IDs are unique slugs within the workspace.
- Repository paths are canonicalized for operation and persisted with explicit portability limitations.
- `scan` inventories every non-excluded file; it does not only store recognized source files.
- Missing repository paths are reported as unavailable and remain in persistent state.
- Default exclusions include `.git`, `node_modules`, `.venv`, `dist`, `build`, and heavy/binary patterns, with configuration overrides.

## Initial Fixture

Create a small heterogeneous polyrepo fixture with:

- `api-python`
- `web-typescript`
- `infra-config`

The fixture must verify multiple repositories, language/file-kind recognition, unknown files, exclusions, and persistence across process runs.

## Acceptance Summary

Relascope 0.0.1 is acceptable when a user can initialize a workspace, register local repositories, scan them, exit, run a new process, and view the same workspace ID, repositories, internal repository IDs, configuration, and last scan inventory through `relascope status` without re-scanning.
