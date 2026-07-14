# Change Proposal: Relascope 0.0.3-c Scan Progress Visibility

## Status

Draft for approval before specification, design, tasks, and implementation.

## Problem Statement

Relascope can now scan real local workspaces, but real dogfooding on a large ReservApp workspace exposed an operational trust problem: `relascope scan` prints no progress until it finishes. On large repositories, the user cannot tell whether the scan is working, slow, or stuck.

This is especially painful on the first scan, where all repositories may be large and there is no previous timing baseline. A local-first architecture tool must make long-running deterministic work visible without forcing the user to inspect SQLite, Task Manager, or guess from disk activity.

A related reliability issue appeared during manual interruption: pressing `Ctrl+C` can leave the current scan persisted as `running`. That makes later `doctor`/`status` output misleading because the scan is no longer running.

## Goals

- Show scan progress by default for human `relascope scan` runs.
- Provide a `--silent` flag to suppress progress output.
- Use terminal-friendly heartbeat output rather than an exact percentage bar for this slice.
- Show useful progress signals while scanning:
  - current repository;
  - files indexed or seen;
  - files skipped by exclusions when practical;
  - elapsed time;
  - periodic reassurance that the scan is still active.
- Keep `relascope scan --format json` stdout as valid JSON with no human progress mixed into it.
- Mark interrupted scans as `cancelled` instead of leaving them as `running` when `Ctrl+C` is received and cancellation can be handled safely.
- Make `doctor` and `status` able to report `cancelled` as a persisted last-scan status.
- Preserve existing scan, graph, import, stale marking, JSON, and smoke behavior.

## Non-Goals

- No exact percentage progress bar in this slice.
- No mandatory pre-count pass over all repositories before scanning.
- No file watcher.
- No scan resume.
- No background scan daemon.
- No parallel scanning.
- No Tree-sitter.
- No cross-repository resolution.
- No impact analysis.
- No stable machine-readable progress event protocol yet.
- No terminal UI framework dependency unless implementation proves it is small and justified.

## Key Product Decisions

- Progress is enabled by default for human scan output.
- `--silent` disables progress output.
- Heartbeat progress is preferred over exact percentage progress because exact totals may require a costly pre-walk on large repositories.
- Terminal behavior should be robust in PowerShell and ordinary terminals. Prefer clear, low-risk output over a fragile animated progress bar.
- `scan --format json` must keep stdout clean JSON. If progress is emitted for JSON mode at all, it must not corrupt stdout; the default for this slice may silence progress in JSON mode.
- `Ctrl+C` should print a short cancellation message when practical and persist scan status as `cancelled`.
- `cancelled` becomes a valid scan lifecycle status alongside existing `running` and `completed` states.

## User-Facing Behavior

Default human scan:

```bash
relascope scan
```

Expected shape:

```text
Scanning repositories...
  backend: 1,250 files indexed, 430 skipped, elapsed 00:00:12
  app: 3,800 files indexed, 2,100 skipped, elapsed 00:00:31
```

The exact formatting can evolve during design, but the command must communicate that work is actively progressing.

Silent human scan:

```bash
relascope scan --silent
```

Suppresses progress heartbeat and keeps final scan summary behavior.

JSON scan:

```bash
relascope scan --format json
```

Prints valid JSON to stdout without human progress lines mixed in.

Interrupted scan:

```text
Scanning repositories...
  backend: 12,000 files indexed, 4,200 skipped, elapsed 00:01:44
^C
Scan cancelled
```

After cancellation, `doctor` or `status` should not misleadingly imply that the scan is still running.

## Acceptance Summary

Relascope 0.0.3-c is acceptable when a user scanning a large local workspace can see ongoing progress by default, can opt out with `--silent`, can still rely on clean JSON output for `scan --format json`, and can interrupt a scan without leaving the persisted last scan stuck as `running`.

Automated coverage should prove:

1. human scan still completes and prints the final summary;
2. progress can be disabled with `--silent`;
3. JSON scan output remains valid JSON;
4. cancelled scans persist status `cancelled` when interruption is handled;
5. existing graph/import/incremental smoke behavior remains valid.

## Review Workload Note

This change touches CLI scan options, scan loop reporting, storage scan lifecycle status, cancellation handling, tests, smoke behavior, and documentation. Keep the slice focused on visibility and cancellation correctness. If implementation grows too large, split into:

1. heartbeat progress for human scans;
2. `--silent` and JSON cleanliness;
3. cancelled scan state.
