# ADR 0002: SQLite Local Persistence

## Status

Accepted for Relascope 0.0.1.

## Context

Relascope 0.0.1 must preserve workspace identity, repository identity, configuration metadata, scan records, and inventory between process runs. The product convention reserves `.relascope/graph.db` for the local knowledge store, even though 0.0.1 does not yet persist graph edges.

## Decision

Use SQLite at `.relascope/graph.db` with SQLx runtime queries. Store initial tables for:

- workspaces
- repositories
- scans
- files

Repository records live in SQLite as the 0.0.1 source of truth. `relascope.yaml` stores the user-visible workspace configuration and exclusions.

## Consequences

SQLite gives transactional local persistence without external services. Naming the file `graph.db` aligns with the long-term product convention while allowing inventory-only tables in the first milestone. Future versions can add node, edge, evidence, and snapshot tables through migrations.
