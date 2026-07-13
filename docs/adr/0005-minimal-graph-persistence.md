# ADR 0005: Minimal Graph Persistence

## Status

Accepted for Relascope 0.0.2-a.

## Context

Relascope 0.0.1 stores a durable inventory of workspaces, repositories, scans, and files. The product vision requires a graph-backed architecture model, but semantic parsing, imports, impact analysis, and UI should not be introduced before the graph storage foundation exists.

## Decision

Add minimal graph tables to `.relascope/graph.db`:

- `nodes`
- `edges`
- `evidence`

Materialize the graph during `relascope scan` using existing inventory facts:

- `Workspace` nodes
- `Repository` nodes
- `File` nodes
- `CONTAINS` edges from workspace to repositories
- `CONTAINS` edges from repositories to files

Keep the existing `files` inventory table as the 0.0.x inventory source. Use deterministic graph IDs to avoid duplicate nodes and edges on repeated scans. Add `relascope graph summary` as the first graph inspection command.

## Consequences

The project now has a graph persistence foundation without semantic analysis. Future milestones can add modules, imports, symbols, dependency edges, impact analysis, and richer evidence on top of the same schema.

Stale graph pruning is intentionally out of scope for this slice. Removed files may leave historical graph nodes until incremental invalidation is designed.
