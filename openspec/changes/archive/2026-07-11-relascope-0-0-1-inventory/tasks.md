# Tasks: Relascope 0.0.1 Inventory

## Implementation Plan

Do not implement graph analysis, AI, MCP, desktop, web UI, or impact analysis in this change.

## 1. Rust workspace foundation

- [x] Create root `Cargo.toml` as a Rust workspace.
- [x] Add `rust-toolchain.toml` using stable Rust.
- [x] Create crates: `crates/core`, `crates/storage-sqlite`, and `crates/cli`.
- [x] Wire crate dependencies so the CLI depends on core and storage through narrow interfaces.
- [x] Add baseline `cargo test` support.

## 2. CLI command shell

- [x] Add `clap` command parsing for `init`, `repo add`, `scan`, and `status`.
- [x] Add human-readable success and error output.
- [x] Return stable non-zero exit codes for configuration and execution errors where practical.
- [x] Add CLI smoke tests for command parsing.

## 3. Workspace initialization

- [x] Implement `relascope init` for the current directory.
- [x] Implement `relascope init <path>` for an explicit target path.
- [x] Create `relascope.yaml` with workspace UUID, name, version, and default exclusions.
- [x] Create `.relascope/graph.db` and run migrations.
- [x] Reject accidental reinitialization without overwriting existing workspace identity.
- [x] Test initialization in current and explicit directories.

## 4. SQLite storage foundation

- [x] Add migrations for `workspaces`, `repositories`, `scans`, and `files`.
- [x] Implement storage open/migrate logic.
- [x] Implement repository persistence with unique `(workspace_id, visible_id)`.
- [x] Implement scan and file inventory persistence.
- [x] Implement status read models from persisted state.
- [x] Test migrations and persistence round-trips.

## 5. Repository registration

- [x] Implement workspace discovery/loading for commands run inside a workspace.
- [x] Implement `relascope repo add <path>` for existing local directories.
- [x] Reject Git URLs and missing paths for 0.0.1.
- [x] Generate stable internal repository UUIDs.
- [x] Generate default visible slug IDs from folder names.
- [x] Implement optional `--id <id>` override.
- [x] Validate visible ID format and uniqueness within the workspace.
- [x] Store canonical absolute path and explicit path metadata/limitation.
- [x] Test generated IDs, explicit IDs, duplicate rejection, and missing/URL rejection.

## 6. Exclusion and classification utilities

- [x] Implement default exclusion patterns for `.git`, `node_modules`, `.venv`, `dist`, `build`, `target`, and `.relascope`.
- [x] Load user-defined exclusions from `relascope.yaml`.
- [x] Implement shallow file kind classification: code, configuration, documentation, binary, generated, unknown.
- [x] Implement extension-based language detection for initial supported extensions.
- [x] Avoid reading entire heavy files into memory.
- [x] Test exclusion matching and file classification.

## 7. Scan command

- [x] Implement `relascope scan` over all registered repositories.
- [x] Persist all non-excluded files, including unknown files.
- [x] Store repository ID, relative path, kind, language, size, optional content hash, modified timestamp, scan ID, and metadata.
- [x] Mark missing registered repository paths as unavailable without deleting them.
- [x] Continue scanning other repositories when one repository is unavailable.
- [x] Persist scan summary counts by repository, kind, language, and availability.
- [x] Ensure scan never executes repository code, uses network, or calls AI.
- [x] Test scan with fixture repositories and missing repository paths.

## 8. Status command

- [x] Implement `relascope status` from persisted state only.
- [x] Report workspace ID, workspace name, root, registered repositories, repository UUIDs, visible IDs, paths, and availability.
- [x] Report last scan timestamp, status, and inventory summary.
- [x] Report a clear no-scan-yet message when appropriate.
- [x] Add a regression test proving `status` does not perform a new scan.
- [x] Test status after reopening the workspace from a separate process or fresh storage connection.

## 9. Polyrepo fixture

- [x] Create `fixtures/polyrepo-basic/api-python` with Python, docs, unknown, and excluded files.
- [x] Create `fixtures/polyrepo-basic/web-typescript` with TypeScript, package config, docs, and excluded files.
- [x] Create `fixtures/polyrepo-basic/infra-config` with Docker/YAML/config files and excluded build output.
- [x] Use the fixture in scan and status tests.

## 10. Documentation and ADRs

- [x] Create ADR for Rust workspace/CLI foundation.
- [x] Create ADR for SQLite local persistence and `.relascope/graph.db` naming.
- [x] Create ADR for local-only repository registration in 0.0.1.
- [x] Create ADR for canonical path persistence limitation and future portability direction.
- [x] Document 0.0.1 scope and non-goals in project documentation.

## 11. Verification before completion

- [x] Run `cargo test`.
- [x] Manually verify `relascope init`, `repo add`, `scan`, and `status` against the polyrepo fixture if CLI integration tests do not fully cover it.
- [x] Verify a reopened workspace preserves workspace UUID, repository UUIDs, visible IDs, configuration, and last scan inventory.
- [x] Verify a missing registered repository is reported unavailable and not deleted.
- [x] Verify excluded directories are not inventoried.

## Review Workload Forecast

Expected implementation size is likely above 400 changed lines if completed as one PR because it creates a Rust workspace, storage, CLI, fixture, tests, and ADRs. Prefer implementation in reviewable work units or chained PRs:

1. Workspace + CLI shell + init.
2. Storage + repo add.
3. Scan + fixture + status.
4. Documentation/ADRs and final verification.

Before `sdd-apply`, confirm whether to implement as chained slices or a single local change set.
