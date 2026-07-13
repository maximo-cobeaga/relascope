# Apply Progress: Relascope 0.0.3-b Real Workspace Dogfooding

## Status

Implementation applied and verified. Ready for OpenSpec sync/archive.

## Implemented

- Added command-local output format support with `--format human|json`.
- Added JSON output for:
  - `relascope scan --format json`
  - `relascope graph summary --format json`
  - `relascope graph imports --format json`
- Kept human output as the default.
- Added repository management commands:
  - `relascope repo list`
  - `relascope repo remove <id>`
- `repo remove` deletes only the repository registration and never deletes files from disk.
- Added read-only doctor command:
  - `relascope doctor`
- Doctor reports config, database, workspace identity, repository health, last scan, stale graph counts, unverified imports, and exclusions.
- Doctor does not scan file trees and does not update repository availability state.
- Added storage support for repository removal, doctor health read model, and graph status counts.
- Added/updated CLI integration tests.
- Updated smoke script to call `repo list`, `doctor`, and JSON output checks.
- Added dogfooding guide:
  - `tutorial/dogfooding-real-workspace.md`
- Updated README with dogfooding guide and new commands.

## Verification Evidence

User ran:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Result: passed.

Test evidence:

- CLI integration tests: 9 passed.
- Core unit tests: 11 passed.
- Storage SQLite tests: 3 passed.
- Total explicit tests: 23 passed.

Smoke verified:

- `repo list`.
- `doctor` before/after scan.
- human and JSON graph summary.
- human and JSON graph imports.
- JSON scan output.
- incremental modified/removed behavior.
- stale and unverified import behavior.
- `repo remove infra` removes registration without deleting files.

See `verify-report.md` for full evidence.

## Scope Preserved

Still intentionally out of scope:

- watcher
- Tree-sitter
- deeper semantic parsing
- cross-repository resolution
- impact analysis
- AI
- MCP
- web UI
- desktop UI
- stable JSON API contract

## Remaining Verification

None for the approved 0.0.3-b dogfooding scope.
