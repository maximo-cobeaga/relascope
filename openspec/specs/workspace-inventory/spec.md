# Specification: Workspace Inventory

## Requirements

### Requirement: Workspace initialization

Relascope SHALL initialize a local workspace in the current directory when the user runs `relascope init` without a path.

#### Scenario: Initialize in current directory

- **Given** the current directory is not already a Relascope workspace
- **When** the user runs `relascope init`
- **Then** Relascope creates `relascope.yaml`
- **And** Relascope creates `.relascope/`
- **And** Relascope creates `.relascope/graph.db`
- **And** the workspace receives a stable internal UUID
- **And** the command exits successfully

#### Scenario: Initialize at explicit path

- **Given** the target path exists or can be created
- **When** the user runs `relascope init <path>`
- **Then** Relascope initializes the workspace at `<path>`
- **And** all persisted workspace files are written under that target path

#### Scenario: Reject conflicting initialization

- **Given** the target path is already a Relascope workspace
- **When** the user runs `relascope init` for that path
- **Then** Relascope does not overwrite existing workspace identity
- **And** reports that the workspace already exists

### Requirement: Local repository registration

Relascope SHALL allow users to register existing local repository directories in a workspace.

#### Scenario: Add local repository with generated visible ID

- **Given** an initialized workspace
- **And** `../backend` exists as a local directory
- **When** the user runs `relascope repo add ../backend`
- **Then** Relascope stores a new repository record
- **And** the repository receives a stable internal UUID
- **And** the repository receives a visible ID slug generated from the folder name
- **And** the canonical absolute path is stored for 0.0.1 operation

#### Scenario: Add local repository with explicit visible ID

- **Given** an initialized workspace
- **And** `../backend` exists as a local directory
- **When** the user runs `relascope repo add ../backend --id api`
- **Then** Relascope stores the repository with visible ID `api`
- **And** stores a stable internal UUID distinct from the visible ID

#### Scenario: Reject duplicate visible repository ID

- **Given** an initialized workspace
- **And** a repository with visible ID `api` already exists
- **When** the user runs `relascope repo add ../other --id api`
- **Then** Relascope rejects the command
- **And** no second repository with visible ID `api` is persisted

#### Scenario: Reject non-local or missing repository path

- **Given** an initialized workspace
- **When** the user runs `relascope repo add https://example.com/repo.git`
- **Then** Relascope rejects the command because Git URLs are out of scope for 0.0.1

- **Given** an initialized workspace
- **When** the user runs `relascope repo add ../missing`
- **Then** Relascope rejects the command because the local path does not exist

### Requirement: File inventory scan

Relascope SHALL scan registered repositories and persist an inventory of all non-excluded files without semantic analysis.

#### Scenario: Scan all non-excluded files

- **Given** an initialized workspace with at least one available repository
- **When** the user runs `relascope scan`
- **Then** Relascope walks registered repository directories
- **And** skips configured exclusions
- **And** persists one inventory record for each non-excluded file
- **And** records repository association, relative path, size, content hash when practical, modified timestamp when available, and shallow file classification

#### Scenario: Classify unknown files

- **Given** a non-excluded file whose language or kind is not recognized
- **When** `relascope scan` inventories the file
- **Then** the file is persisted with an `unknown` or equivalent classification
- **And** the file is not dropped solely because it is unrecognized

#### Scenario: Exclude default heavy and generated paths

- **Given** a registered repository containing `.git`, `node_modules`, `.venv`, `dist`, or `build`
- **When** the user runs `relascope scan`
- **Then** files under those excluded paths are not persisted in the inventory by default

#### Scenario: Scan does not execute repository code

- **Given** a registered repository containing scripts, package hooks, task definitions, or executable files
- **When** the user runs `relascope scan`
- **Then** Relascope does not execute code from the repository
- **And** does not require network access
- **And** does not require AI access

#### Scenario: Preserve missing repository record during scan

- **Given** a repository was previously registered
- **And** its stored path no longer exists
- **When** the user runs `relascope scan`
- **Then** Relascope marks the repository as unavailable
- **And** does not delete the repository silently
- **And** continues scanning other available repositories when possible

### Requirement: Persistent status

Relascope SHALL report persisted workspace state without performing a new scan.

#### Scenario: Status after scan

- **Given** a workspace has registered repositories
- **And** a successful scan has persisted inventory
- **When** the user runs `relascope status`
- **Then** Relascope reports the workspace ID
- **And** reports registered repositories with visible IDs, internal IDs, paths, and availability
- **And** reports the last scan summary from persisted state
- **And** does not perform a new file scan

#### Scenario: Reopen workspace from a new process

- **Given** a workspace was initialized, repositories were added, and a scan completed
- **When** a new process runs `relascope status` in that workspace
- **Then** the same workspace UUID is reported
- **And** the same repository UUIDs and visible IDs are reported
- **And** the persisted configuration is loaded
- **And** the last scan inventory summary remains available

### Requirement: Configurable exclusions

Relascope SHALL provide default scan exclusions and allow users to customize exclusions in workspace configuration.

#### Scenario: Default exclusions exist after init

- **Given** a newly initialized workspace
- **When** the user opens `relascope.yaml`
- **Then** the configuration includes or references default exclusions for common VCS, dependency, virtual environment, build, and generated directories

#### Scenario: User-defined exclusion affects scan

- **Given** an initialized workspace
- **And** the user configures an additional exclusion pattern
- **When** the user runs `relascope scan`
- **Then** files matching the additional exclusion are not persisted in the latest inventory

### Requirement: 0.0.1 portability limitation is documented

Relascope SHALL document that 0.0.1 stores canonicalized paths for operation and is not yet fully portable across machines or relocated directory layouts.

#### Scenario: Workspace documentation describes path limitation

- **Given** the 0.0.1 implementation is complete
- **When** a user reads the relevant project documentation or ADR
- **Then** it states that paths are canonicalized and persisted for 0.0.1
- **And** it states that repository UUIDs, visible IDs, and enough path metadata are retained to evolve portability later
