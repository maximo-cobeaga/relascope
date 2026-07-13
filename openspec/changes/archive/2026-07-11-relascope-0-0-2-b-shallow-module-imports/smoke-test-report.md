# Smoke Test Report: Relascope 0.0.2-b Shallow Module and Import Detection

## Status

Passed.

## Environment

User executed the automated smoke test from PowerShell 7.6.3 on Windows.

Command:

```powershell
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

## Script Behavior

The script:

1. Built the Relascope CLI with `cargo build --manifest-path ... -p relascope-cli`.
2. Created a temporary smoke workspace.
3. Ran `relascope init`.
4. Registered fixture repositories:
   - `api`
   - `web`
   - `infra`
5. Confirmed `graph summary` before scan reported no materialized graph.
6. Confirmed `graph imports` before scan reported no materialized imports.
7. Ran `scan`.
8. Verified graph summary counts.
9. Verified graph import evidence.
10. Ran a second `scan`.
11. Verified graph counts remained stable.

## Evidence

First scan:

```text
Scan completed
  files: 12
```

Graph summary after first scan:

```text
Graph summary
  nodes: 21
  by_kind:
    File: 12
    Module: 5
    Repository: 3
    Workspace: 1
  edges: 22
  by_relation:
    CONTAINS: 19
    IMPORTS: 3
```

Graph imports after first scan:

```text
Graph imports
  api/app.py -> api/settings.py [active]
    evidence: app.py:2 from .settings import CONFIG
  api/app.py -> api:os [unverified]
    evidence: app.py:1 import os
  web/src/main.ts -> web/src/message.ts [active]
    evidence: src/main.ts:1 import { message } from "./message";
  web/src/main.ts -> web/src/message.ts [active]
    evidence: src/main.ts:2 export { message } from "./message";
```

Second scan:

```text
Scan completed
  files: 12
```

Graph summary after second scan:

```text
Graph summary
  nodes: 21
  by_kind:
    File: 12
    Module: 5
    Repository: 3
    Workspace: 1
  edges: 22
  by_relation:
    CONTAINS: 19
    IMPORTS: 3
```

## Acceptance Coverage

Covered by this smoke test:

- `graph summary` before scan exits successfully.
- `graph imports` before scan exits successfully.
- `scan` materializes `Module` nodes.
- `scan` materializes `IMPORTS` edges.
- `graph imports` shows status and line-level evidence.
- Python relative import resolved to `api/settings.py`.
- Python external import `os` preserved as `unverified`.
- TypeScript import/export-from to `./message` resolved to `web/src/message.ts`.
- Repeated scan does not duplicate graph node or edge counts.

## Remaining Verification

Still required before archive:

```powershell
cargo fmt
cargo test
```

After those pass, create/update the final `verify-report.md`, mark remaining verification tasks complete, sync the active spec, and archive this change.
