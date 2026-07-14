# Relascope — Current State and Next Steps

This document summarizes the current Relascope state after closing the early local-first milestones through the scan progress visibility slice.

## Current state

Relascope has a functional local-first CLI engine for registering local repositories, scanning them into SQLite, materializing an evidence-backed graph, detecting shallow imports, and preserving stale graph facts across incremental scans.

Closed milestones:

| Milestone | Status | Result |
|---|---:|---|
| `0.0.1 — Inventory` | Closed | Workspaces, local repos, scan, status, SQLite |
| `0.0.2-a — Minimal Graph Schema` | Closed | `Workspace`, `Repository`, `File`, `CONTAINS`, `graph summary` |
| `0.0.2-b — Shallow Module and Import Detection` | Closed | `Module`, `IMPORTS`, line evidence, `graph imports` |
| `0.0.3-a — Incremental Scan State and Stale Graph Marking` | Closed | Added/modified/removed counts and stale graph facts |
| `0.0.3-b — Real Workspace Dogfooding` | Closed | `repo list`, `repo remove`, `doctor`, experimental JSON, dogfooding docs |
| `0.0.3-c — Scan Progress Visibility` | Closed | Progress visibility for long scans, post-scan materialization progress, `--silent`, clean JSON, and handled cancellation state |

## What Relascope can do today

### 1. Create a workspace

```powershell
relascope init [path]
```

Creates:

```text
relascope.yaml
.relascope/graph.db
```

### 2. Register local repositories

```powershell
relascope repo add <path> --id <id>
```

Each repo has:

- stable internal UUID;
- visible unique ID;
- canonical local path;
- availability state.

### 3. List repositories

```powershell
relascope repo list
```

### 4. Scan repositories

```powershell
relascope scan
```

Relascope shows progress for large scans and post-scan materialization:

```text
Scanning repositories...
  backend: 500 files indexed, 120 skipped, elapsed 00:00:03
Persisting inventory...
Marking stale graph facts...
Materializing graph...
  graph: 500/18605 files, 4/4 repositories, elapsed 00:00:03
Materializing shallow imports...
  imports backend: 250/5861 code files, 1200 imports, elapsed 00:00:30
```

Use `--silent` for concise output:

```powershell
relascope scan --silent
```

The scan:

- inventories non-excluded files;
- classifies files shallowly;
- persists rows in SQLite;
- creates graph nodes and edges;
- creates modules for recognized code files;
- detects shallow Python/TypeScript/JavaScript imports;
- stores line evidence;
- reports added, modified, removed, and unchanged files;
- marks obsolete facts as `stale`.

### 5. Inspect persisted state

```powershell
relascope status
relascope doctor
relascope graph summary
relascope graph imports
```

These commands are read-only and do not rescan repositories.

### 6. Export experimental JSON

```powershell
relascope scan --format json > scan.json
relascope graph summary --format json > graph-summary.json
relascope graph imports --format json > graph-imports.json
```

JSON output is useful for dogfooding evidence, but it is not a stable public API yet.

## Verification commands

From the Relascope repo root:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

## Active specs

| Spec | Path |
|---|---|
| Workspace inventory | `openspec/specs/workspace-inventory/spec.md` |
| Minimal graph schema | `openspec/specs/minimal-graph-schema/spec.md` |
| Shallow module imports | `openspec/specs/shallow-module-imports/spec.md` |
| Incremental stale graph | `openspec/specs/incremental-stale-graph/spec.md` |
| Real workspace dogfooding | `openspec/specs/real-workspace-dogfooding/spec.md` |
| Scan progress visibility | `openspec/specs/scan-progress-visibility/spec.md` |

No active OpenSpec change is currently under development.

## Recommended next steps

1. Consider a focused progress-polish slice, for example `0.0.3-d — Compact Progress Output`, so large scans do not print hundreds of heartbeat lines by default.
2. Consider stale `running` scan cleanup for hard process kills or terminal closes.
3. Use dogfooding evidence to guide import filtering and resolution improvements; the ReservApp run produced 32,646 unverified imports.
4. Move toward the next parser foundation slice, likely `0.0.4 — Tree-sitter Parser Foundation`, before impact analysis.
5. Do not jump to impact analysis before improving parser confidence and validating real workspace behavior.

## Scope still intentionally excluded

- No watcher yet.
- No Tree-sitter yet.
- No cross-repository resolution yet.
- No impact analysis yet.
- No AI/model integration yet.
- No MCP yet.
- No web or desktop UI yet.
