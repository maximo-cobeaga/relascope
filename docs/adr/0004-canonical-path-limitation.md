# ADR 0004: Canonical Path Persistence Limitation

## Status

Accepted for Relascope 0.0.1.

## Context

Relascope should eventually support portable workspaces, relocated repositories, shared team state, and source metadata beyond local absolute paths. Designing that fully in 0.0.1 would add complexity before the inventory proof is validated.

## Decision

For 0.0.1, canonicalize repository paths for validation, scanning, and persistence. Store the path with `path_kind = "canonical-local"` and keep repository identity independent from the path through an internal UUID.

The workspace ID, repository UUIDs, visible IDs, and path metadata are persisted so later versions can migrate toward portable path anchors or source descriptors.

## Consequences

A 0.0.1 workspace is not guaranteed to remain valid if the workspace or repositories are relocated. Missing registered paths are reported as unavailable and are not deleted silently. This limitation is acceptable for the first local inventory milestone and must remain documented until portability is improved.
