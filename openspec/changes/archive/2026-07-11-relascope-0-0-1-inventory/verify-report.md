# Verify Report: Relascope 0.0.1 Inventory

## Status

Verified.

## Verification Commands

The user ran from PowerShell in `C:\Users\MAXIMO\Desktop\relascope`:

```powershell
cargo fmt
cargo test
```

## Result

`cargo fmt` completed successfully with no reported output.

`cargo test` completed successfully.

## Test Evidence

- `relascope-cli` unit test binary: 0 tests, passed.
- CLI integration tests: 3 passed.
  - `rejects_duplicate_visible_id`
  - `init_repo_scan_status_round_trip`
  - `status_does_not_rescan_missing_repository`
- `relascope-core` unit tests: 5 passed.
  - `repository::tests::slugifies_folder_names`
  - `classify::tests::classifies_common_languages_and_kinds`
  - `repository::tests::validates_visible_ids`
  - `workspace::tests::finds_workspace_from_child_directory`
  - `scan::tests::scans_unknown_files_and_skips_defaults`
- `relascope-storage-sqlite` unit tests: 1 passed.
  - `tests::persists_repository_and_status`
- Doc-tests:
  - `relascope_core`: 0 tests, passed.
  - `relascope_storage_sqlite`: 0 tests, passed.

Total explicit tests: 9 passed.

## Acceptance Coverage

- Workspace initialization is covered by CLI integration test flow.
- Repository registration is covered by CLI integration tests and storage tests.
- Duplicate visible repository ID rejection is covered.
- Scan persists non-excluded files and retains unknown files.
- Default exclusions are covered by unit/integration tests.
- Status reads persisted state and does not rescan.
- Reopened storage preserves repository identity.
- Missing registered repository paths are marked unavailable on scan and not deleted.

## Limitations

- Verification evidence was produced in the user's PowerShell environment because the Pi harness shell did not have `cargo`, `rustc`, or `rustup` in PATH.
- Manual CLI smoke testing against the checked-in `fixtures/polyrepo-basic` is still optional because the CLI integration tests create equivalent temporary fixture flows.

## Conclusion

Relascope 0.0.1 Inventory satisfies the approved OpenSpec scope for implementation-level verification.
