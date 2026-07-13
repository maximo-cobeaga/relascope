# Apply Progress: Relascope 0.0.3-c Scan Progress Visibility

## Status

Applied.

## Summary

Implemented scan progress visibility and cancelled scan lifecycle support in one slice.

User-facing behavior added:

- `relascope scan` now prints terminal-friendly progress by default in human mode.
- `relascope scan --silent` suppresses progress and keeps the final summary.
- `relascope scan --format json` keeps stdout as valid JSON with no progress text.
- Ctrl+C is handled through a cooperative cancellation path and persists the scan as `cancelled` when the CLI can handle the signal.
- `status` and `doctor` display `cancelled` naturally because scan status is persisted as a string.

## Files Changed

- `Cargo.toml`
- `Cargo.lock`
- `crates/core/src/lib.rs`
- `crates/core/src/scan.rs`
- `crates/cli/src/main.rs`
- `crates/cli/tests/cli.rs`
- `scripts/smoke-graph.ps1`
- `README.md`
- `tutorial/dogfooding-real-workspace.md`
- `tutorial/test-real-workspace-step-by-step.md`
- `tutorial/current-state-and-next-steps.md`
- `openspec/changes/relascope-0-0-3-c-scan-progress-visibility/tasks.md`

## Implementation Notes

### Progress

Core scanning now supports progress events without terminal formatting in `relascope_core`.

The CLI owns formatting and prints log-friendly heartbeat lines such as:

```text
Scanning repositories...
  api: starting, 0 files indexed, 0 skipped, elapsed 00:00:00
  api: 5 files indexed, 1 skipped, elapsed 00:00:00
```

Progress is throttled for large scans and always reports repository start/finish events.

### Silent and JSON modes

Progress enablement is centralized as:

```rust
format == human && !silent
```

So JSON output remains clean for redirection.

### Cancellation

Added cooperative cancellation using `ScanCancellationToken` backed by `Arc<AtomicBool>`.

The CLI runs the synchronous file walk in `tokio::task::spawn_blocking` and keeps a `tokio::signal::ctrl_c()` listener active during the scan lifecycle. On handled Ctrl+C, it signals cancellation, waits for the scan task/checkpoint, persists `cancelled`, prints `Scan cancelled`, and does not print the successful final summary.

The scan loop checks cancellation:

- before each repository;
- during file iteration;
- before/after file hashing;
- between major post-scan persistence/materialization stages.

### Skipped count

`skipped` counts excluded entries encountered by the walker when practical. It intentionally does not walk excluded directory descendants just to count them. This is a progress signal, not an exact excluded-file metric.

## Verification Evidence

Commands run from `C:\Users\MAXIMO\Desktop\relascope`:

```powershell
cargo fmt --check
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Results:

- `cargo fmt --check`: passed.
- `cargo test`: passed.
  - CLI integration tests: 11 passed.
  - Core unit tests: 13 passed.
  - Storage SQLite tests: 3 passed.
  - Doc-tests: passed with 0 tests.
- Smoke script: passed.

Smoke confirmed default progress appears on the first scan and `--silent` keeps later smoke scan output stable.

## Remaining Manual Check

The implementation includes Ctrl+C handling and deterministic tests for persisted `cancelled` status visibility, but a live manual Ctrl+C test against a large workspace is still recommended during the next dogfooding run.

Manual check to run:

```powershell
& $relascope scan
# Press Ctrl+C while scan is active
& $relascope doctor
```

Expected:

```text
last_scan: cancelled
```

## Scope Boundaries Preserved

This apply did not add:

- exact percentage progress;
- mandatory pre-count walk;
- watcher;
- scan resume;
- Tree-sitter;
- cross-repository resolution;
- impact analysis;
- AI/network behavior.
