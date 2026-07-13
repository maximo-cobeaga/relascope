# Change Proposal: Relascope 0.0.3-b Real Workspace Dogfooding

## Status

Draft for approval before specification, design, tasks, and implementation.

## Problem Statement

Relascope now has a functional local engine for workspace inventory, graph materialization, shallow import detection, evidence, incremental scan summaries, and stale graph marking. However, using it on a real project still requires too much manual interpretation and lacks basic operational commands needed for dogfooding.

Before adding a watcher, Tree-sitter, cross-repository resolution, or impact analysis, Relascope needs a practical dogfooding slice: commands that help a real user inspect workspace state, repository registration, graph outputs, and health signals without opening SQLite or reading internal implementation details.

## Goals

- Make Relascope usable on a real local workspace for internal dogfooding.
- Add repository inspection command:
  - `relascope repo list`
- Add repository removal command:
  - `relascope repo remove <id>`
- Add workspace health command:
  - `relascope doctor`
- Add experimental JSON output for key commands:
  - `relascope scan --format json`
  - `relascope graph summary --format json`
  - `relascope graph imports --format json`
- Keep existing human output as the default.
- Document JSON output as experimental, not a stable public API.
- Add a dogfooding guide for running Relascope on a real workspace such as ReservApp while keeping tests fixture-based and generic.
- Capture useful health signals:
  - config existence;
  - database open/migration success;
  - registered repository availability;
  - last scan status;
  - stale graph fact counts;
  - unverified import counts;
  - active exclusions.

## Non-Goals

- No watcher.
- No Tree-sitter.
- No deeper semantic parsing.
- No cross-repository import resolution.
- No impact analysis.
- No web UI.
- No desktop UI.
- No stable API contract for JSON output yet.
- No cloud/team collaboration.
- No automatic mutation of registered repositories.

## User-Facing Commands

Existing commands remain:

```bash
relascope init [path]
relascope repo add <path> [--id <id>]
relascope scan
relascope status
relascope graph summary
relascope graph imports
```

New or extended commands:

```bash
relascope repo list
relascope repo remove <id>
relascope doctor
relascope scan --format human
relascope scan --format json
relascope graph summary --format human
relascope graph summary --format json
relascope graph imports --format human
relascope graph imports --format json
```

`human` remains the default format.

## Key Decisions

- Dogfooding target can be ReservApp or another real local workspace, but tests must remain generic and fixture-based.
- JSON output is experimental and documented as subject to change.
- `doctor` reports health; it does not modify repositories or run deeper analysis.
- `repo remove <id>` removes the repository registration from the workspace but does not delete files from disk.
- Existing graph facts for a removed repository may be marked stale or left historical according to design, but the command must never delete the actual repository directory.

## Acceptance Summary

Relascope 0.0.3-b is acceptable when a user can run it on a real local workspace and perform the basic dogfooding loop:

1. initialize workspace;
2. add repositories;
3. list repositories;
4. scan;
5. inspect status, graph summary, and imports;
6. run doctor;
7. export scan/graph/import outputs as JSON;
8. remove a repository registration without deleting files;
9. follow a dogfooding guide to record findings and friction.

Automated tests must cover the new commands and JSON paths with the generic fixture.

## Review Workload Note

This change is mostly CLI and operational UX, but it touches storage read models, command parsing, output formatting, docs, and tests. Keep JSON experimental and avoid over-designing a public API schema. If implementation grows, split into:

1. repository list/remove + doctor;
2. JSON format support;
3. dogfooding guide and smoke updates.
