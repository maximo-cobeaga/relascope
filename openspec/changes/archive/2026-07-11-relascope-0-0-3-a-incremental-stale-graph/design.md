# Design: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Overview

Relascope 0.0.3-a adds the first incremental scan semantics. The goal is not to implement a watcher or full graph invalidation engine. The goal is to compare the current scan against the previous completed scan, report file changes, and mark graph facts tied to removed or modified files as `stale` instead of leaving obsolete facts active.

This protects graph trust before adding richer parsers or impact analysis.

## Current Foundation

Relascope already has:

- per-scan file inventory in `files`;
- scan records in `scans`;
- graph tables: `nodes`, `edges`, `evidence`;
- deterministic IDs for workspace, repository, file, module, and import facts;
- `Workspace`, `Repository`, `File`, and `Module` nodes;
- `CONTAINS` and `IMPORTS` edges;
- shallow import materialization;
- `graph summary` and `graph imports`.

## Architecture Slice

```text
relascope scan
  ├─ load previous completed scan file state
  ├─ walk repositories and produce current file inventory
  ├─ compute scan diff
  │    ├─ added
  │    ├─ modified
  │    ├─ unchanged
  │    └─ removed
  ├─ persist current file inventory
  ├─ mark stale graph facts for removed files
  ├─ mark stale previous imports for modified code files
  ├─ materialize current graph facts
  ├─ materialize current shallow imports
  └─ finish scan with incremental summary
```

## Data Model Strategy

### Keep `files` per scan

Do not add a `current_files` table in 0.0.3-a.

Use query/read-model helpers to load:

- current scan files from the in-memory scan outcome;
- previous completed scan files from SQLite.

The comparison key is:

```text
repository_id + relative_path
```

### Scan summary JSON

Extend `ScanSummary` with incremental fields:

```rust
pub struct ScanSummary {
    pub total_files: u64,
    pub by_repository: BTreeMap<String, u64>,
    pub by_kind: BTreeMap<String, u64>,
    pub by_language: BTreeMap<String, u64>,
    pub unavailable_repositories: Vec<String>,
    pub added: u64,
    pub modified: u64,
    pub removed: u64,
    pub unchanged: u64,
}
```

Existing summary consumers should continue to work because the JSON gains optional/new fields and Rust tests can be updated.

## Scan Diff Read Model

Add a core type:

```rust
pub struct FileChangeSummary {
    pub added: u64,
    pub modified: u64,
    pub removed: u64,
    pub unchanged: u64,
    pub removed_files: Vec<PreviousFileState>,
    pub modified_files: Vec<FileInventoryRecord>,
}
```

Previous file state should include enough data to identify graph facts:

```rust
pub struct PreviousFileState {
    pub repository_id: String,
    pub relative_path: String,
    pub content_hash: Option<String>,
    pub language: Option<String>,
    pub kind: String,
}
```

## Previous Scan Lookup

Add storage method:

```rust
previous_completed_scan_files(workspace_id, current_scan_id) -> Vec<PreviousFileState>
```

Important: call this before finishing the current scan, and exclude the current scan. Simpler implementation:

1. Query latest completed scan for workspace before current scan was created, or latest scan with `status = 'completed'`.
2. Load its `files` rows.

Because current scan starts with `running`, querying `status = 'completed'` naturally returns the previous completed scan.

## Diff Algorithm

Build maps:

```text
previous: (repository_id, relative_path) -> PreviousFileState
current:  (repository_id, relative_path) -> FileInventoryRecord
```

Rules:

- In current but not previous → added.
- In previous but not current → removed.
- In both and content hash differs → modified.
- In both and content hash same → unchanged.

If either hash is missing:

- Fall back to size/modified time if available later.
- For 0.0.3-a, treat missing-vs-missing as unchanged only when both are missing and size/metadata did not obviously change.
- Conservative acceptable fallback: if hashes cannot be compared and file exists in both, treat as unchanged to avoid noisy false positives.

## Stale Status Model

Add graph status constant:

```rust
STATUS_STALE = "stale"
```

`invalidated` remains a documented future concept, not broadly used here.

### Nodes

`nodes` currently has no `status` column. Two design options:

1. Add `status TEXT NOT NULL DEFAULT 'active'` to nodes.
2. Store status only in `metadata_json`.

Recommended: add a real `status` column to `nodes`.

Why:

- status is a first-class graph concept;
- queries and future UI need it;
- edges already have `status`;
- storing node status only in JSON would become technical debt immediately.

Migration:

```sql
ALTER TABLE nodes ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
```

SQLite caveat: `ALTER TABLE ... ADD COLUMN` must be guarded because repeated migration attempts fail if the column already exists. Implement a helper that checks `PRAGMA table_info(nodes)` before adding the column.

Update node upsert to set status.

### Edges

Edges already have `status`; update stale edges with SQL.

### Evidence

Evidence has no status. For 0.0.3-a, do not add evidence status unless needed. Evidence remains historical support for stale edges. If necessary, annotate `metadata_json` later.

## Stale Marking Rules

### Removed file

For each removed file:

1. Compute file node ID:

```text
file_node_id(workspace_id, repository_id, relative_path)
```

2. Compute module node ID if language is supported code:

```text
module_node_id(workspace_id, repository_id, relative_path)
```

3. Mark stale:

- file node;
- module node;
- edges where source or target is file node and relation is `CONTAINS`;
- edges where source or target is module node and relation is `CONTAINS`;
- edges where source is module node and relation is `IMPORTS`.

Do not mark workspace→repository `CONTAINS` stale for removed files.

### Modified code file

For each modified file whose language is supported code:

1. Compute source module node ID.
2. Mark outgoing `IMPORTS` edges from that source module as `stale`.
3. Materialize current imports afterward.

If the same import still exists, upsert will restore its edge status to `active` or `unverified`.

## Ordering During Scan

Recommended order:

1. Load previous completed file states.
2. Begin current scan.
3. Walk repositories and produce current files.
4. Compute diff from previous states + current files.
5. Persist current files.
6. Update repository availability.
7. Mark stale removed-file facts.
8. Mark stale outgoing imports for modified code files.
9. Materialize minimal graph for current files.
10. Materialize shallow imports for current files.
11. Attach diff counts to summary.
12. Finish scan.

Rationale:

- stale marking happens before current materialization;
- current materialization can restore current facts;
- removed files are not materialized, so their stale status remains.

## CLI Output

`relascope scan` should print:

```text
Scan completed
  scan_id: <uuid>
  files: 12
  added: 0
  modified: 1
  removed: 0
```

Keep output concise. Do not list every changed file by default.

## Graph Imports Output

`graph imports` already prints edge status. No new command is required.

A stale import should appear as:

```text
Graph imports
  web/src/main.ts -> web/src/message.ts [stale]
    evidence: src/main.ts:1 import { message } from "./message";
```

If the same import still exists after modification, materialization restores it to active/unverified.

## Smoke Script Update

Extend `scripts/smoke-graph.ps1`:

1. Run initial scan and verify current counts.
2. Modify a fixture copy, not the checked-in fixture.
3. Run second scan.
4. Verify `modified: 1`.
5. Remove or change an import line.
6. Verify old import appears stale or no longer active.
7. Remove a file in the copied fixture workspace.
8. Run third scan.
9. Verify `removed: 1` and no duplicate graph counts.

Important: to avoid mutating checked-in fixtures, the smoke script should copy `fixtures/polyrepo-basic` into the temporary smoke workspace or temp directory, then register the copied repos.

This is a worthwhile improvement because current smoke registers checked-in fixture paths directly.

## Testing Strategy

Unit tests:

- diff computation first scan;
- diff computation unchanged scan;
- added file;
- modified file hash;
- removed file;
- missing hash fallback.

Storage tests:

- load previous completed scan files;
- add node status migration idempotently;
- mark removed file node stale;
- mark removed module outgoing imports stale;
- modified code file import stale → current materialization restores current import.

CLI integration tests:

- first scan prints added count;
- second unchanged scan prints zero counts;
- modified file prints modified count;
- removed file prints removed count;
- `graph imports` displays stale edge after import removal.

Smoke:

- run complete fixture flow;
- modify copied fixture;
- remove copied fixture file;
- verify summary and stale behavior.

## Tradeoffs

### Add node status column now

Pros:

- Makes graph state queryable.
- Aligns nodes with edge status.
- Avoids hiding lifecycle state in JSON.

Cons:

- Requires migration guard.
- Requires updating node upsert code.

Decision: add `nodes.status`.

### No physical deletion

Pros:

- Preserves evidence/history.
- Avoids accidental data loss.
- Supports later UI/history work.

Cons:

- Graph counts include stale facts.
- Users need status-aware views later.

Decision: mark stale, do not delete.

### No watcher yet

Pros:

- Keeps this milestone bounded.
- Avoids long-running process semantics.

Cons:

- User must run `scan` manually.

Decision: watcher belongs to a later slice.

## Risks

- Stale graph counts may surprise users if summary does not show status breakdown.
- Migration for node status must be idempotent.
- Modified import stale/restore ordering can accidentally leave current imports stale.
- Removed file logic must not mark repository/workspace containment stale.
- Smoke script should not mutate checked-in fixtures.

## Open Questions

### Should `graph summary` show status breakdown?

Not required by the proposal, but useful. Optional if cheap:

```text
by_status:
  active: 20
  stale: 2
  unverified: 1
```

Recommendation: add only if implementation is straightforward; otherwise defer.
