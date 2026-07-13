# Sync Report: Relascope 0.0.2-a Minimal Graph Schema

## Status

Synced.

## Synced Specification

The approved and verified minimal graph schema specification was promoted from the change delta into the active OpenSpec specification tree.

- Source: `openspec/changes/relascope-0-0-2-a-minimal-graph-schema/specs/minimal-graph-schema/spec.md`
- Target: `openspec/specs/minimal-graph-schema/spec.md`

## Verification Basis

This sync is based on the verified implementation recorded in:

- `openspec/changes/relascope-0-0-2-a-minimal-graph-schema/apply-progress.md`
- `openspec/changes/relascope-0-0-2-a-minimal-graph-schema/verify-report.md`

Verification evidence:

- `cargo fmt`: passed according to user-side PowerShell execution.
- `cargo test`: passed according to user-side PowerShell execution.
- Manual graph smoke test passed.
- Full fixture graph summary produced:
  - `Workspace`: 1
  - `Repository`: 3
  - `File`: 10
  - total nodes: 14
  - `CONTAINS`: 13
  - total edges: 13
- Repeated scan preserved node and edge counts.

## Notes

Relascope 0.0.2-a remains limited to the minimal graph foundation. Import parsing, module nodes, symbol nodes, impact analysis, AI, MCP, web UI, desktop UI, and stale graph pruning remain out of scope.
