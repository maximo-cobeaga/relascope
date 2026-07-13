# Specification Delta: Incremental Scan State and Stale Graph Marking

## ADDED Requirements

### Requirement: Scan-to-scan file change summary

Relascope SHALL compare the current scan with the previous completed scan for the same workspace and produce an incremental file-change summary.

#### Scenario: First scan reports all files as added

- **Given** an initialized workspace has registered repositories
- **And** no previous completed scan exists
- **When** the user runs `relascope scan`
- **Then** Relascope completes the scan successfully
- **And** reports current file count
- **And** reports all inventoried files as `added`
- **And** reports `modified: 0`
- **And** reports `removed: 0`

#### Scenario: Second scan with no file changes reports unchanged state

- **Given** a workspace has a previous completed scan
- **And** no registered repository files changed
- **When** the user runs `relascope scan` again
- **Then** Relascope completes the scan successfully
- **And** reports `added: 0`
- **And** reports `modified: 0`
- **And** reports `removed: 0`
- **And** does not duplicate graph nodes or edges

#### Scenario: New file is reported as added

- **Given** a workspace has a previous completed scan
- **And** a new non-excluded file is added to an available registered repository
- **When** the user runs `relascope scan`
- **Then** Relascope reports the new file in the `added` count
- **And** materializes current graph facts for that file when applicable

#### Scenario: Existing file content change is reported as modified

- **Given** a workspace has a previous completed scan
- **And** a previously inventoried file still exists at the same relative path
- **And** its content hash changes
- **When** the user runs `relascope scan`
- **Then** Relascope reports the file in the `modified` count

#### Scenario: Removed file is reported as removed

- **Given** a workspace has a previous completed scan
- **And** a previously inventoried file no longer exists in the current scan
- **When** the user runs `relascope scan`
- **Then** Relascope reports the file in the `removed` count

### Requirement: Preserve per-scan inventory history

Relascope SHALL preserve the existing per-scan `files` inventory model.

#### Scenario: File rows remain scan-specific

- **Given** a workspace has multiple completed scans
- **When** Relascope compares scan state
- **Then** it reads previous file state from the previous completed scan
- **And** reads current file state from the current scan results
- **And** does not require a new `current_files` table for 0.0.3-a

### Requirement: Stale marking for removed files

Relascope SHALL mark graph facts associated with removed files as `stale` rather than deleting them.

#### Scenario: Removed file marks File node stale

- **Given** a file was present in the previous completed scan
- **And** its `File` node exists in the graph
- **And** the file is absent from the current scan
- **When** Relascope processes stale graph marking
- **Then** the corresponding `File` node status or metadata is marked `stale`
- **And** the node remains persisted

#### Scenario: Removed code file marks source Module node stale

- **Given** a recognized code file was present in the previous completed scan
- **And** a source `Module` node exists for that file
- **And** the file is absent from the current scan
- **When** Relascope processes stale graph marking
- **Then** the corresponding source `Module` node is marked `stale`
- **And** the node remains persisted

#### Scenario: Removed code file marks outgoing imports stale

- **Given** a source `Module` node has outgoing `IMPORTS` edges
- **And** the source file for that module is removed in the current scan
- **When** Relascope processes stale graph marking
- **Then** outgoing `IMPORTS` edges from that module are marked `stale`
- **And** evidence for those edges remains available

#### Scenario: Removed file marks containment edges stale where appropriate

- **Given** a removed file had graph containment edges
- **When** Relascope processes stale graph marking
- **Then** containment edges directly connecting stale file/module facts are marked `stale` where appropriate
- **And** repository/workspace containment remains active for still-registered repositories

### Requirement: Stale marking for modified code-file imports

Relascope SHALL mark previous import facts from a modified code file as `stale` before materializing current imports.

#### Scenario: Modified code file stales previous outgoing imports

- **Given** a recognized code file existed in the previous completed scan
- **And** the file exists in the current scan with a different content hash
- **And** the file's source module has outgoing `IMPORTS` edges
- **When** Relascope processes the current scan
- **Then** previous outgoing `IMPORTS` edges from that module are marked `stale`
- **And** current detected imports are materialized as `active` or `unverified`

#### Scenario: Removed import no longer remains active after modification

- **Given** a code file imported `./old` in the previous scan
- **And** that import line is removed before the current scan
- **When** the user runs `relascope scan`
- **Then** the old `IMPORTS` edge is not left as `active`
- **And** it is marked `stale`

#### Scenario: New import appears after modification

- **Given** a code file changes to add a new import line
- **When** the user runs `relascope scan`
- **Then** Relascope materializes the new import relationship
- **And** stores line-level evidence for the new import

### Requirement: Current facts remain active or unverified

Relascope SHALL keep current facts active or unverified according to existing graph behavior.

#### Scenario: Current resolved import is active

- **Given** a current scan detects a local resolved import
- **When** Relascope materializes the import edge
- **Then** the edge status is `active`

#### Scenario: Current unresolved import is unverified

- **Given** a current scan detects an unresolved or external import
- **When** Relascope materializes the import edge
- **Then** the edge status is `unverified`

#### Scenario: Reintroduced stale import becomes current again

- **Given** an import edge was previously marked `stale`
- **And** the same import appears in a later current scan
- **When** Relascope materializes current imports
- **Then** the edge status is restored to `active` or `unverified` according to resolution

### Requirement: Scan output includes incremental counts

Relascope SHALL include incremental counts in `relascope scan` output.

#### Scenario: Scan output includes added modified and removed counts

- **Given** a scan completes successfully
- **When** Relascope prints scan output
- **Then** output includes `added: <count>`
- **And** includes `modified: <count>`
- **And** includes `removed: <count>`

### Requirement: Graph inspection can expose stale state

Relascope SHALL make stale graph state visible through existing graph inspection commands where relevant.

#### Scenario: Graph imports shows stale import status

- **Given** an import edge was marked `stale`
- **When** the user runs `relascope graph imports`
- **Then** the import appears with status `stale`
- **And** its evidence remains visible when available

#### Scenario: Graph summary remains deterministic with stale facts

- **Given** a workspace contains active and stale graph facts
- **When** the user runs `relascope graph summary`
- **Then** Relascope reports persisted graph counts deterministically
- **And** does not scan repositories
- **And** does not silently delete stale facts

### Requirement: Scope boundaries

Relascope SHALL keep 0.0.3-a limited to basic incremental scan state and stale graph marking.

#### Scenario: No watcher is started

- **Given** the user runs `relascope scan`
- **Then** Relascope performs a single scan operation
- **And** does not start a file watcher

#### Scenario: No rename intelligence is inferred

- **Given** a file is renamed between scans
- **When** the user runs `relascope scan`
- **Then** Relascope may report the old path as removed
- **And** may report the new path as added
- **And** does not claim a verified rename relationship in 0.0.3-a

#### Scenario: No semantic parser expansion is introduced

- **Given** a workspace contains Python or TypeScript files
- **When** the user runs `relascope scan`
- **Then** Relascope does not introduce Tree-sitter or full parser semantics as part of this change

#### Scenario: No external capabilities are used

- **Given** a workspace has registered repositories
- **When** the user runs `relascope scan`
- **Then** Relascope does not use AI
- **And** does not require network access
- **And** does not execute code from registered repositories
