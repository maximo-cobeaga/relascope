# Design: Relascope 0.0.3-b Real Workspace Dogfooding

## Overview

Relascope 0.0.3-b improves operational usability for internal dogfooding on real local workspaces. The engine already supports inventory, graph materialization, shallow imports, evidence, incremental diffing, and stale marking. This slice adds the minimum CLI ergonomics and machine-readable output needed to use those capabilities without opening SQLite.

This change should stay focused on command UX and health reporting. It should not add deeper analysis.

## Architecture Slice

```text
CLI
  ├─ repo list
  ├─ repo remove <id>
  ├─ doctor
  ├─ scan --format human|json
  └─ graph
       ├─ summary --format human|json
       └─ imports --format human|json

Storage
  ├─ repository list/remove
  ├─ doctor read models
  ├─ graph status counts
  └─ existing scan/graph/import read models
```

## Command Format Handling

Add a small enum in CLI:

```rust
#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}
```

Use `#[arg(long, value_enum, default_value_t = OutputFormat::Human)]` on supported commands.

Supported for 0.0.3-b:

- `scan`
- `graph summary`
- `graph imports`

Do not add global `--format` yet. Keeping it command-local avoids needing to define behavior for every command prematurely.

## JSON Output Strategy

Use `serde::Serialize` response structs from CLI or storage read models.

JSON is experimental. Shape should be stable enough for dogfooding scripts, but docs must say it is not a public API contract yet.

### Scan JSON

Suggested shape:

```json
{
  "scan_id": "...",
  "status": "completed",
  "files": 12,
  "added": 0,
  "modified": 1,
  "removed": 0,
  "unchanged": 11,
  "unavailable_repositories": []
}
```

### Graph summary JSON

Use existing `GraphSummary` shape if acceptable:

```json
{
  "total_nodes": 21,
  "nodes_by_kind": { "File": 12, "Module": 5 },
  "total_edges": 22,
  "edges_by_relation": { "CONTAINS": 19, "IMPORTS": 3 }
}
```

### Graph imports JSON

Use existing/import read model shape:

```json
[
  {
    "source": "api/app.py",
    "target": "api:os",
    "status": "unverified",
    "file_path": "app.py",
    "line": 1,
    "excerpt": "import os"
  }
]
```

## Repository List

Storage already supports `list_repositories(workspace_id)`. CLI can reuse it.

Human output:

```text
Repositories
  - api
      internal_id: ...
      path: ...
      availability: available
```

If empty:

```text
Repositories
  no repositories registered
```

No JSON requirement in this slice unless cheap. The spec only requires listing; avoid overexpanding.

## Repository Remove

Add storage method:

```rust
remove_repository_by_visible_id(workspace_id, visible_id) -> bool
```

Behavior:

- delete from `repositories` table only;
- do not delete repository files from disk;
- do not scan;
- return false if no row was removed.

Graph facts associated with removed repositories:

- For 0.0.3-b, do not implement deep graph cleanup.
- Optionally mark repository node stale if cheap, but avoid expanding scope.
- Document that removing registration stops future scans from scanning that repo; historical graph facts may remain until a later cleanup/invalidation feature.

This is consistent with the product's local-first, evidence-preserving model.

## Doctor Command

`relascope doctor` should be read-only.

Recommended read model:

```rust
pub struct DoctorReport {
    pub workspace_id: String,
    pub workspace_name: String,
    pub config_ok: bool,
    pub database_ok: bool,
    pub repositories_total: u64,
    pub repositories_available: u64,
    pub repositories_unavailable: u64,
    pub last_scan_status: Option<String>,
    pub stale_nodes: u64,
    pub stale_edges: u64,
    pub unverified_imports: u64,
    pub exclusions: Vec<String>
}
```

Storage helpers:

- count repository availability from stored state and/or path existence;
- last scan from existing status method;
- count nodes by status;
- count edges by status;
- count `IMPORTS` edges with status `unverified`.

Important design choice:

- Doctor may check whether repository paths exist.
- Doctor must not update repository availability in the database.
- Doctor must not walk files recursively.

Human output:

```text
Relascope doctor
  config: ok
  database: ok
  repositories: 3 total, 3 available, 0 unavailable
  last_scan: completed
  graph:
    stale_nodes: 0
    stale_edges: 0
    unverified_imports: 1
  exclusions: 7 configured
```

Exit code:

- `0` for command success even if warnings exist.
- Future versions can add thresholds/strict mode.

## Storage Read Models

Add or reuse methods:

```rust
remove_repository_by_visible_id(...)
graph_status_counts(...)
doctor_report(...)
```

Keep storage read models small. Do not introduce a service layer unless necessary.

## CLI Structure

Current command shape:

```text
repo add
graph summary
graph imports
```

Extend:

```text
repo list
repo remove <id>
doctor
```

`scan` currently has no args; convert it to args struct:

```rust
Scan(ScanArgs)
```

`graph summary` and `graph imports` become arg-carrying subcommands:

```rust
Summary(FormatArgs)
Imports(FormatArgs)
```

## Tests

CLI integration tests:

- `repo list` empty.
- `repo list` after adding repos.
- `repo remove <id>` removes registration but does not delete directory.
- `repo remove missing` fails clearly.
- `scan --format json` outputs valid JSON and expected counts.
- `graph summary --format json` outputs valid JSON.
- `graph imports --format json` outputs valid JSON with evidence.
- `doctor` reports config/database/repository/graph health.
- `doctor` does not update unavailable repository state.

Storage tests:

- repository removal by visible ID.
- graph status counts.
- doctor report read model if implemented in storage.

Smoke script:

- Optionally call `doctor` after scan.
- Optionally call JSON output paths to validate parseable JSON.

## Documentation

Add:

- `tutorial/dogfooding-real-workspace.md`

Guide should explain:

1. create a Relascope workspace outside source repos;
2. add local repos;
3. run `repo list`;
4. run `scan`;
5. run `doctor`;
6. inspect `graph summary` and `graph imports`;
7. save JSON output to files;
8. record findings:
   - false positives;
   - false negatives;
   - unresolved imports;
   - stale facts;
   - slow scans;
   - confusing output.

## Tradeoffs

### Command-local format option

Pros:

- Keeps behavior explicit.
- Avoids global CLI complexity.

Cons:

- Repeated flag definitions.

Decision: command-local for now.

### JSON experimental

Pros:

- Enables scripts and evidence capture immediately.
- Avoids overcommitting to schema before product shape stabilizes.

Cons:

- Users need warning that schema may change.

Decision: document clearly as experimental.

### Doctor checks path existence without database update

Pros:

- Read-only command remains safe.
- No surprising state changes.

Cons:

- Doctor may report path missing while stored availability still says available until scan.

Decision: acceptable and explicit.

## Risks

- JSON output could accidentally become treated as stable API.
- `repo remove` could be misunderstood as deleting files; messaging must be explicit.
- Doctor could grow into a deep analyzer; keep it read-only and shallow.
- Too much output could reduce usability; keep human output concise.
