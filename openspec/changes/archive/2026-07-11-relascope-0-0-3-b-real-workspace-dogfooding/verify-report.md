# Verify Report: Relascope 0.0.3-b Real Workspace Dogfooding

## Status

Verified.

## Verification Commands

The user ran from PowerShell in `C:\Users\MAXIMO\Desktop\relascope`:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

All passed.

## Test Evidence

`cargo test` passed with:

- CLI integration tests: 9 passed.
- Core unit tests: 11 passed.
- Storage SQLite tests: 3 passed.
- Doc-tests: passed with 0 tests.

Total explicit tests: 23 passed.

## Smoke Evidence

The automated smoke script passed end-to-end.

Covered behavior:

- Builds Relascope CLI.
- Creates a temporary workspace.
- Copies fixture repositories into a mutable temp fixture.
- Initializes workspace.
- Adds copied `api`, `web`, and `infra` repos.
- Runs `repo list`.
- Runs `doctor` before scan.
- Verifies graph summary/imports empty states before scan.
- Runs first scan.
- Verifies human graph summary.
- Verifies JSON graph summary.
- Runs `doctor` after scan.
- Verifies human graph imports.
- Verifies JSON graph imports.
- Runs unchanged second scan.
- Verifies JSON scan output.
- Modifies copied fixture and verifies `modified: 1`.
- Verifies stale import behavior.
- Removes copied fixture file and verifies `removed: 1`.
- Removes `infra` repository registration.
- Verifies files on disk were not deleted.
- Verifies `repo list` no longer shows `infra`.

## Key Verified Outputs

Initial graph summary after scan:

```text
nodes: 21
File: 12
Module: 5
Repository: 3
Workspace: 1
edges: 22
CONTAINS: 19
IMPORTS: 3
```

Doctor after scan:

```text
repositories: 3 total, 3 available, 0 unavailable
last_scan: completed
stale_nodes: 0
stale_edges: 0
unverified_imports: 1
```

JSON graph summary was valid and included:

```json
{
  "total_nodes": 21,
  "total_edges": 22
}
```

JSON graph imports was valid and included active/unverified imports with evidence.

Unchanged scan JSON included:

```json
{
  "added": 0,
  "files": 12,
  "modified": 0,
  "removed": 0,
  "status": "completed",
  "unchanged": 12
}
```

Modified copied fixture verified:

```text
files: 12
added: 0
modified: 1
removed: 0
api/app.py -> api:os [stale]
```

Removed copied fixture file verified:

```text
files: 11
added: 0
modified: 0
removed: 1
api/app.py -> api/settings.py [stale]
api/app.py -> api:.settings [unverified]
```

Repo removal verified:

- `relascope repo remove infra` succeeded.
- `repo list` no longer showed `infra`.
- The copied `infra` directory remained on disk.

## Acceptance Coverage

- `repo list` works.
- `repo remove <id>` removes registration without deleting files.
- `doctor` reports useful health state and remains read-only.
- `scan --format json` outputs valid JSON.
- `graph summary --format json` outputs valid JSON.
- `graph imports --format json` outputs valid JSON.
- Existing inventory, graph, import, incremental, and stale behavior still works.
- Dogfooding documentation exists.

## Scope Boundaries Confirmed

This verification does not include, and the implementation intentionally does not add:

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

## Conclusion

Relascope 0.0.3-b satisfies the approved OpenSpec scope and is ready for sync/archive.
