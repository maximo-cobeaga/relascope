# Test Plan: Real Workspace Dogfooding

## Purpose

This test plan records the manual dogfooding flow for Relascope after `0.0.3-b`. It complements the active specification in `spec.md`.

## Preconditions

Run from the Relascope repository root:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
cargo build -p relascope-cli
```

All must pass before testing a real workspace.

## Manual Real Workspace Flow

1. Create a dogfood workspace outside source repositories.
2. Run `relascope init <workspace>`.
3. Add real local repositories with stable visible IDs.
4. Run `relascope repo list`.
5. Run `relascope doctor` before scan.
6. Run `relascope scan`.
7. Run `relascope status`.
8. Run `relascope doctor` after scan.
9. Run `relascope graph summary`.
10. Run `relascope graph imports`.
11. Export JSON evidence:
    - `relascope scan --format json > scan.json`
    - `relascope graph summary --format json > graph-summary.json`
    - `relascope graph imports --format json > graph-imports.json`
12. Make a safe import modification in a disposable branch/copy.
13. Re-run `relascope scan` and verify `modified` count.
14. Verify removed imports become `stale` when appropriate.
15. Remove a safe file in a disposable branch/copy.
16. Re-run `relascope scan` and verify `removed` count.
17. Test `relascope repo remove <id>` and confirm files remain on disk.

## Evidence To Capture

- Terminal output from `doctor` before and after scan.
- `scan.json`.
- `graph-summary.json`.
- `graph-imports.json`.
- Notes for false positives, false negatives, unresolved imports, stale facts, slow scans, and confusing output.

## Pass Criteria

- Commands complete without crashing.
- Repository registration is understandable.
- Scan output includes plausible file and incremental counts.
- Doctor output is useful and read-only.
- Graph summary includes plausible node/edge counts.
- Graph imports include evidence lines.
- JSON output is valid and useful for local evidence capture.
- Repo removal does not delete repository files.

## Known Limitations

- JSON schema is experimental.
- Import detection is shallow and line-based.
- No Tree-sitter.
- No watcher.
- No cross-repository resolution.
- No impact analysis.
