# Handoff: Relascope 0.0.3-c Scan Progress Visibility

## Status

Implementation applied and automated verification passed.

## What changed

Relascope now supports scan progress visibility for large workspaces:

- `relascope scan` shows heartbeat progress by default in human mode.
- `relascope scan --silent` suppresses progress output.
- `relascope scan --format json` keeps stdout as valid JSON only.
- Ctrl+C is handled through cooperative cancellation when the process can catch it.
- Handled interrupted scans are persisted as `cancelled`.
- `status` and `doctor` can show `cancelled` as the latest scan state.

## Automated verification already completed

Run from `C:\Users\MAXIMO\Desktop\relascope`:

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
- Smoke test: passed.

## Follow-up after failed dogfooding

The first ReservApp dogfooding run exposed a post-scan visibility/cancellation gap: repository file-walk progress reached the final repo, then the command stayed silent during post-scan materialization. Interrupting there left `doctor` reporting `last_scan: running`.

Fix now applied:

- post-scan stage messages are printed in human mode;
- minimal graph materialization reports file/repository progress;
- cancellation is checked inside minimal graph materialization;
- shallow import materialization reports code-file/import progress;
- cancellation is checked inside shallow import materialization;
- cancellation is checked before starting imports and before marking the scan `completed`;
- automated verification passed again.

## Pending manual dogfooding for tomorrow

### 1. Rebuild the local binary

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo build -p relascope-cli
```

Expected binary:

```text
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe
```

If needed:

```powershell
$relascope = "C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe"
```

### 2. Return to the ReservApp dogfood workspace

```powershell
cd C:\Users\MAXIMO\Desktop\relascope-dogfood
```

Optional sanity check:

```powershell
& $relascope repo list
& $relascope doctor
```

### 3. Run the real scan and confirm progress heartbeat

```powershell
& $relascope scan
```

Expected behavior:

```text
Scanning repositories...
  backend: ... files indexed, ... skipped, elapsed ...
  app: ... files indexed, ... skipped, elapsed ...
Persisting inventory...
Marking stale graph facts...
Materializing graph...
  graph: .../... files, .../... repositories, elapsed ...
Materializing shallow imports...
  imports backend: .../... code files, ... imports, elapsed ...
```

Record:

- whether progress appears quickly;
- which repo is slowest;
- whether indexed/skipped counts feel useful;
- total scan time;
- final file count;
- any repo that appears unexpectedly large.

### 4. Test Ctrl+C cancellation while scan is active

Only do this if the scan is still running long enough to interrupt safely:

```powershell
& $relascope scan
# Press Ctrl+C while it is actively scanning
```

Expected output:

```text
Scan cancelled
```

Then verify persisted status:

```powershell
& $relascope doctor
& $relascope status
```

Expected:

```text
last_scan: cancelled
```

or in status:

```text
status: cancelled
```

### 5. Re-run scan after cancellation

```powershell
& $relascope scan
```

Expected:

- scan can run again normally;
- cancelled scan is not used as the previous completed baseline;
- final status becomes `completed` if the scan finishes.

### 6. Verify JSON remains clean

```powershell
& $relascope scan --format json > scan.json
Get-Content .\scan.json
```

Expected:

- file contains JSON only;
- no `Scanning repositories...` lines inside `scan.json`.

### 7. Capture dogfooding notes

Record findings in the dogfood workspace, for example:

```powershell
notepad .\dogfood-notes.md
```

Minimum notes:

```markdown
# Relascope 0.0.3-c dogfood notes

Date:
Workspace:
Repos:

## Progress behavior
- Did progress appear quickly?
- Which repo was slowest?
- Was indexed/skipped useful?

## Cancellation behavior
- Was Ctrl+C handled?
- Did doctor/status show cancelled?
- Could scan run again afterward?

## JSON behavior
- Was scan.json valid JSON only?

## Bugs or UX friction
-
```

## Next SDD steps after manual confirmation

If manual dogfooding passes:

1. Create/update `verify-report.md` for this OpenSpec change.
2. Sync the spec into `openspec/specs/scan-progress-visibility/spec.md`.
3. Archive the change.
4. Update Engram with the final dogfooding outcome.

If dogfooding fails:

1. Keep the change active.
2. Record the failing command/output.
3. Fix only the scoped issue.
4. Re-run automated verification plus the failed manual check.

## Known limitation

A hard process kill or terminal close can still leave a scan as `running`. This slice handles normal Ctrl+C when the process is alive and can receive the signal.
