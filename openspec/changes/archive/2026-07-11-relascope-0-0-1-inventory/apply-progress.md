# Apply Progress: Relascope 0.0.1 Inventory

## Status

Implementation applied and verified with `cargo fmt` and `cargo test` in the user's PowerShell environment.

## Implemented

- Created Rust workspace with `crates/core`, `crates/storage-sqlite`, and `crates/cli`.
- Added `rust-toolchain.toml` targeting stable Rust.
- Added `relascope` CLI with:
  - `init [path]`
  - `repo add <path> [--id <id>]`
  - `scan`
  - `status`
- Added workspace configuration writing/loading through `relascope.yaml`.
- Added local SQLite storage at `.relascope/graph.db`.
- Added tables for workspaces, repositories, scans, and files.
- Added internal UUID generation for workspaces, repositories, scans, and file inventory rows.
- Added visible repository slug generation and validation.
- Added duplicate visible ID checks.
- Added local-only repository registration and Git URL rejection.
- Added shallow scan inventory for all non-excluded files.
- Added default exclusions for `.git`, `node_modules`, `.venv`, `dist`, `build`, `target`, and `.relascope`.
- Added configurable exclusions from `relascope.yaml`.
- Added shallow file classification for code, configuration, documentation, binary, generated, and unknown.
- Added missing repository availability handling without silent deletion.
- Added persisted `status` output that does not re-scan.
- Added heterogeneous fixture `fixtures/polyrepo-basic` with `api-python`, `web-typescript`, and `infra-config`.
- Added ADRs 0001-0004.
- Added README section documenting 0.0.1 scope and path portability limitation.
- Added unit, storage, and CLI integration tests.

## Verification Evidence

User ran:

```powershell
cargo fmt
cargo test
```

Result:

- `cargo fmt`: passed with no reported output.
- CLI integration tests: 3 passed.
- Core unit tests: 5 passed.
- Storage SQLite test: 1 passed.
- Doc tests: passed with 0 tests.
- Total explicit tests: 9 passed.

See `verify-report.md` for full evidence.

## Remaining Verification

None for the approved 0.0.1 inventory scope.

## Risks

- The Pi harness shell still cannot execute Cargo directly, so command execution evidence is user-provided.
- Future CI should run the same `cargo fmt` and `cargo test` commands automatically.
