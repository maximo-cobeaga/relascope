# ADR 0007: Stale Graph Marking Instead of Deletion

## Status

Accepted for Relascope 0.0.3-a.

## Context

Relascope now stores inventory, graph containment, module nodes, and shallow import relationships. Repeated scans can update current facts, but removed or modified files can leave older graph facts behind. Deleting those facts immediately would lose historical evidence and make future audit/debug flows harder.

## Decision

Use scan-to-scan comparison to detect added, modified, unchanged, and removed files. Mark obsolete graph facts as `stale` rather than deleting them.

For 0.0.3-a:

- removed file nodes become `stale`;
- removed file module nodes become `stale`;
- containment edges touching removed file/module facts become `stale`;
- import edges touching removed module facts become `stale`;
- modified code files mark previous outgoing `IMPORTS` edges as `stale` before current imports are materialized;
- current imports can restore an edge to `active` or `unverified` if the import is still present.

Add `nodes.status` as a first-class column so node lifecycle state is queryable without parsing JSON metadata.

## Consequences

Relascope preserves evidence and graph history while avoiding silently active obsolete relationships. Graph counts may include stale facts, so future views should become status-aware. Rename detection remains out of scope and is represented as removed plus added in this milestone.
