# Smoke Test Report: Relascope 0.0.1 Inventory

## Status

Passed.

## Environment

User executed the smoke test from PowerShell on Windows.

## Commands Covered

- `relascope init <path>`
- `relascope repo add <path> --id <id>`
- `relascope status`
- `relascope scan`

## Evidence Summary

Workspace initialized at:

```text
C:\Users\MAXIMO\Desktop\relascope-smoke
```

Workspace ID persisted:

```text
5cc3bed0-49f7-4ae0-bc48-fe2bd98e8165
```

Repositories added:

- `api` → `fixtures/polyrepo-basic/api-python`
- `web` → `fixtures/polyrepo-basic/web-typescript`
- `infra` → `fixtures/polyrepo-basic/infra-config`

Initial status before scan showed:

- `repositories: 3`
- all repositories `available`
- `last_scan: none`

First scan result:

- status: completed
- files: 10
- by kind:
  - code: 2
  - configuration: 4
  - documentation: 2
  - unknown: 2
- by language:
  - json: 1
  - markdown: 2
  - python: 1
  - typescript: 1
  - yaml: 1

Persistence confirmed:

- `.relascope/graph.db` was created.
- `status` after scan reported the same workspace ID and repository internal IDs.

No-rescan behavior confirmed:

- `api-python` fixture directory was renamed away.
- `relascope status` still reported `api` as `available`, proving status did not rescan.

Missing repository behavior confirmed:

- A subsequent `relascope scan` reported:
  - files: 6
  - unavailable repositories: `api`
- A subsequent `relascope status` reported `api` as `unavailable` and preserved the repository record.

Fixture was restored after the test.

## Cleanup Note

Removing `C:\Users\MAXIMO\Desktop\relascope-smoke` failed while PowerShell was still inside that directory. The user should change directory out of the smoke workspace before deleting it.

## Conclusion

Manual smoke testing confirms the core Relascope 0.0.1 inventory workflow works end-to-end beyond automated tests.
