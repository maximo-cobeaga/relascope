# Tasks: Relascope 0.0.3-b Real Workspace Dogfooding

## Implementation Plan

Keep this slice focused on operational CLI usability for real local workspace dogfooding. Do not add watcher, Tree-sitter, deeper semantic parsing, cross-repository resolution, impact analysis, AI, MCP, web UI, or desktop UI.

## 1. CLI output format foundation

- [x] Add `OutputFormat` enum with `human` and `json` values.
- [x] Add reusable `FormatArgs` or command-local format args.
- [x] Apply format args to `scan`, `graph summary`, and `graph imports`.
- [x] Keep human output as default.
- [x] Reject unsupported format values through `clap`.

## 2. JSON output models

- [x] Add serializable response for scan output.
- [x] Reuse or wrap `GraphSummary` for graph summary JSON.
- [x] Ensure import edge read model is serializable for JSON.
- [x] Ensure JSON output prints only valid JSON, without extra human text.
- [x] Document JSON as experimental.

## 3. Repository list command

- [x] Add `relascope repo list` command parsing.
- [x] Implement human output for registered repositories.
- [x] Print clear empty-state message when no repositories are registered.
- [x] Ensure `repo list` does not scan or update availability.
- [x] Add CLI integration tests for empty and populated repo list.

## 4. Repository remove command

- [x] Add `relascope repo remove <id>` command parsing.
- [x] Add storage method to remove repository by visible ID within workspace.
- [x] Ensure command deletes only registration rows, not files from disk.
- [x] Return clear error for unknown repository ID.
- [x] Ensure removed repositories are absent from future `repo list` and scans.
- [x] Document that historical graph facts may remain until future cleanup.
- [x] Add storage and CLI integration tests.

## 5. Doctor read model

- [x] Define `DoctorReport` model.
- [x] Include workspace ID and name.
- [x] Include config status.
- [x] Include database status.
- [x] Include repository totals and path-existence availability summary.
- [x] Include last scan status or no-scan-yet state.
- [x] Include stale node count.
- [x] Include stale edge count.
- [x] Include unverified import count.
- [x] Include exclusion count or active exclusion list.
- [x] Ensure doctor does not scan file trees.
- [x] Ensure doctor does not update database availability state.

## 6. Doctor CLI command

- [x] Add `relascope doctor` command parsing.
- [x] Implement concise human output.
- [x] Exit successfully when warnings exist but command ran correctly.
- [x] Add CLI integration tests for healthy workspace.
- [x] Add CLI integration test proving doctor reports missing path without mutating stored availability.

## 7. JSON command support

- [x] Implement `relascope scan --format json`.
- [x] Implement `relascope graph summary --format json`.
- [x] Implement `relascope graph imports --format json`.
- [x] Add CLI tests that parse JSON output for each supported command.
- [x] Ensure human output remains unchanged enough for existing tests/smoke.

## 8. Smoke script updates

- [x] Add `repo list` call to smoke script.
- [x] Add `doctor` call to smoke script.
- [x] Add JSON scan output smoke check if practical.
- [x] Add JSON graph summary output smoke check if practical.
- [x] Add JSON graph imports output smoke check if practical.
- [x] Avoid making smoke output too noisy.

## 9. Dogfooding documentation

- [x] Add `tutorial/dogfooding-real-workspace.md`.
- [x] Explain how to create a dogfood workspace outside source repos.
- [x] Explain how to add real local repositories.
- [x] Explain how to run `repo list`, `scan`, `doctor`, `graph summary`, and `graph imports`.
- [x] Explain how to save JSON evidence to files.
- [x] Include a dogfooding findings template.
- [x] Include what to record: false positives, false negatives, unresolved imports, stale facts, slow scans, confusing output.
- [x] Link guide from README.

## 10. Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run `pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1`.
- [x] Verify `repo list` works before and after adding repos.
- [x] Verify `repo remove` removes registration but does not delete files.
- [x] Verify `doctor` reports useful health state.
- [x] Verify JSON outputs are valid JSON.
- [x] Verify existing inventory, graph, import, incremental, and stale behavior still works.

## Review Workload Forecast

This is medium-sized and mostly operational. It touches CLI parsing, output formatting, storage read models, docs, and tests. If implementation grows too much, split into:

1. `repo list/remove` and doctor.
2. JSON output support.
3. dogfooding guide and smoke updates.
