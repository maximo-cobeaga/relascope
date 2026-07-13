# Apply Progress: Relascope 0.0.2-a Minimal Graph Schema

## Status

Implementation applied and verified by user-side PowerShell execution.

## Implemented

- Added core graph domain module:
  - `GraphNode`
  - `GraphEdge`
  - `GraphEvidence`
  - `GraphSummary`
  - deterministic ID helpers for workspace, repository, file, edge, and evidence facts
- Added graph constants:
  - `Workspace`
  - `Repository`
  - `File`
  - `CONTAINS`
- Added SQLite graph tables:
  - `nodes`
  - `edges`
  - `evidence`
- Added graph indexes for summary queries.
- Added graph persistence methods:
  - node upsert
  - edge upsert
  - evidence upsert
  - graph summary query
- Added minimal graph materialization during `relascope scan`:
  - one workspace node
  - one repository node per registered repository
  - one file node per current scan file
  - workspace-to-repository `CONTAINS` edges
  - repository-to-file `CONTAINS` edges
  - basic repository and file evidence
- Added CLI command:
  - `relascope graph summary`
- Extended CLI tests for graph summary before scan, graph counts after scan, and repeated scan idempotency.
- Added ADR:
  - `docs/adr/0005-minimal-graph-persistence.md`
- Added tutorial:
  - `tutorial/onboarding-relascope-0-0-2-a.md`
- Updated README with graph summary and tutorial link.

## Scope Preserved

Still intentionally out of scope:

- import parsing
- module nodes
- symbol nodes
- semantic edges beyond `CONTAINS`
- impact analysis
- AI
- MCP
- web UI
- desktop UI
- stale graph pruning

## Verification Evidence

User ran from PowerShell:

```powershell
cargo fmt
cargo test
```

Result: OK, according to user report.

Manual smoke test verified full fixture graph counts:

```text
nodes: 14
File: 10
Repository: 3
Workspace: 1
edges: 13
CONTAINS: 13
```

Repeated scan preserved the same counts, confirming graph node/edge idempotency for this fixture.

See `verify-report.md` for detailed evidence.

## Remaining Verification

None for the approved 0.0.2-a minimal graph scope.

## Risks

- The Pi harness shell still cannot execute Cargo directly, so command execution evidence is user-provided.
- Stale graph pruning remains intentionally deferred.
