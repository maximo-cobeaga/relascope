# Handoff: Relascope 0.0.2-b Shallow Module and Import Detection

## Current Status

Implementation is applied. Verification is pending user-side because the Pi harness shell cannot execute Cargo.

This change is **not archived yet**. Do not sync/archive until verification passes.

## Implemented

- Shallow import detector in `crates/core/src/imports.rs`.
- Graph constants and deterministic IDs for:
  - `Module`
  - `IMPORTS`
  - `unverified`
  - module nodes
  - unresolved module nodes
  - import edges
- Storage materialization for:
  - source `Module` nodes
  - `File CONTAINS Module` edges
  - unresolved target `Module` nodes
  - `IMPORTS` edges
  - line-level import evidence
- CLI command:
  - `relascope graph imports`
- Fixture updates with real imports:
  - `fixtures/polyrepo-basic/api-python/app.py`
  - `fixtures/polyrepo-basic/api-python/settings.py`
  - `fixtures/polyrepo-basic/web-typescript/src/main.ts`
  - `fixtures/polyrepo-basic/web-typescript/src/message.ts`
- Smoke script updated:
  - `scripts/smoke-graph.ps1`
- Documentation added/updated:
  - `docs/adr/0006-shallow-line-based-import-detection.md`
  - `tutorial/onboarding-relascope-0-0-2-b.md`
  - `README.md`
  - older onboarding fixture counts updated for current fixture state

## Expected Current Fixture Counts

After scanning `fixtures/polyrepo-basic`:

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

Expected imports:

```text
api/app.py -> api/settings.py [active]
api/app.py -> api:os [unverified]
web/src/main.ts -> web/src/message.ts [active]
```

The TypeScript `import` and `export-from` lines both point to `./message`; they intentionally collapse into one `IMPORTS` edge under the current edge uniqueness rule, while preserving multiple evidence rows.

## Verification To Run Next Session

From PowerShell:

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

If tests fail, fix compile/test issues first. If all pass:

1. Update `verify-report.md`.
2. Mark verification tasks complete in `tasks.md`.
3. Sync spec into `openspec/specs/shallow-module-imports/spec.md`.
4. Archive the change under `openspec/changes/archive/`.

## Scope Boundaries Still Active

Do not add these in this change:

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
