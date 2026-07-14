# Tasks: Relascope 0.0.3-c Scan Progress Visibility

## Implementation Plan

Keep this slice focused on scan progress visibility and cancelled scan lifecycle state. Do not add watcher, exact percentage progress, pre-count walking, Tree-sitter, cross-repository resolution, or impact analysis.

## 1. CLI scan arguments

- [x] Replace `Command::Scan(FormatArgs)` with `Command::Scan(ScanArgs)`.
- [x] Add `ScanArgs` with `--format human|json` and `--silent`.
- [x] Keep `human` as default format.
- [x] Ensure `relascope scan --silent` is accepted.
- [x] Ensure `relascope scan --format json --silent` is accepted.
- [x] Centralize progress enablement as `format == human && !silent`.

## 2. Progress event model

- [x] Add a small scan progress event/read model in `crates/core/src/scan.rs`.
- [x] Include current repository visible ID.
- [x] Include indexed file count.
- [x] Include skipped entry count where practical.
- [x] Include elapsed duration or enough data for CLI to compute elapsed.
- [x] Keep terminal formatting out of `relascope_core`.
- [x] Add a no-op path for disabled progress.

## 3. Heartbeat progress reporting

- [x] Emit a human scan start message such as `Scanning repositories...` when progress is enabled.
- [x] Report repository transitions.
- [x] Report progress no more often than a useful threshold.
- [x] Use initial thresholds around 500 events or 2 seconds.
- [x] Print log-friendly heartbeat lines, not carriage-return animation.
- [x] Include repo ID, indexed files, skipped entries, and elapsed time.
- [x] Avoid printing progress in JSON mode.
- [x] Avoid printing progress when `--silent` is set.

## 4. Skipped-count behavior

- [x] Count accepted/indexed files exactly.
- [x] Count skipped excluded entries when practical.
- [x] Do not walk excluded directory descendants just to count skipped files.
- [x] Treat skipped count as an operational signal, not an exact excluded-file total.
- [x] Add comments or docs clarifying this limitation.

## 5. Cancellation token

- [x] Add a cooperative scan cancellation token using `Arc<AtomicBool>`.
- [x] Pass the token through scan options or scan execution context.
- [x] Check cancellation before each repository.
- [x] Check cancellation during file iteration.
- [x] Check cancellation before expensive file hashing.
- [x] Return a typed or reliably detectable cancelled result.
- [x] Preserve partial scan summary if practical.

## 6. Tokio Ctrl+C integration

- [x] Enable Tokio `signal` feature in `Cargo.toml` if needed.
- [x] Run synchronous `scan_repositories` work inside `tokio::task::spawn_blocking`.
- [x] Race scan completion against `tokio::signal::ctrl_c()`.
- [x] On Ctrl+C, signal cancellation token.
- [x] Wait for cooperative scan completion when practical.
- [x] Persist current scan as `cancelled`.
- [x] Print concise cancellation message such as `Scan cancelled`.
- [x] Do not print successful final scan summary after cancellation.

## 7. Scan lifecycle statuses

- [x] Add scan status constants if useful: `running`, `completed`, `cancelled`.
- [x] Use `completed` for successful scans.
- [x] Use `cancelled` for handled interruptions.
- [x] Ensure `previous_completed_scan_files` continues to consider only `completed` scans.
- [x] Ensure cancelled scans do not become comparison baselines for incremental state.

## 8. Post-scan cancellation checkpoints

- [x] Check cancellation before persisting scan files.
- [x] Check cancellation before stale marking.
- [x] Check cancellation before minimal graph materialization.
- [x] Check cancellation before shallow import materialization.
- [x] If cancelled before these stages, persist `cancelled` and stop safely.
- [x] Do not attempt broad rollback across partially completed post-scan stages in this slice.

## 9. Status and doctor behavior

- [x] Verify `status` displays `cancelled` when it is the last scan status.
- [x] Verify `doctor` displays `cancelled` when it is the last scan status.
- [x] Avoid special casing that implies cancelled scans are still running.
- [x] Add tests for cancelled last scan visibility.

## 10. Tests

- [x] Add core tests for progress event/reporting behavior.
- [x] Add core tests for cancellation token behavior if deterministic.
- [x] Add CLI test that `scan --silent` completes without heartbeat text.
- [x] Add CLI test that normal human `scan` emits progress/start feedback.
- [x] Add CLI test that `scan --format json` remains parseable JSON.
- [x] Add storage/CLI test for `cancelled` last scan in `status`.
- [x] Add storage/CLI test for `cancelled` last scan in `doctor`.
- [x] Keep existing CLI/core/storage tests passing.

## 11. Smoke script

- [x] Update `scripts/smoke-graph.ps1` to tolerate progress output or use `--silent` where stable output is expected.
- [x] Keep smoke coverage for graph summary, imports, JSON, incremental modified/removed counts, stale behavior, and repo remove.
- [x] Optionally include one focused check that default scan emits progress feedback.
- [x] Avoid noisy smoke output if it makes failures harder to read.

## 12. Documentation

- [x] Update `README.md` with `relascope scan --silent`.
- [x] Update `tutorial/dogfooding-real-workspace.md` to describe expected scan heartbeat during large scans.
- [x] Update `tutorial/test-real-workspace-step-by-step.md` so users know scan progress means the command is alive.
- [x] Update `tutorial/current-state-and-next-steps.md` to reflect that 0.0.3-a and 0.0.3-b are closed and 0.0.3-c is the active dogfooding polish slice.
- [x] Document that skipped count is a progress signal and may not equal all files under excluded directories.

## 13. Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run `pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1`.
- [x] Manually test `relascope scan` on a small fixture and confirm progress appears.
- [x] Manually test `relascope scan --silent` and confirm no progress heartbeat appears.
- [x] Manually test `relascope scan --format json > scan.json` and confirm valid JSON only.
- [ ] Manually test Ctrl+C during a large scan when practical and confirm `doctor` shows `last_scan: cancelled`.

## Dogfooding Fix: Post-scan Progress and Cancellation

Real workspace dogfooding showed the file-walk heartbeat can finish while post-scan graph/import materialization continues silently. Interrupting during that silent window can leave the current scan persisted as `running`.

- [x] Add human-mode stage messages after file walking: inventory persistence, stale marking, graph materialization, and shallow import materialization.
- [x] Add minimal graph materialization progress reporting for file/repository facts.
- [x] Pass the scan cancellation token into minimal graph materialization.
- [x] Check cancellation during minimal graph repository and file loops.
- [x] Check cancellation after minimal graph materialization before starting shallow imports.
- [x] Add shallow import materialization progress reporting for code files/imports.
- [x] Pass the scan cancellation token into shallow import materialization.
- [x] Check cancellation during shallow import repository, code-file, and import-fact loops.
- [x] Check cancellation after shallow import materialization before marking the scan `completed`.
- [x] Add deterministic storage tests for cancelled minimal graph and shallow import materialization.
- [x] Extend CLI progress test so post-scan stage output appears by default and remains suppressed with `--silent`.
- [x] Re-run `cargo fmt --check`, `cargo test`, and `scripts/smoke-graph.ps1`.

## Review Workload Forecast

This is a medium-sized CLI/core lifecycle change. It touches scan arguments, scan loop internals, cancellation plumbing, tests, smoke script, and docs. Expected review risk is moderate because cancellation and output contracts can regress existing dogfooding flows.

If implementation grows beyond a comfortable review slice, split into:

1. default heartbeat progress + `--silent` + JSON cleanliness;
2. cancelled scan status and Ctrl+C handling;
3. docs and smoke updates.
