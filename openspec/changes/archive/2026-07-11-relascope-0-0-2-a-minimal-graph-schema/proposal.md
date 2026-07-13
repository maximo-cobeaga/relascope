# Change Proposal: Relascope 0.0.2-a Minimal Graph Schema

## Status

Draft for approval before specification, design, tasks, and implementation.

## Problem Statement

Relascope 0.0.1 can initialize a workspace, register local repositories, scan files, and persist inventory. The next product step is to introduce the smallest useful knowledge graph foundation without jumping into semantic code analysis.

The product vision depends on graph-backed architecture knowledge, but adding imports, modules, impact analysis, or UI before a durable graph schema would create fragile abstractions. Relascope needs a minimal graph layer that can represent the workspace, repositories, files, containment relationships, and evidence derived from the existing scan.

## Goals

- Add graph persistence tables for nodes, edges, and evidence.
- Keep the existing `files` inventory table as the inventory source for 0.0.x.
- Extend `relascope scan` so it also materializes a minimal graph.
- Create graph nodes for:
  - `Workspace`
  - `Repository`
  - `File`
- Create graph edges for:
  - `CONTAINS`
- Attach basic evidence for file-derived graph facts.
- Use deterministic graph IDs for the same workspace/repository/file facts where practical.
- Use `repository_id + relative_path` as the initial file node identity.
- Add a minimal inspection command:
  - `relascope graph summary`
- Report graph node and edge counts by kind/type.
- Preserve all 0.0.1 behavior and tests.

## Non-Goals

- No Python or TypeScript import parsing yet.
- No `Module`, `Function`, `Class`, `Interface`, or symbol nodes yet.
- No cross-repository dependency resolution.
- No impact analysis.
- No graph visualization.
- No web UI or desktop UI.
- No AI enrichment.
- No MCP integration.
- No migration away from the `files` inventory table.
- No stable rename tracking yet.

## User-Facing Commands

Existing commands remain:

```bash
relascope init [path]
relascope repo add <path> [--id <id>]
relascope scan
relascope status
```

New command:

```bash
relascope graph summary
```

## Key Decisions

- `scan` remains the single operation that updates persisted inventory and the minimal graph.
- `files` remains the inventory record table; graph tables are added alongside it.
- Graph nodes and edges are materialized from persisted scan facts.
- File node identity for 0.0.2-a is based on `workspace_id`, `repository_id`, and `relative_path`.
- Evidence is basic in this slice and points to repository/file path facts, not semantic source excerpts.
- `relascope graph summary` is the first graph inspection surface.

## Acceptance Summary

Relascope 0.0.2-a is acceptable when a user can run the existing 0.0.1 workflow, execute `relascope scan`, then run `relascope graph summary` and see persisted counts for workspace, repository, and file nodes plus `CONTAINS` edges. Re-running status or graph summary from a new process must read persisted graph state without requiring a new scan.

## Review Workload Note

This change should be smaller than the full 0.0.2 milestone because it intentionally excludes import parsing and semantic graph expansion. If implementation forecast grows, split storage/schema first and CLI summary second.
