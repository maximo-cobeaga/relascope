# Apply Progress: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Status

Implementation applied and verified. Ready for OpenSpec sync/archive.

## Implemented

- Extended `ScanSummary` with incremental counts:
  - `added`
  - `modified`
  - `removed`
  - `unchanged`
- Added scan diff models and logic:
  - `PreviousFileState`
  - `FileIdentityKey`
  - `FileChangeSummary`
  - `compute_file_changes`
- Added graph stale status:
  - `STATUS_STALE`
- Added `GraphNode.status`.
- Added SQLite migration support for `nodes.status`.
- Updated node upsert SQL to persist node status.
- Added previous completed scan file loading.
- Added stale marking for removed files and modified code-file imports.
- Updated scan flow to compute diff, mark stale facts, then materialize current facts.
- Updated CLI scan output with incremental counts.
- Updated smoke script to copy checked-in fixtures to a temporary mutable fixture directory.
- Added ADR:
  - `docs/adr/0007-stale-graph-marking.md`
- Added tutorial:
  - `tutorial/onboarding-relascope-0-0-3-a.md`
- Updated README and current-state docs.

## Verification Evidence

User confirmed:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Result: passed.

Verified smoke behavior:

```text
first scan:      files 12, added 12, modified 0, removed 0
unchanged scan:  files 12, added 0,  modified 0, removed 0
modified file:   files 12, added 0,  modified 1, removed 0
removed file:    files 11, added 0,  modified 0, removed 1
```

Verified stale/unverified import behavior:

```text
api/app.py -> api:os [stale]
api/app.py -> api/settings.py [stale]
api/app.py -> api:.settings [unverified]
```

See `verify-report.md` for full evidence.

## Scope Preserved

Still intentionally out of scope:

- watcher
- rename intelligence
- Tree-sitter
- package resolution
- cross-repository resolution
- impact analysis
- AI
- MCP
- web UI
- desktop UI

## Remaining Verification

None for the approved 0.0.3-a incremental stale graph scope.
