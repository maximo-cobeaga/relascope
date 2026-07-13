# Specification Delta: Real Workspace Dogfooding

## ADDED Requirements

### Requirement: Repository listing

Relascope SHALL list repositories registered in the current workspace.

#### Scenario: List repositories in workspace

- **Given** an initialized workspace has registered repositories
- **When** the user runs `relascope repo list`
- **Then** Relascope prints each repository visible ID
- **And** prints its internal ID
- **And** prints its path
- **And** prints its availability
- **And** does not scan repository files

#### Scenario: List repositories when none are registered

- **Given** an initialized workspace has no registered repositories
- **When** the user runs `relascope repo list`
- **Then** Relascope exits successfully
- **And** reports that no repositories are registered

### Requirement: Repository removal

Relascope SHALL remove repository registrations without deleting repository files from disk.

#### Scenario: Remove repository by visible ID

- **Given** an initialized workspace has a repository with visible ID `api`
- **When** the user runs `relascope repo remove api`
- **Then** Relascope removes the repository registration from active repository listings
- **And** does not delete the repository directory from disk
- **And** exits successfully

#### Scenario: Remove missing repository ID

- **Given** an initialized workspace has no repository with visible ID `missing`
- **When** the user runs `relascope repo remove missing`
- **Then** Relascope returns a clear error
- **And** leaves existing repository registrations unchanged

#### Scenario: Removed repository is not scanned in future scans

- **Given** a repository registration was removed
- **When** the user runs `relascope scan`
- **Then** Relascope does not scan the removed repository path

### Requirement: Workspace doctor

Relascope SHALL provide a workspace health command.

#### Scenario: Doctor reports healthy workspace

- **Given** an initialized workspace has a readable config and database
- **And** registered repositories are available
- **When** the user runs `relascope doctor`
- **Then** Relascope reports workspace config status
- **And** database status
- **And** repository availability summary
- **And** last scan status or no-scan-yet state
- **And** graph health signals
- **And** exits successfully

#### Scenario: Doctor reports unavailable repositories

- **Given** a registered repository path no longer exists
- **When** the user runs `relascope doctor`
- **Then** Relascope reports that repository as unavailable or missing
- **And** does not remove it automatically
- **And** does not scan repository files

#### Scenario: Doctor reports stale and unverified graph signals

- **Given** a workspace has graph facts
- **When** the user runs `relascope doctor`
- **Then** Relascope reports stale graph fact counts when available
- **And** reports unverified import counts when available

### Requirement: Experimental output format option

Relascope SHALL support a `--format` option for selected commands.

#### Scenario: Human format remains default

- **Given** the user runs a supported command without `--format`
- **Then** Relascope prints the existing human-readable output

#### Scenario: Explicit human format

- **Given** the user runs a supported command with `--format human`
- **Then** Relascope prints human-readable output

#### Scenario: JSON format for scan

- **Given** an initialized workspace has registered repositories
- **When** the user runs `relascope scan --format json`
- **Then** Relascope prints valid JSON
- **And** includes scan ID
- **And** includes total files
- **And** includes added, modified, removed, and unchanged counts
- **And** includes unavailable repositories

#### Scenario: JSON format for graph summary

- **Given** a workspace has graph state
- **When** the user runs `relascope graph summary --format json`
- **Then** Relascope prints valid JSON
- **And** includes total node count
- **And** includes node counts by kind
- **And** includes total edge count
- **And** includes edge counts by relation

#### Scenario: JSON format for graph imports

- **Given** a workspace has persisted import relationships
- **When** the user runs `relascope graph imports --format json`
- **Then** Relascope prints valid JSON
- **And** includes source module
- **And** target module
- **And** status
- **And** evidence location when available

#### Scenario: Unsupported format is rejected

- **Given** the user passes an unsupported format value
- **When** Relascope parses the command
- **Then** it returns a clear error

### Requirement: JSON output is experimental

Relascope SHALL document JSON output as experimental in 0.0.3-b.

#### Scenario: Documentation states experimental JSON contract

- **Given** the user reads dogfooding or command documentation
- **Then** the documentation states that JSON output exists for scripting and inspection
- **And** states that the schema is not stable yet

### Requirement: Dogfooding guide

Relascope SHALL provide a guide for running the tool against a real local workspace.

#### Scenario: User follows dogfooding guide

- **Given** a user wants to test Relascope on a real workspace such as ReservApp
- **When** they read the dogfooding guide
- **Then** it explains how to initialize a workspace
- **And** how to add multiple local repositories
- **And** how to run scan, status, doctor, graph summary, and graph imports
- **And** how to save JSON evidence
- **And** how to record false positives, false negatives, unresolved imports, stale facts, and friction

### Requirement: Preserve existing behavior

Relascope SHALL preserve previous inventory, graph, import, and incremental stale behavior.

#### Scenario: Existing smoke flow still passes

- **Given** the automated fixture smoke flow exists
- **When** the smoke script runs
- **Then** existing inventory, graph summary, graph imports, incremental counts, and stale behavior remain valid

#### Scenario: Doctor and listing commands do not scan

- **Given** a workspace has registered repositories
- **When** the user runs `repo list` or `doctor`
- **Then** Relascope does not walk repository file trees
- **And** does not update scan inventory
- **And** does not modify repository files
