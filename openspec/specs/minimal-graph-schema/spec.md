# Specification: Minimal Graph Schema

## Requirements

### Requirement: Graph persistence schema

Relascope SHALL persist a minimal graph model alongside the existing inventory tables.

#### Scenario: Workspace database includes graph tables

- **Given** a Relascope workspace is initialized or opened after this change
- **When** migrations run
- **Then** the local SQLite database includes tables for graph nodes, graph edges, and graph evidence
- **And** existing 0.0.1 inventory tables remain available
- **And** existing workspace, repository, scan, and file inventory behavior continues to work

#### Scenario: Graph schema does not replace file inventory

- **Given** a workspace has file inventory rows from `relascope scan`
- **When** graph rows are materialized
- **Then** the `files` inventory table remains the inventory source for 0.0.x
- **And** graph rows are persisted alongside inventory rows

### Requirement: Minimal graph materialization during scan

Relascope SHALL materialize a minimal graph during `relascope scan`.

#### Scenario: Scan creates workspace node

- **Given** an initialized workspace
- **When** the user runs `relascope scan`
- **Then** Relascope persists a `Workspace` graph node for the current workspace
- **And** the node identity is stable for the workspace UUID

#### Scenario: Scan creates repository nodes

- **Given** an initialized workspace with registered repositories
- **When** the user runs `relascope scan`
- **Then** Relascope persists one `Repository` graph node per registered repository
- **And** each repository node identity is stable for the repository UUID
- **And** unavailable repositories remain represented as repository nodes

#### Scenario: Scan creates file nodes for inventoried files

- **Given** a registered repository contains non-excluded files
- **When** the user runs `relascope scan`
- **Then** Relascope persists one `File` graph node per inventoried file
- **And** each file node identity is based on workspace ID, repository ID, and relative path
- **And** unknown files are represented as `File` nodes rather than dropped

#### Scenario: Scan creates containment edges

- **Given** a workspace with registered repositories and inventoried files
- **When** the user runs `relascope scan`
- **Then** Relascope persists `CONTAINS` edges from the workspace node to repository nodes
- **And** persists `CONTAINS` edges from repository nodes to their file nodes

#### Scenario: Re-running scan refreshes minimal graph deterministically

- **Given** a workspace has already been scanned
- **When** the user runs `relascope scan` again without changing registered repositories or files
- **Then** Relascope does not create duplicate graph nodes for the same workspace, repositories, or files
- **And** does not create duplicate `CONTAINS` edges for the same source, target, and relation type

### Requirement: Graph evidence for materialized facts

Relascope SHALL persist basic evidence for graph facts created from inventory.

#### Scenario: File node evidence points to inventory source

- **Given** `relascope scan` creates a `File` node
- **When** the graph fact is persisted
- **Then** Relascope stores evidence with repository ID and relative file path
- **And** associates that evidence with the file node or its repository containment edge

#### Scenario: Repository containment evidence is explicit

- **Given** `relascope scan` creates a `CONTAINS` edge from workspace to repository
- **When** the edge is persisted
- **Then** Relascope stores or can derive evidence from the repository registration record

### Requirement: Graph summary command

Relascope SHALL provide a minimal graph inspection command.

#### Scenario: Graph summary after scan

- **Given** a workspace has been scanned after graph materialization is available
- **When** the user runs `relascope graph summary`
- **Then** Relascope reads persisted graph state
- **And** reports total node count
- **And** reports node counts by kind
- **And** reports total edge count
- **And** reports edge counts by relation type

#### Scenario: Graph summary before scan

- **Given** an initialized workspace has no materialized graph yet
- **When** the user runs `relascope graph summary`
- **Then** Relascope reports that no graph has been materialized yet
- **And** exits successfully

#### Scenario: Graph summary does not scan

- **Given** a workspace has a persisted graph
- **And** a registered repository path changes after the last scan
- **When** the user runs `relascope graph summary`
- **Then** Relascope reports persisted graph state only
- **And** does not walk repository files
- **And** does not update repository availability

### Requirement: Preserve 0.0.1 behavior

Relascope SHALL preserve all verified 0.0.1 inventory behavior while adding the minimal graph.

#### Scenario: Existing inventory commands still pass

- **Given** a user follows the 0.0.1 workflow
- **When** they run `init`, `repo add`, `scan`, and `status`
- **Then** the commands continue to work as before
- **And** `status` remains based on persisted state without scanning

#### Scenario: Missing repository is still not deleted

- **Given** a repository was previously registered
- **And** its stored path no longer exists
- **When** the user runs `relascope scan`
- **Then** the repository is marked unavailable
- **And** the repository record remains persisted
- **And** the repository graph node remains represented

### Requirement: Scope boundaries

Relascope SHALL not perform semantic analysis as part of 0.0.2-a.

#### Scenario: Scan does not create semantic nodes

- **Given** a repository contains Python or TypeScript source files
- **When** the user runs `relascope scan`
- **Then** Relascope may classify files by shallow inventory kind and language
- **But** does not create `Module`, `Function`, `Class`, `Interface`, or symbol nodes
- **And** does not create `IMPORTS` edges

#### Scenario: Graph summary remains local and deterministic

- **Given** a workspace has a materialized minimal graph
- **When** the user runs `relascope graph summary`
- **Then** Relascope does not use AI
- **And** does not require network access
- **And** does not execute code from registered repositories
