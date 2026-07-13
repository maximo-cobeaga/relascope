# Design: Relascope 0.0.2-a Minimal Graph Schema

## Overview

Relascope 0.0.2-a adds the first graph-backed representation of a workspace. The graph is deliberately minimal and deterministic: workspace, repositories, files, and `CONTAINS` relationships only.

This slice should not parse imports, create modules, infer dependencies, call AI, or expose a visual graph. It establishes the persistence and command surface needed for later graph expansion.

## Current Foundation

Relascope 0.0.1 already provides:

- `relascope init [path]`
- `relascope repo add <path> [--id <id>]`
- `relascope scan`
- `relascope status`
- SQLite database at `.relascope/graph.db`
- inventory tables: `workspaces`, `repositories`, `scans`, `files`
- shallow file classification
- tests and a polyrepo fixture

0.0.2-a builds on this without replacing the inventory model.

## Architecture Slice

```text
relascope scan
  ├─ scan repositories
  ├─ persist file inventory
  ├─ update repository availability
  └─ materialize minimal graph
       ├─ Workspace node
       ├─ Repository nodes
       ├─ File nodes
       ├─ Workspace CONTAINS Repository edges
       ├─ Repository CONTAINS File edges
       └─ Basic evidence

relascope graph summary
  └─ read persisted graph counts
```

## Data Model

Add graph tables to SQLite.

### `nodes`

```sql
nodes(
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  repository_id TEXT,
  kind TEXT NOT NULL,
  qualified_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  fingerprint TEXT,
  metadata_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### `edges`

```sql
edges(
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  source_node_id TEXT NOT NULL,
  target_node_id TEXT NOT NULL,
  relation_type TEXT NOT NULL,
  origin_type TEXT NOT NULL,
  origin_id TEXT,
  confidence REAL NOT NULL,
  status TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  UNIQUE(workspace_id, source_node_id, target_node_id, relation_type)
);
```

### `evidence`

```sql
evidence(
  id TEXT PRIMARY KEY,
  edge_id TEXT,
  node_id TEXT,
  repository_id TEXT,
  file_path TEXT,
  start_line INTEGER,
  end_line INTEGER,
  content_hash TEXT,
  excerpt TEXT,
  metadata_json TEXT NOT NULL
);
```

This matches the master architecture closely enough for future evolution.

## Graph Identity

Use deterministic IDs instead of random UUIDs for graph facts where possible.

Recommended format:

```text
node:workspace:<workspace_id>
node:repository:<workspace_id>:<repository_id>
node:file:<workspace_id>:<repository_id>:<relative_path_hash>
edge:contains:<source_node_id_hash>:<target_node_id_hash>
```

Implementation can use SHA-256/hex helpers to keep IDs short and safe for arbitrary paths.

### Why deterministic IDs

- Re-running `scan` should not duplicate graph facts.
- Later scans should update timestamps/metadata, not create parallel nodes.
- Tests can assert stable counts.

## Node Mapping

| Source | Node kind | qualified_name | display_name | repository_id |
|---|---|---|---|---|
| workspace config | `Workspace` | workspace UUID | workspace name | null |
| repository record | `Repository` | repository visible ID or UUID | visible ID | repository UUID |
| file inventory row | `File` | `<repo_visible_id>/<relative_path>` | file name or relative path | repository UUID |

File node identity is based on:

```text
workspace_id + repository_id + relative_path
```

This is intentionally not rename-stable yet. Rename stability belongs to later incremental/identity work.

## Edge Mapping

| Source | Edge |
|---|---|
| workspace → repository | `Workspace CONTAINS Repository` |
| repository → file | `Repository CONTAINS File` |

Edge defaults:

```text
origin_type: deterministic
origin_id: scan_id or repository registration ID when available
confidence: 1.0
status: active
metadata_json: {}
```

Unavailable repositories still get repository nodes and workspace containment edges. They simply have no file nodes from the current scan if their files cannot be inventoried.

## Evidence Mapping

Evidence in 0.0.2-a is basic and factual.

### Workspace contains repository

Evidence can be associated with the edge and derived from repository registration:

```json
{
  "source": "repository_registration",
  "repository_id": "...",
  "path": "..."
}
```

### Repository contains file

Evidence can be associated with the file node or edge:

```text
repository_id: <repo uuid>
file_path: <relative path>
content_hash: <inventory content hash if available>
metadata_json: { "source": "file_inventory", "scan_id": "..." }
```

No source excerpts or line ranges are required yet because this slice does not parse file contents semantically.

## Scan Integration

Current scan flow:

1. Load repositories.
2. Create scan record.
3. Walk files.
4. Update availability.
5. Persist file rows.
6. Finish scan.

New flow:

1. Load repositories.
2. Create scan record.
3. Walk files.
4. Update availability.
5. Persist file rows.
6. Materialize minimal graph from repositories and current scan files.
7. Finish scan.

Materialization should happen in storage or a narrow graph service layer. For this slice, adding graph persistence methods to `relascope-storage-sqlite` is acceptable to avoid premature crate splitting.

## Duplicate Handling

Use `INSERT ... ON CONFLICT DO UPDATE` for nodes and edges.

Node update should refresh:

- display name
- qualified name
- metadata
- fingerprint
- updated_at

Edge update should refresh:

- origin fields
- confidence/status if needed
- metadata
- last_seen_at

Evidence can be inserted per scan. If evidence grows too quickly later, add uniqueness rules or evidence compaction in a future slice.

## `relascope graph summary`

Add a nested CLI command:

```bash
relascope graph summary
```

Suggested output:

```text
Graph summary
  nodes: 14
  by_kind:
    Workspace: 1
    Repository: 3
    File: 10
  edges: 13
  by_relation:
    CONTAINS: 13
```

Before any scan/materialized graph:

```text
Graph summary
  no graph has been materialized yet
```

The command must read persisted graph state only. It must not scan repositories or update availability.

## Testing Strategy

Add/extend tests for:

- migrations create graph tables;
- graph materialization creates expected node/edge counts;
- repeated scan does not duplicate graph nodes/edges;
- unavailable repositories remain represented as repository nodes;
- `graph summary` before scan exits successfully;
- `graph summary` after scan reports expected counts;
- existing 0.0.1 CLI tests continue to pass.

Expected fixture counts after full `polyrepo-basic` scan:

```text
Workspace nodes: 1
Repository nodes: 3
File nodes: 10
Total nodes: 14
CONTAINS edges: 13
```

If a repository is unavailable during scan:

- Repository node remains.
- Workspace → repository edge remains.
- File nodes for unavailable repository are not created from that scan.
- Existing stale file nodes from previous scans may remain unless invalidation is explicitly implemented later.

For 0.0.2-a, stale node invalidation is out of scope. The graph represents accumulated known facts refreshed by scans, not a fully pruned current-state graph.

## Tradeoffs

### Keep `files` as inventory source

Pros:

- Preserves 0.0.1 behavior.
- Avoids migrating status and scan logic immediately.
- Keeps graph materialization explicit and testable.

Cons:

- File facts exist in both `files` and `nodes`.
- Future code must avoid confusion about source of truth.

Decision: acceptable for 0.0.x. `files` remains inventory; `nodes/edges/evidence` represent graph knowledge.

### Materialize graph during `scan`

Pros:

- One user action updates inventory and graph.
- No extra command needed for the happy path.
- Easier onboarding.

Cons:

- `scan` now does more work.
- Future users may want separate inventory and graph phases.

Decision: acceptable for minimal graph. A separate graph rebuild command can be added later if needed.

### Deterministic graph IDs

Pros:

- Prevents duplicates.
- Simplifies tests.
- Makes re-scan behavior predictable.

Cons:

- Requires careful ID helpers.
- Changing ID format later needs migration.

Decision: use deterministic IDs based on stable domain identifiers.

### No stale graph pruning yet

Pros:

- Keeps this slice small.
- Avoids designing invalidation before incremental graph work.

Cons:

- Removed files may leave stale graph nodes after later scans.

Decision: document as limitation. Full invalidation belongs to incremental graph work.

## Risks

- Accidentally expanding into import parsing.
- Treating graph as current-state perfect truth before invalidation exists.
- Duplicating graph facts on repeated scans.
- Making `graph summary` scan implicitly.
- Over-coupling graph storage to the current inventory schema.

## Open Questions

None blocking for 0.0.2-a. Stale graph invalidation and semantic nodes are intentionally deferred.
