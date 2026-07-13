# Tasks: Relascope 0.0.2-a Minimal Graph Schema

## Implementation Plan

Keep this slice focused on minimal graph persistence and inspection. Do not implement imports, module nodes, semantic parsing, impact analysis, AI, MCP, web UI, or desktop UI.

## 1. Core graph domain types

- [x] Add graph domain structs for nodes, edges, evidence, and graph summary.
- [x] Add deterministic graph ID helper functions.
- [x] Add tests for stable workspace, repository, file, and edge IDs.
- [x] Add graph kind/relation constants or enums for `Workspace`, `Repository`, `File`, and `CONTAINS`.

## 2. SQLite graph schema

- [x] Add migrations for `nodes`, `edges`, and `evidence` tables.
- [x] Add uniqueness constraint for `(workspace_id, source_node_id, target_node_id, relation_type)` on edges.
- [x] Add indexes needed for graph summary counts.
- [x] Ensure migrations are idempotent for existing 0.0.1 workspaces.
- [x] Test opening an existing inventory database and applying graph migrations.

## 3. Graph persistence methods

- [x] Implement upsert for graph nodes.
- [x] Implement upsert for graph edges.
- [x] Implement insert or upsert for basic evidence.
- [x] Implement graph summary query returning total nodes, nodes by kind, total edges, and edges by relation.
- [x] Test persistence and summary counts directly through storage.

## 4. Minimal graph materialization

- [x] Implement materialization of one `Workspace` node per workspace.
- [x] Implement materialization of one `Repository` node per registered repository, including unavailable repositories.
- [x] Implement materialization of one `File` node per current scan file inventory row.
- [x] Implement `Workspace CONTAINS Repository` edges.
- [x] Implement `Repository CONTAINS File` edges.
- [x] Add basic evidence for repository registration and file inventory facts.
- [x] Ensure repeated scans do not duplicate nodes or edges.
- [x] Document that stale graph pruning is out of scope.

## 5. Scan integration

- [x] Extend `relascope scan` to materialize the minimal graph after file inventory persistence.
- [x] Preserve all existing scan output or add graph counts only if concise.
- [x] Ensure scan still marks missing repositories unavailable without deleting them.
- [x] Ensure scan still avoids network, AI, and repository code execution.
- [x] Extend CLI integration tests to assert graph materialization after scan.

## 6. CLI graph summary

- [x] Add nested CLI command parsing for `relascope graph summary`.
- [x] Implement graph summary output from persisted state.
- [x] Report a clear no-graph-yet message before first materialized scan.
- [x] Ensure `graph summary` does not scan repositories or update availability.
- [x] Add CLI integration tests for graph summary before and after scan.

## 7. Documentation

- [x] Update `README.md` with `relascope graph summary`.
- [x] Update `tutorial/onboarding-relascope-0-0-1.md` or create a new `tutorial/onboarding-relascope-0-0-2-a.md` explaining the graph layer.
- [x] Add an ADR for introducing minimal graph persistence.
- [x] Document the stale graph pruning limitation.

## 8. Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Manually smoke test `graph summary` against `fixtures/polyrepo-basic`.
- [x] Verify expected fixture counts after full scan:
  - [x] `Workspace`: 1
  - [x] `Repository`: 3
  - [x] `File`: 10
  - [x] total nodes: 14
  - [x] `CONTAINS`: 13
- [x] Verify `graph summary` before scan exits successfully.
- [x] Verify repeated scan does not duplicate nodes or edges.

## Review Workload Forecast

Likely medium-sized but bounded. The highest-risk areas are schema changes, deterministic IDs, repeated-scan idempotency, and CLI behavior. If the implementation starts exceeding comfortable review size, split into:

1. Schema + storage + graph summary query.
2. Scan materialization + CLI command + docs/tests.
