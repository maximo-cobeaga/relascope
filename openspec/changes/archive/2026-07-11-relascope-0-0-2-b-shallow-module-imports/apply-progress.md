# Apply Progress: Relascope 0.0.2-b Shallow Module and Import Detection

## Status

Implementation applied and verified. Ready for OpenSpec sync/archive.

## Implemented

- Added shallow import detection in `crates/core/src/imports.rs`.
- Added graph constants and deterministic IDs for:
  - `Module`
  - `IMPORTS`
  - `unverified`
  - source module nodes
  - unresolved module nodes
  - import edges
- Added line-based Python detection for:
  - `import x`
  - `import x, y`
  - `import x as alias`
  - `from x import y`
  - relative specifiers such as `.settings`
- Added line-based TypeScript/JavaScript detection for:
  - `import ... from "specifier"`
  - side-effect imports
  - `export ... from "specifier"`
- Added simple local resolution for TypeScript/JavaScript relative imports.
- Added conservative Python local resolution for simple dotted/relative imports.
- Extended graph materialization to create:
  - source `Module` nodes
  - `File CONTAINS Module` edges
  - unresolved target `Module` nodes
  - `IMPORTS` edges
  - line-level import evidence
- Added storage read model for persisted imports.
- Added CLI command:
  - `relascope graph imports`
- Updated fixtures with import examples.
- Updated automated smoke script to verify module/import counts and import evidence.
- Added ADR and tutorial for shallow imports.
- Updated README and existing tutorials for current fixture counts.
- Added/updated unit, storage, and CLI integration tests.

## Verification Evidence

User confirmed:

```powershell
cargo fmt
cargo test
```

Result: passed.

User also ran:

```powershell
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Result: passed.

Verified counts:

```text
files: 12
nodes: 21
File: 12
Module: 5
Repository: 3
Workspace: 1
edges: 22
CONTAINS: 19
IMPORTS: 3
```

Verified imports:

```text
api/app.py -> api/settings.py [active]
api/app.py -> api:os [unverified]
web/src/main.ts -> web/src/message.ts [active]
```

See `verify-report.md` and `smoke-test-report.md` for full evidence.

## Scope Preserved

Still intentionally out of scope:

- Tree-sitter
- full parser semantics
- symbol/function/class/interface nodes
- TSConfig alias resolution
- package resolution
- cross-repository resolution
- impact analysis
- AI
- MCP
- web UI
- desktop UI

## Remaining Verification

None for the approved 0.0.2-b shallow imports scope.
