# Sync Report: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Status

Synced.

## Synced Specification

The approved and verified incremental stale graph specification was promoted from the change delta into the active OpenSpec specification tree.

- Source: `openspec/changes/relascope-0-0-3-a-incremental-stale-graph/specs/incremental-stale-graph/spec.md`
- Target: `openspec/specs/incremental-stale-graph/spec.md`

## Verification Basis

This sync is based on the verified implementation recorded in:

- `openspec/changes/relascope-0-0-3-a-incremental-stale-graph/apply-progress.md`
- `openspec/changes/relascope-0-0-3-a-incremental-stale-graph/verify-report.md`

Verification evidence:

- `cargo fmt`: passed according to user-side PowerShell execution.
- `cargo test`: passed according to user-side PowerShell execution.
- Automated graph/import/incremental smoke test passed.
- First scan reports all files as added.
- Unchanged scan reports zero added/modified/removed.
- Modified file reports modified count.
- Removed file reports removed count.
- Stale import facts remain visible with `stale` status.
- Current unresolved import facts materialize as `unverified`.

## Notes

Relascope 0.0.3-a remains limited to scan-to-scan diffing and stale graph marking. Watcher, rename intelligence, Tree-sitter, package resolution, cross-repository resolution, impact analysis, AI, MCP, web UI, and desktop UI remain out of scope.
