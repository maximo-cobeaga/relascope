# Relascope — Current State and Next Steps

This document summarizes the current Relascope state after closing the early local-first milestones and opening the current dogfooding polish slice.

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

Active slice:

| Milestone | Status | Goal |
|---|---:|---|
| `0.0.3-c — Scan Progress Visibility` | Active OpenSpec change | Show scan progress on large repositories and mark interrupted scans as `cancelled` |

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

The current active slice adds progress visibility for large scans:

```text
Scanning repositories...
  backend: 500 files indexed, 120 skipped, elapsed 00:00:03
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

Active change under development:

```text
openspec/changes/relascope-0-0-3-c-scan-progress-visibility/
```

## Recommended next steps

1. Complete the pending manual dogfooding checks for `0.0.3-c`:
   - rebuild the CLI;
   - run `relascope scan` on the ReservApp dogfood workspace;
   - confirm progress appears during large scans;
   - test Ctrl+C while scan is active;
   - confirm `doctor` / `status` show `cancelled`;
   - re-run scan and confirm it can complete afterward;
   - confirm `scan --format json > scan.json` stays JSON-only.
2. Capture evidence: scan duration, expensive repos, unresolved imports, stale behavior, false positives, false negatives, and UX friction.
3. If dogfooding passes, create the verify report, sync the active spec, and archive `relascope-0-0-3-c-scan-progress-visibility`.
4. If dogfooding is usable after archive, move to the next parser foundation slice, likely `0.0.4 — Tree-sitter Parser Foundation`.
5. Do not jump to impact analysis before improving parser confidence and validating real workspace behavior.

Detailed handoff for tomorrow:

```text
openspec/changes/relascope-0-0-3-c-scan-progress-visibility/handoff.md
```

## Scope still intentionally excluded

- No watcher yet.
- No Tree-sitter yet.
- No cross-repository resolution yet.
- No impact analysis yet.
- No AI/model integration yet.
- No MCP yet.
- No web or desktop UI yet.
