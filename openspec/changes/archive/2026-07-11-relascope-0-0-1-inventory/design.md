# Design: Relascope 0.0.1 Inventory

## Overview

Relascope 0.0.1 establishes the durable local foundation for a multi-repository workspace. The implementation should be intentionally small: a Rust workspace, a CLI, workspace configuration, SQLite persistence, repository registration, shallow file inventory, and status reporting from persisted state.

This design does not introduce graph semantics, AST parsing, AI, MCP, desktop, web UI, or impact analysis.

## Architecture Slice

```text
CLI commands
  ├─ init
  ├─ repo add
  ├─ scan
  └─ status
       │
       ▼
Core workspace services
  ├─ workspace initialization
  ├─ config loading/saving
  ├─ repository registration
  ├─ scan orchestration
  └─ status projection
       │
       ▼
Storage
  ├─ relascope.yaml
  └─ .relascope/graph.db
```

## Recommended Initial Repository Layout

```text
relascope/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── core/
│   ├── storage-sqlite/
│   └── cli/
├── fixtures/
│   └── polyrepo-basic/
│       ├── api-python/
│       ├── web-typescript/
│       └── infra-config/
├── docs/
│   └── adr/
└── relascope.yaml            # only inside initialized workspaces, not repo root by default
```

For 0.0.1, three crates are enough:

- `relascope-core`: workspace domain types, config, repository registration, scan logic, file classification.
- `relascope-storage-sqlite`: migrations and persistence repositories.
- `relascope-cli`: command parsing and user-facing output.

Additional crates from the master architecture should wait until their functionality is needed.

## CLI Contract

Use `clap` for command parsing.

```bash
relascope init [path]
relascope repo add <path> [--id <id>]
relascope scan
relascope status
```

The CLI should locate the workspace from the current directory for commands other than `init`. If no workspace is found, return a clear configuration error.

## Workspace Files

### `relascope.yaml`

Purpose: user-visible workspace configuration.

Initial shape:

```yaml
version: 1
workspace:
  id: "<uuid>"
  name: "<folder-name>"
analysis:
  exclude:
    - "**/.git/**"
    - "**/node_modules/**"
    - "**/.venv/**"
    - "**/dist/**"
    - "**/build/**"
```

Repository records may be stored in SQLite as source of truth for 0.0.1. If repository entries are also mirrored into YAML later, that should be a separate decision because it affects conflict handling.

### `.relascope/graph.db`

Although named `graph.db` to preserve the product convention, 0.0.1 stores inventory tables only. Graph edges are out of scope.

## Persistence Model

Use SQLite with migrations executed on workspace initialization and open.

Initial logical tables:

```sql
workspaces(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

repositories(
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  visible_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  path TEXT NOT NULL,
  path_kind TEXT NOT NULL,
  availability TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(workspace_id, visible_id)
);

scans(
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  status TEXT NOT NULL,
  summary_json TEXT NOT NULL
);

files(
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  repository_id TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  kind TEXT NOT NULL,
  language TEXT,
  size_bytes INTEGER,
  content_hash TEXT,
  modified_at TEXT,
  scan_id TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  UNIQUE(repository_id, relative_path, scan_id)
);
```

Potential future evolution:

- Move from per-scan file rows to current-state plus history if storage grows too quickly.
- Add node/edge/evidence tables in 0.0.2+ when graph extraction begins.
- Add relative path anchors or repository origin metadata to improve workspace portability.

## Identity Model

### Workspace

- Internal UUID generated at `init`.
- Persisted in both `relascope.yaml` and SQLite.
- Must remain stable across processes.

### Repository

- Internal UUID generated at `repo add`.
- Visible ID generated from folder name slug or provided by `--id`.
- Visible ID is unique per workspace.
- Canonical absolute path is persisted for 0.0.1 operation.
- Filesystem path is not repository identity.

### File Inventory

For 0.0.1, file identity can be repository ID + relative path + scan ID. Stable file identity across renames is explicitly out of scope until graph/incremental work begins.

## Path Handling

- Convert input repository paths to canonical absolute paths for validation and walking.
- Store canonical path in SQLite.
- Store `path_kind = "canonical-local"` or equivalent metadata to make the limitation explicit.
- Document that relocating a workspace or repositories may require reconfiguration in 0.0.1.
- Do not design path as primary identity.

## Repository Availability

Availability values:

- `available`
- `unavailable`

`repo add` rejects paths that do not exist. Later, if a registered path disappears, `scan` and `status` mark/report the repository as unavailable without deleting it.

## Scan Behavior

`scan` performs shallow inventory only:

1. Load workspace config and repositories.
2. Create a scan record with `running` status.
3. For each repository:
   - Check path availability.
   - If unavailable, update repository availability and continue.
   - Walk files recursively with exclusion matching.
   - Classify each non-excluded file.
   - Persist inventory rows transactionally.
4. Finish scan with counts by repository, kind, language, and unavailable repositories.

The scan must not:

- Execute repository code.
- Run package managers.
- Access network.
- Call AI models.
- Interpret repository documentation as instructions.

## Exclusion Strategy

Start with simple glob-like patterns using a Rust crate such as `ignore` or `globset`.

Default exclusions:

```text
**/.git/**
**/node_modules/**
**/.venv/**
**/dist/**
**/build/**
**/target/**
**/.relascope/**
```

The master document mentions `.git`, `node_modules`, `.venv`, `dist`, `build`, and heavy binaries. Adding `target` and `.relascope` avoids scanning Rust build output and Relascope internals.

Binary/heavy handling for 0.0.1:

- Avoid reading entire large files into memory.
- Use metadata size first.
- Hash only files below a conservative configurable threshold, or stream hashes.
- Classify obvious binary extensions as `binary`.
- Unknown non-binary extensions should remain inventory records, not errors.

## File Classification

Initial `kind` values:

- `code`
- `configuration`
- `documentation`
- `binary`
- `generated`
- `unknown`

Initial language detection may use extension mapping only:

- `.py` → `python`
- `.ts`, `.tsx` → `typescript`
- `.js`, `.jsx` → `javascript`
- `.json` → `json`
- `.yaml`, `.yml` → `yaml`
- `.toml` → `toml`
- `.md`, `.mdx` → `markdown`
- `.dockerfile`, `Dockerfile` → `dockerfile`

This is intentionally shallow and should not imply semantic support.

## Status Behavior

`relascope status` reads persisted state only. It reports:

- Workspace ID and name.
- Workspace root.
- Repository count.
- Repository visible IDs, internal IDs, paths, and availability.
- Last scan status and timestamp.
- Last scan counts from persisted summary.
- A clear message if no scan has been run.

It must not walk repositories or refresh inventory.

## Fixture Design

`fixtures/polyrepo-basic/` should include:

```text
api-python/
  app.py
  requirements.txt
  README.md
  data.unknownext
  .venv/ignored.py

web-typescript/
  src/main.ts
  package.json
  README.md
  node_modules/ignored.js
  dist/bundle.js

infra-config/
  docker-compose.yml
  nginx.conf
  notes.txt
  build/generated.txt
```

The fixture should verify:

- Multiple repositories can be added.
- Slug IDs can be generated.
- Explicit IDs can be used.
- Python and TypeScript are recognized shallowly.
- Config and documentation files are classified.
- Unknown files are retained.
- Excluded directories are skipped.
- Status survives process reopening.

## Testing Strategy

After the Rust workspace is created, use `cargo test` as the baseline test command.

Test categories for 0.0.1:

- Unit tests for slug generation, ID validation, exclusion matching, and file classification.
- Storage tests for migrations, repository uniqueness, scan persistence, and status projection.
- CLI integration tests for `init`, `repo add`, `scan`, and `status` using temp directories and fixtures.
- Regression test for missing repository paths remaining registered as unavailable.

Strict TDD is not currently enabled in `openspec/config.yaml`, but implementation should still add tests alongside behavior.

## ADRs Required

Create initial ADRs during implementation:

1. Rust CLI and workspace foundation.
2. SQLite local persistence and `.relascope/graph.db` naming.
3. Local-only repository registration for 0.0.1.
4. Canonical path persistence limitation and future portability direction.

## Tradeoffs

### Store repositories in SQLite first

Pros:

- Easier uniqueness enforcement.
- Keeps status and scan queries centralized.
- Avoids YAML merge/conflict complexity in the first slice.

Cons:

- Less transparent to users than a fully YAML-declared workspace.
- Future sync/edit flows may need a config migration.

Decision: acceptable for 0.0.1 because the milestone is durable local operation, not collaborative config editing.

### Canonical paths in 0.0.1

Pros:

- Simple and reliable for local operation.
- Avoids premature path abstraction.

Cons:

- Workspace relocation is limited.
- Cross-machine portability is not solved.

Decision: document the limitation and keep identity independent from paths.

### Shallow extension-based classification

Pros:

- Fast and deterministic.
- No semantic parsing dependency.
- Enough to prove heterogeneous inventory.

Cons:

- Low accuracy for complex projects.
- Does not establish real graph knowledge yet.

Decision: acceptable for 0.0.1; semantic analyzers begin later.

## Risks

- Scope creep into graph extraction or semantic parsing.
- Over-designing portability before inventory is proven.
- Treating unknown files as failures instead of inventory records.
- Accidentally scanning build outputs or dependency directories.
- Status implementation accidentally re-scanning and hiding persistence bugs.
