# Tasks: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Implementation Plan

Keep this slice focused on scan-to-scan diffing and stale graph marking. Do not implement watcher, rename intelligence, Tree-sitter, package resolution, cross-repository resolution, impact analysis, AI, MCP, web UI, or desktop UI.

## 1. Core scan diff types

- [x] Extend `ScanSummary` with `added`, `modified`, `removed`, and `unchanged` counts.
- [x] Add `PreviousFileState` model for previous completed scan rows.
- [x] Add `FileChangeSummary` model for scan diff results.
- [x] Add file identity key based on `repository_id + relative_path`.
- [x] Implement scan diff computation for first scan, unchanged scan, added files, modified files, and removed files.
- [x] Add unit tests for scan diff computation.

## 2. Graph status constants and node status

- [x] Add `STATUS_STALE = "stale"` graph constant.
- [x] Add `status` field to `GraphNode`.
- [x] Update all graph node construction sites to set `active`, `unverified`, or `stale` as appropriate.
- [x] Update graph node tests for new status field if needed.

## 3. SQLite migration for node status

- [x] Add idempotent migration helper that checks `PRAGMA table_info(nodes)`.
- [x] Add `nodes.status TEXT NOT NULL DEFAULT 'active'` only when missing.
- [x] Update node insert/upsert SQL to include `status`.
- [x] Add indexes for status queries if needed.
- [x] Test opening an existing graph database and applying the node status migration.

## 4. Previous completed scan loading

- [x] Add storage method to load latest completed scan ID for a workspace.
- [x] Add storage method to load file states for the previous completed scan.
- [x] Ensure current `running` scan is not treated as previous completed scan.
- [x] Add storage tests for no previous scan and previous scan lookup.

## 5. Stale marking storage methods

- [x] Add method to mark a graph node stale by node ID.
- [x] Add method to mark edges stale by source/target node and relation type.
- [x] Add method to mark removed file facts stale.
- [x] Add method to mark outgoing imports stale for a source module.
- [x] Ensure stale marking does not mark workspace-to-repository containment stale for removed files.
- [x] Add storage tests for stale file/module/import behavior.

## 6. Scan flow integration

- [x] Load previous completed scan file states before finishing the current scan.
- [x] Compute diff after current repository walk.
- [x] Persist current file inventory as before.
- [x] Mark removed-file graph facts stale before current materialization.
- [x] Mark outgoing imports stale for modified code files before current import materialization.
- [x] Materialize current minimal graph and shallow imports after stale marking.
- [x] Attach diff counts to `ScanSummary` before `finish_scan`.
- [x] Preserve repository unavailable behavior.

## 7. CLI scan output

- [x] Print `added`, `modified`, and `removed` counts from `relascope scan`.
- [x] Optionally print `unchanged` if useful and concise.
- [x] Keep existing `scan_id`, `files`, and `unavailable_repositories` output.
- [x] Add CLI integration tests for first scan and unchanged second scan output.

## 8. Graph inspection behavior

- [x] Ensure `relascope graph imports` displays `stale` status for stale import edges.
- [x] Optionally extend `graph summary` with status breakdown if cheap.
- [x] Ensure graph inspection commands do not scan or update availability.
- [x] Add tests for stale import visibility.

## 9. Smoke script updates

- [x] Update `scripts/smoke-graph.ps1` to copy `fixtures/polyrepo-basic` into a temporary fixture directory.
- [x] Register copied fixture repositories instead of checked-in fixtures.
- [x] Verify first scan counts.
- [x] Modify copied fixture to remove or change an import line.
- [x] Verify second scan reports `modified: 1`.
- [x] Verify stale import behavior after modification.
- [x] Remove a copied fixture file.
- [x] Verify later scan reports `removed: 1`.
- [x] Ensure checked-in fixture files are never mutated by the smoke script.

## 10. Documentation and ADRs

- [x] Update `README.md` if scan output examples change.
- [x] Add or update tutorial for `0.0.3-a` incremental stale graph behavior.
- [x] Add ADR for stale marking rather than deleting graph facts.
- [x] Document that `renames = removed + added` for this slice.
- [x] Document that watcher is deferred.

## 11. Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run `pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1`.
- [x] Verify first scan reports added files.
- [x] Verify unchanged second scan reports zero added/modified/removed.
- [x] Verify modified file reports modified count.
- [x] Verify removed file reports removed count.
- [x] Verify stale imports remain visible with `stale` status.
- [x] Verify current imports can be restored to `active` or `unverified` if reintroduced.
- [x] Verify existing inventory, graph summary, and graph imports behavior still works.

## Review Workload Forecast

This change is medium-to-large because it touches scan state, graph lifecycle, storage migrations, CLI output, smoke tests, and docs. Keep it bounded. If implementation grows too much, split into:

1. Diff model + scan output.
2. Node status migration + stale marking.
3. Smoke/docs and final verification.
