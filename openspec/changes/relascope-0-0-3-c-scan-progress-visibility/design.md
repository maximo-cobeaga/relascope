# Design: Relascope 0.0.3-c Scan Progress Visibility

## Overview

Relascope needs visible progress during long `scan` runs. The design should favor operational trust over visual polish: a heartbeat that proves work is advancing is more valuable than an exact progress bar that requires an expensive pre-count pass.

This slice adds three related capabilities:

1. default human-mode scan progress;
2. `--silent` to suppress progress;
3. cooperative cancellation that persists `cancelled` instead of leaving scans as `running`.

The implementation should keep JSON stdout clean and preserve existing scan/materialization behavior.

## Current Scan Flow

Current CLI flow in `crates/cli/src/main.rs`:

```text
open workspace
list repositories
load previous completed scan files
begin_scan -> scans.status = running
scan_repositories(...)       // synchronous file walk
compute_file_changes
update repository availability
persist files
mark stale graph facts
materialize minimal graph
materialize shallow imports
finish_scan(..., completed)
print final output
```

Current core scan flow in `crates/core/src/scan.rs`:

```text
for repository in repositories:
  WalkDir repository path
  filter excluded entries
  for each file:
    classify
    maybe hash
    push FileInventoryRecord
    update ScanSummary
```

The visibility gap is in `scan_repositories`: it can walk and hash many files without any output.

## CLI Surface

Extend scan args from format-only to scan-specific args:

```rust
#[derive(Debug, Args)]
struct ScanArgs {
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,

    /// Suppress scan progress output.
    #[arg(long)]
    silent: bool,
}
```

Command mapping changes from:

```rust
Scan(FormatArgs)
```

to:

```rust
Scan(ScanArgs)
```

Behavior:

- `relascope scan`: progress enabled, final human summary printed.
- `relascope scan --silent`: progress disabled, final human summary printed.
- `relascope scan --format json`: stdout must remain JSON only.
- `relascope scan --format json --silent`: also valid; same clean JSON stdout.

Recommended first-slice rule:

```rust
let progress_enabled = args.format == OutputFormat::Human && !args.silent;
```

That means JSON mode emits no progress by default in this slice. This avoids stderr/stdout ambiguity until we define a machine-readable progress story.

## Progress Reporter Abstraction

Add a small core-side progress interface, not a terminal dependency:

```rust
pub trait ScanProgressReporter {
    fn repository_started(&mut self, repository_visible_id: &str);
    fn file_indexed(&mut self, repository_visible_id: &str);
    fn file_skipped(&mut self, repository_visible_id: &str);
    fn repository_finished(&mut self, repository_visible_id: &str);
}
```

Use a no-op reporter when progress is disabled.

For simpler ownership and testability, implementation may use callbacks instead of a trait:

```rust
pub struct ScanProgress {
    pub repository_visible_id: String,
    pub indexed_files: u64,
    pub skipped_files: u64,
    pub elapsed: Duration,
}

pub type ProgressCallback<'a> = dyn FnMut(ScanProgress) + 'a;
```

Either is acceptable if the final implementation keeps CLI formatting out of the core scanning logic.

### Why reporter in core?

The file walk happens in `relascope_core::scan`. The core knows when repositories, files, and skipped paths are encountered. The CLI should own how progress is printed; the core should only report scan events.

## Heartbeat Strategy

Do not print on every file. For large repositories that would make the terminal noisy and slow.

Use throttling:

```text
emit progress when either:
- at least 500 indexed/skipped events have happened since the last emit; or
- at least 2 seconds elapsed since the last emit.
```

Recommended output shape:

```text
Scanning repositories...
  backend: 500 files indexed, 120 skipped, elapsed 00:00:03
  backend: 1,000 files indexed, 340 skipped, elapsed 00:00:06
  app: 500 files indexed, 2,200 skipped, elapsed 00:00:11
```

This is intentionally log-friendly. It should work in PowerShell, redirected terminals, CI logs, and basic terminals. Avoid carriage-return animation in this slice.

## Counting Indexed vs Skipped

`indexed` means a file accepted into `FileInventoryRecord` and counted in `ScanSummary`.

`skipped` means an excluded entry/path was pruned or ignored when practical.

Implementation detail:

- `WalkDir::filter_entry` currently filters excluded directories before the main loop sees them.
- Counting every skipped file inside an excluded directory would require walking it, which defeats the exclusion.
- Therefore, for this slice, `skipped` may count excluded directory/file entries encountered by `filter_entry`, not every descendant inside a pruned directory.

Document this as a progress signal, not an exact skipped-file metric.

## Cancellation Design

### Problem

The CLI currently creates a scan row as `running`, then performs synchronous scanning. If the process receives `Ctrl+C`, the row can remain `running` forever.

### Target

When cancellation can be handled safely:

```text
running -> cancelled
```

and final successful summary must not be printed.

### Cooperative Cancellation Token

Introduce a small cancellation token checked by the scan loop:

```rust
#[derive(Clone, Default)]
pub struct ScanCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl ScanCancellationToken {
    pub fn cancel(&self) { ... }
    pub fn is_cancelled(&self) -> bool { ... }
}
```

Pass it through scan options:

```rust
pub struct ScanOptions {
    pub workspace_id: String,
    pub scan_id: String,
    pub exclusions: Vec<String>,
    pub cancellation: Option<ScanCancellationToken>,
}
```

The scan loop checks before entering a repository, during file iteration, and before expensive file hashing.

If cancellation is observed, return a typed error:

```rust
pub enum ScanError {
    Cancelled,
    Other(...),
}
```

If changing the project error type is too large, use an error variant/string wrapper but keep CLI detection reliable. Prefer a typed error.

### Ctrl+C Handling

The project already uses Tokio. Add Tokio signal support rather than a new terminal UI dependency.

Cargo change:

```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
```

CLI scan flow should run the blocking file walk in `spawn_blocking` and keep a Ctrl+C listener active for the whole scan lifecycle:

```rust
let cancellation = ScanCancellationToken::default();
let cancellation_listener = tokio::spawn({
    let cancellation = cancellation.clone();
    async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancellation.cancel();
        }
    }
});
let scan_task = tokio::task::spawn_blocking({
    let cancellation = cancellation.clone();
    move || scan_repositories(... cancellation ...)
});

let outcome = scan_task.await??;
if outcome.cancelled || cancellation.is_cancelled() {
    store.finish_scan(&scan_id, "cancelled", &outcome.summary).await?;
    eprintln!("Scan cancelled");
    return Ok(());
}
// Check the same token between post-scan persistence/materialization stages.
```

Important: after Ctrl+C, the cooperative scan loop should notice cancellation and return. Because hashing is capped at 10 MB today, checking between files and before hashing should be sufficient for this slice. Keeping the listener alive through post-scan stages also lets Relascope mark `cancelled` if interruption happens after the file walk but before final completion.

### Persisted Summary for Cancelled Scans

Use a `ScanSummary` for cancelled scans too.

Preferred behavior:

- persist the partial summary if the scan loop can return it;
- otherwise persist default summary.

A useful implementation shape:

```rust
pub enum ScanRunResult {
    Completed(ScanOutcome),
    Cancelled(ScanOutcome),
}
```

This lets `finish_scan(scan_id, "cancelled", &partial_summary)` store useful partial counts without pretending the scan completed.

If partial outcome plumbing is too large, default summary is acceptable for first implementation, but the design preference is partial counts.

### Cancellation During Persistence/Materialization

This slice primarily targets cancellation during the long file-walk phase. After the file walk completes, persistence/materialization should usually be faster but can still take time on large workspaces.

Minimum acceptable behavior:

- if cancellation is observed before persistence starts, mark `cancelled` and do not persist files/materialize graph;
- if cancellation happens after the scan has committed final storage changes, do not mark a completed scan as cancelled;
- avoid leaving `running` if the CLI is still alive and can handle the signal.

Optional if cheap:

- check cancellation between major post-scan stages:
  - before `persist_scan_files`;
  - before `mark_stale_for_file_changes`;
  - before materialization;
  - before `finish_scan(completed)`.

Do not attempt transactional rollback across all post-scan stages in this slice.

## Storage Design

Current `finish_scan(scan_id, status, summary)` can already persist arbitrary status strings. No schema migration is required for `cancelled` unless status validation is added later.

Add constants in storage or core if useful:

```rust
pub const SCAN_STATUS_RUNNING: &str = "running";
pub const SCAN_STATUS_COMPLETED: &str = "completed";
pub const SCAN_STATUS_CANCELLED: &str = "cancelled";
```

`status` and `doctor` already read status as a string, so they should display `cancelled` naturally. Tests should prove this.

## Output Design

### Human progress

Print progress to stdout for human mode.

Rationale: human scan output is already stdout; progress is part of the human interaction.

### JSON mode

Do not print progress in JSON mode in this slice.

Rationale: the most important contract is clean stdout for redirection:

```powershell
& $relascope scan --format json > scan.json
```

### Cancellation message

Print a short message when cancellation is handled:

```text
Scan cancelled
```

Prefer stderr for cancellation messages to avoid confusing structured output. In human mode either stdout or stderr is acceptable, but stderr is safer if users later redirect stdout.

## Test Strategy

### Unit tests

Core scan tests:

- reporter receives repository/file progress events;
- `--silent` equivalent no-op reporter does not affect scan results;
- cancellation token checked during scan returns cancelled result;
- skipped count does not require walking excluded descendants.

### CLI integration tests

- `relascope scan --silent` completes and prints final summary without progress heartbeat text.
- `relascope scan --format json` remains parseable JSON.
- `status` can display a manually or programmatically persisted `cancelled` last scan.
- `doctor` can display a manually or programmatically persisted `cancelled` last scan.

Testing real `Ctrl+C` end-to-end may be brittle in the existing integration harness. Prefer testing the cancellation path through a deterministic internal seam. If feasible later, add one process-level test that starts a large fixture scan and sends interrupt, but do not make the suite flaky.

### Smoke test

Update `scripts/smoke-graph.ps1` carefully:

- use `relascope scan --silent` where exact output assertions expect the old concise summary; or
- update assertions to tolerate heartbeat lines.

Prefer `--silent` in smoke for stable output, plus one focused check that normal scan emits progress on a fixture if practical.

## Documentation Updates

Update:

- `README.md`: mention `relascope scan --silent`.
- `tutorial/dogfooding-real-workspace.md`: explain progress heartbeat during large scans.
- `tutorial/test-real-workspace-step-by-step.md`: replace uncertainty guidance with expected progress behavior.
- `tutorial/current-state-and-next-steps.md`: bring stale milestone status up to date if not done separately.

## Tradeoffs

### Heartbeat over percentage

Pros:

- starts immediately;
- no extra full tree walk;
- useful on huge repositories;
- simpler and more robust across terminals.

Cons:

- does not tell exact percent complete;
- skipped count is an operational signal, not an exact skipped-file total.

This matches the dogfooding problem: the user needs to know the scan is alive and which repo is expensive.

### Cooperative cancellation over hard abort

Pros:

- lets Relascope persist `cancelled` cleanly;
- avoids corrupting internal state intentionally;
- keeps implementation understandable.

Cons:

- cancellation may wait until the current file/hash operation finishes;
- cannot always handle OS-level process kill.

This is acceptable for `Ctrl+C`. Hard kills can still leave `running`, and future doctor cleanup can detect stale running scans by age.

## Risks

- Progress printing could slow scans if emitted too often. Mitigation: throttle by count/time.
- Cancellation handling could complicate async/sync boundaries. Mitigation: keep file walk in `spawn_blocking`, use an atomic token, and persist cancellation in CLI orchestration.
- JSON output could be corrupted if progress accidentally prints in JSON mode. Mitigation: centralize `progress_enabled` logic and test JSON parsing.
- Smoke assertions may become brittle. Mitigation: use `--silent` for existing smoke checks.

## Open Questions

None blocking. Exact heartbeat interval can be tuned during implementation; recommended initial values are 500 events or 2 seconds.
