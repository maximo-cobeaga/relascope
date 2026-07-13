# Verify Report: Relascope 0.0.3-a Incremental Scan State and Stale Graph Marking

## Status

Verified.

## Verification Commands

The user confirmed the following commands passed from PowerShell in `C:\Users\MAXIMO\Desktop\relascope`:

```powershell
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

The Pi harness shell still cannot execute Cargo directly, so command evidence is user-provided.

## Verified Behavior

The automated smoke script validates the incremental graph flow against a copied mutable fixture, not the checked-in fixture.

### First scan

Expected and verified:

```text
files: 12
added: 12
modified: 0
removed: 0
```

### Unchanged second scan

Expected and verified:

```text
files: 12
added: 0
modified: 0
removed: 0
```

### Modified copied `api-python/app.py`

The smoke script removes `import os` from the copied fixture and verifies:

```text
files: 12
added: 0
modified: 1
removed: 0
```

Expected stale import:

```text
api/app.py -> api:os [stale]
```

Expected current import remains active:

```text
api/app.py -> api/settings.py [active]
```

### Removed copied `api-python/settings.py`

The smoke script removes the copied target file and verifies:

```text
files: 11
added: 0
modified: 0
removed: 1
```

Expected stale/unverified import behavior:

```text
api/app.py -> api/settings.py [stale]
api/app.py -> api:.settings [unverified]
```

## Acceptance Coverage

- Current scan is compared against previous completed scan.
- First scan reports all files as added.
- Unchanged scan reports zero added/modified/removed.
- Modified file reports modified count.
- Removed file reports removed count.
- Removed/modified import facts can become `stale`.
- Current unresolved imports can be materialized as `unverified`.
- Graph facts are not physically deleted.
- Smoke script mutates copied fixtures, not checked-in fixtures.
- Existing inventory, graph summary, and graph imports behavior remains covered by `cargo test` and smoke.

## Scope Boundaries Confirmed

This verification does not include, and the implementation intentionally does not add:

- watcher
- rename intelligence
- Tree-sitter
- package resolution
- cross-repository resolution
- impact analysis
- AI
- MCP
- web UI
- desktop UI

## Conclusion

Relascope 0.0.3-a satisfies the approved OpenSpec scope and is ready for sync/archive.
