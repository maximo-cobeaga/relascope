# Specification: Scan Progress Visibility

## Requirements

### Requirement: Human scan shows progress by default

Relascope SHALL show terminal-friendly progress feedback by default when the user runs `relascope scan` in human output mode.

#### Scenario: Human scan starts with progress feedback

- **Given** an initialized workspace has at least one registered available repository
- **When** the user runs `relascope scan`
- **Then** Relascope prints a progress start message before the final scan summary
- **And** the output makes clear that scanning is active
- **And** the command still prints the final scan summary after completion

#### Scenario: Progress identifies current repository

- **Given** an initialized workspace has multiple registered repositories
- **When** the user runs `relascope scan`
- **Then** progress output identifies which repository is currently being scanned
- **And** progress output changes as scanning advances across repositories

#### Scenario: Progress includes useful scan counters

- **Given** a scan is processing repository files
- **When** Relascope emits progress output
- **Then** progress includes a count of files indexed or seen
- **And** includes skipped files when that count is practical to collect
- **And** includes elapsed time or equivalent timing feedback

#### Scenario: Progress is heartbeat-based not percentage-based

- **Given** a large repository may take noticeable time to scan
- **When** Relascope emits progress output
- **Then** progress does not require an exact total file count before scanning
- **And** Relascope does not perform a mandatory pre-count walk solely to compute a percentage

### Requirement: Human scan supports silent mode

Relascope SHALL let users suppress scan progress output with `--silent`.

#### Scenario: Silent scan suppresses progress heartbeat

- **Given** an initialized workspace has registered repositories
- **When** the user runs `relascope scan --silent`
- **Then** Relascope does not print progress heartbeat lines
- **And** still performs the scan
- **And** still prints the final human scan summary after completion

#### Scenario: Silent scan preserves existing summary behavior

- **Given** a scan completes successfully with `--silent`
- **When** Relascope prints the final summary
- **Then** the summary includes scan ID, file count, added count, modified count, and removed count
- **And** existing human summary expectations remain valid

### Requirement: JSON scan output remains clean

Relascope SHALL keep `relascope scan --format json` stdout valid JSON without human progress lines mixed into it.

#### Scenario: JSON scan prints valid JSON to stdout

- **Given** an initialized workspace has registered repositories
- **When** the user runs `relascope scan --format json`
- **Then** stdout is valid JSON
- **And** stdout includes scan ID, status, files, added, modified, removed, unchanged, and unavailable repositories
- **And** stdout does not include human progress lines

#### Scenario: JSON scan can be redirected to a file

- **Given** a user redirects JSON scan output to `scan.json`
- **When** the command completes
- **Then** `scan.json` contains valid JSON only
- **And** no progress heartbeat text corrupts the file

### Requirement: Cancelled scans are persisted correctly

Relascope SHALL mark a running scan as `cancelled` when the user interrupts it and cancellation can be handled safely.

#### Scenario: Ctrl+C marks scan cancelled

- **Given** `relascope scan` has created a running scan record
- **And** the scan is still in progress
- **When** the user interrupts the command with `Ctrl+C`
- **Then** Relascope updates the scan status to `cancelled` when possible
- **And** the scan is not left persisted as `running`
- **And** the command exits without reporting successful completion

#### Scenario: Cancelled scan may print a short cancellation message

- **Given** a human scan is interrupted
- **When** Relascope handles cancellation
- **Then** it may print a concise message such as `Scan cancelled`
- **And** it does not print the normal successful final scan summary

#### Scenario: Cancelled scan status appears in status output

- **Given** the last scan was cancelled
- **When** the user runs `relascope status`
- **Then** Relascope reports the last scan status as `cancelled`
- **And** does not imply that the scan is still running

#### Scenario: Cancelled scan status appears in doctor output

- **Given** the last scan was cancelled
- **When** the user runs `relascope doctor`
- **Then** Relascope reports the last scan status as `cancelled`
- **And** does not imply that the scan is still running

### Requirement: Existing scan behavior is preserved

Relascope SHALL preserve existing scan, graph materialization, import detection, stale marking, JSON, doctor, status, and smoke-test behavior while adding progress visibility and cancelled scan state.

#### Scenario: Successful scan still materializes graph facts

- **Given** an initialized workspace has registered repositories
- **When** the user runs `relascope scan`
- **Then** Relascope persists file inventory rows
- **And** materializes workspace, repository, file, module, containment, import, and evidence graph facts according to existing behavior

#### Scenario: Incremental counts still work

- **Given** a workspace has a previous completed scan
- **When** the user runs another scan
- **Then** Relascope still reports added, modified, removed, and unchanged counts according to existing incremental behavior

#### Scenario: Stale graph marking still works

- **Given** a file was removed or a code file import changed between scans
- **When** the user runs `relascope scan`
- **Then** Relascope still marks obsolete graph facts as `stale` according to existing stale-marking behavior

#### Scenario: Existing graph inspection commands remain read-only

- **Given** a workspace has persisted graph state
- **When** the user runs `relascope graph summary`, `relascope graph imports`, `relascope status`, or `relascope doctor`
- **Then** those commands do not walk repository files
- **And** do not start scan progress output
- **And** do not mutate repository source files

### Requirement: Scope boundaries

Relascope SHALL keep this change limited to scan visibility and cancelled scan lifecycle state.

#### Scenario: No watcher is introduced

- **Given** the user runs `relascope scan`
- **Then** Relascope performs a single scan operation
- **And** does not start a file watcher

#### Scenario: No exact progress total is required

- **Given** a repository contains many files
- **When** Relascope scans it
- **Then** Relascope does not require a full pre-scan count before starting useful work
- **And** progress can be based on heartbeat counters rather than percentage completion

#### Scenario: No semantic parser expansion is introduced

- **Given** a workspace contains Python, TypeScript, or JavaScript files
- **When** the user runs `relascope scan`
- **Then** Relascope does not introduce Tree-sitter or full parser semantics as part of this change

#### Scenario: No external capabilities are used

- **Given** a workspace has registered repositories
- **When** the user runs `relascope scan`
- **Then** Relascope does not use AI
- **And** does not require network access
- **And** does not execute code from registered repositories
