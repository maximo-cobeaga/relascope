# Verify Report: Relascope 0.0.3-c Scan Progress Visibility

## Status

Passed and accepted for archive.

## Scope Verified

This verification covers the `0.0.3-c` slice:

- human-mode scan progress visibility;
- `--silent` suppression;
- clean `scan --format json` stdout;
- cooperative cancellation plumbing for file walking and post-scan materialization stages;
- persisted `cancelled` scan status when cancellation is handled;
- existing graph/import/incremental behavior preservation;
- real workspace dogfooding on the ReservApp workspace.

## Automated Verification

Commands run from:

```text
C:\Users\MAXIMO\Desktop\relascope
```

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
  - Storage SQLite tests: 5 passed.
  - Doc-tests: passed with 0 tests.
- Smoke script: passed.

## Test Coverage Added or Confirmed

### CLI

- default human scan emits progress;
- post-scan stage progress appears in human mode;
- `scan --silent` suppresses progress and post-scan stage output;
- `scan --format json` remains valid JSON;
- cancelled last scan status is visible in `status`;
- cancelled last scan status is visible in `doctor`.

### Core

- scan progress events are emitted without changing scan results;
- cancellation token stops scan with a partial outcome.

### Storage

- minimal graph materialization stops when the cancellation token is already cancelled;
- shallow import materialization stops when the cancellation token is already cancelled;
- minimal graph materialization remains idempotent;
- shallow import materialization preserves import evidence behavior.

## Real Workspace Dogfooding Evidence

Dogfood workspace:

```text
C:\Users\MAXIMO\Desktop\relascope-dogfood
```

Binary:

```text
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe
```

Command:

```powershell
& C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe scan
```

Observed behavior:

- repository scan progress appeared immediately;
- post-scan stage messages appeared;
- minimal graph materialization progress appeared;
- shallow import materialization progress appeared;
- scan completed successfully.

Final scan summary:

```text
Scan completed
  scan_id: 72f3f402-8f19-48c8-aff8-4b4f55b90aa5
  files: 18605
  added: 18605
  modified: 0
  removed: 0
```

Doctor evidence:

```text
Relascope doctor
  config: ok
  database: ok
  workspace: relascope-dogfood (f99dc3d7-cf41-44f5-9434-149509991fe2)
  repositories: 4 total, 4 available, 0 unavailable
  last_scan: completed
  graph:
    stale_nodes: 0
    stale_edges: 0
    unverified_imports: 32646
  exclusions: 7 configured
```

Timing observations from dogfooding output:

- file walk completed in roughly 10 seconds;
- minimal graph materialization completed in roughly 2 minutes 37 seconds;
- shallow import materialization completed in roughly 6 minutes 19 seconds;
- backend dominated import materialization work with 5,861 code files and 38,391 detected imports.

## Dogfooding Bug Found and Fixed

Initial dogfooding revealed that the original file-walk heartbeat could finish while post-scan materialization continued silently. Interrupting during that silent window left the latest scan persisted as `running`.

Follow-up fixes added:

- human-mode post-scan stage messages;
- minimal graph materialization progress;
- cancellation token checks in minimal graph materialization;
- shallow import materialization progress;
- cancellation token checks in shallow import materialization;
- cancellation checks before marking the scan `completed`.

A second dogfooding run confirmed the scan no longer appears stuck and completes successfully on the real workspace.

## Residual Notes

- The final post-fix dogfooding run verified full completion, not another live Ctrl+C interruption.
- Ctrl+C behavior after the post-scan fixes is covered by deterministic cancellation tests for file scan, minimal graph materialization, and shallow import materialization.
- Hard process kills or terminal closure can still leave a scan as `running`; this remains outside the slice and should be handled by a future stale-running-scan cleanup feature.
- Progress output is intentionally verbose in this slice. A future slice should introduce compact progress output or a `--verbose` split.

## Acceptance Decision

Accepted for archive.

The slice meets its primary product goal: real users can now see long-running scan and post-scan work progressing, can suppress progress with `--silent`, can keep JSON output clean, and no longer have silent post-scan work that appears stuck during real dogfooding.
