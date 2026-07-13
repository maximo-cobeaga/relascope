# Change Proposal: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Status

Draft for approval before specification, design, tasks, and implementation.

## Problem Statement

Relascope now inventories files, persists a minimal graph, and detects shallow module imports. Re-running scans updates and upserts known facts, but the graph does not yet clearly mark facts that became obsolete when files are removed or modified.

This is a trust problem. If Relascope continues to accumulate file, module, and import facts without marking stale knowledge, future graph queries and impact analysis can mislead users. Before adding richer parsers or cross-repository resolution, Relascope needs basic incremental scan state and stale graph marking.

## Goals

- Compare the current scan with the previous completed scan for the same workspace.
- Detect added, modified, unchanged, and removed files per repository.
- Preserve existing per-scan `files` inventory rows.
- Avoid introducing a `current_files` table in this slice.
- Mark graph facts associated with removed files as `stale` rather than deleting them.
- When a recognized code file changes, mark previous import edges from its module as `stale` before materializing current imports.
- Keep new graph facts active/unverified according to existing import behavior.
- Show an incremental summary in `relascope scan` output.
- Extend automated smoke coverage to verify stale marking.
- Preserve all 0.0.1, 0.0.2-a, and 0.0.2-b behavior.

## Non-Goals

- No file watcher yet.
- No rename detection beyond removed+added.
- No Tree-sitter.
- No package resolution.
- No cross-repository resolution.
- No impact analysis.
- No UI.
- No physical deletion or compaction of stale graph facts.
- No full historical graph browser.
- No new `current_files` table unless implementation proves it is strictly necessary.

## Key Decisions

- Use scan-to-scan comparison against the previous completed scan.
- Keep `files` as per-scan inventory history.
- Mark stale facts instead of deleting them.
- Use `stale` as the primary obsolete-state marker for 0.0.3-a.
- Document `invalidated` as a future/stronger state, but avoid overusing it in this slice.
- Add incremental counts to `scan` output.

## User-Facing Behavior

Existing commands remain:

```bash
relascope init [path]
relascope repo add <path> [--id <id>]
relascope scan
relascope status
relascope graph summary
relascope graph imports
```

`relascope scan` should show an incremental summary, for example:

```text
Scan completed
  scan_id: ...
  files: 12
  added: 1
  modified: 2
  removed: 1
```

On the first scan, all files may be reported as added and removed/modified as zero.

## Stale Marking Rules

### Removed file

If a file existed in the previous completed scan but not in the current scan:

- Mark its `File` node `stale`.
- Mark its source `Module` node `stale` if one exists.
- Mark `CONTAINS` edges connected to that file/module as `stale` where appropriate.
- Mark `IMPORTS` edges originating from that module as `stale`.
- Preserve evidence rows, updating metadata/status only if needed.

### Modified code file

If a recognized code file exists in both scans but its content hash changed:

- Mark previous `IMPORTS` edges originating from that file's module as `stale`.
- Re-detect current imports.
- Upsert current `IMPORTS` edges as `active` or `unverified` according to resolution.

### Unchanged file

If a file exists in both scans and hash/metadata indicates it is unchanged:

- Preserve current active graph facts.
- Re-materialization can still upsert timestamps, but should not create duplicate nodes or edges.

## Acceptance Summary

Relascope 0.0.3-a is acceptable when repeated scans can detect added, modified, and removed files, print incremental counts, mark obsolete graph facts as `stale`, and keep current facts active/unverified. Automated tests and smoke coverage must prove that deleting or modifying a fixture file changes graph/import state without duplicating facts or silently leaving obsolete facts active.

## Review Workload Note

This change touches scan flow, storage queries, graph status updates, smoke tests, and docs. Keep it focused on stale marking, not watcher or rename intelligence. If implementation grows too large, split into:

1. Scan diff read model and CLI summary.
2. Stale marking for removed files.
3. Modified-file import stale marking and smoke/docs.
