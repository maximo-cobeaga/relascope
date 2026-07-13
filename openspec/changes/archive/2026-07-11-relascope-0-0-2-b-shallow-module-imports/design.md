# Design: Relascope 0.0.2-b Shallow Module and Import Detection

## Overview

Relascope 0.0.2-b adds the first dependency-oriented graph facts. It creates `Module` nodes for recognized code files and `IMPORTS` edges for simple Python and TypeScript/JavaScript import statements.

This slice remains intentionally shallow. It uses deterministic line-based detection, stores evidence, and preserves unresolved imports instead of trying to solve package resolution, TSConfig aliases, Python environments, or cross-repository matching.

## Current Foundation

Relascope already has:

- durable workspaces and repository registration;
- file inventory in `files`;
- minimal graph tables: `nodes`, `edges`, `evidence`;
- graph materialization during `scan`;
- `Workspace`, `Repository`, and `File` nodes;
- `CONTAINS` edges;
- `relascope graph summary`.

0.0.2-b adds:

- `Module` nodes;
- `IMPORTS` edges;
- import-line evidence;
- `relascope graph imports`.

## Architecture Slice

```text
relascope scan
  ├─ scan repositories
  ├─ persist file inventory
  ├─ materialize minimal graph
  │    ├─ Workspace / Repository / File nodes
  │    └─ CONTAINS edges
  └─ materialize shallow imports
       ├─ Module nodes for recognized code files
       ├─ unresolved target Module nodes when needed
       ├─ IMPORTS edges
       └─ line-level evidence

relascope graph imports
  └─ read persisted IMPORTS edges + evidence
```

## Core Domain Additions

Add graph constants:

```rust
NODE_KIND_MODULE = "Module"
RELATION_IMPORTS = "IMPORTS"
STATUS_UNVERIFIED = "unverified"
```

Add import detection structs in `relascope-core`, likely under a new module:

```text
crates/core/src/imports.rs
```

Suggested types:

```rust
pub struct ImportFact {
    pub source_relative_path: String,
    pub source_language: String,
    pub target_specifier: String,
    pub resolved_relative_path: Option<String>,
    pub line_number: u32,
    pub excerpt: String,
}
```

The detector should not know about SQLite. It should accept file content and metadata, then return import facts.

## Scan Input for Import Detection

Current file inventory records contain paths, kind, language, hashes, and scan IDs, but not file contents. Import detection needs to read source files during scan.

Design options:

1. Make `scan_repositories` parse imports while walking files.
2. Add a second pass after file inventory persistence that reads recognized code files.

Recommended for this slice: **second pass after inventory walk but before finishing scan**.

Why:

- Keeps existing inventory behavior stable.
- Avoids overloading `FileInventoryRecord`.
- Lets graph materialization use the already produced file list.
- Keeps import detection as an additive graph concern.

The second pass should:

1. Iterate current scan files.
2. Filter recognized code languages:
   - `python`
   - `typescript`
   - `javascript`
3. Read only files under available repository paths.
4. Apply max file-size guard reused from inventory/hash strategy where practical.
5. Produce import facts.
6. Materialize module/import graph facts.

## Module Node Identity

### Source modules

A source module represents a recognized code file.

Deterministic ID:

```text
node:module:<workspace_id>:<repository_id>:<hash(relative_path)>
```

Qualified name:

```text
<repo_visible_id>/<relative_path>
```

Display name:

```text
relative path or file stem
```

Metadata:

```json
{
  "source": "file_inventory",
  "module_role": "source",
  "language": "python",
  "relative_path": "app.py"
}
```

### Target modules

A target module may be resolved or unresolved.

If resolved to a known local file in the same repository, reuse that file's source module node ID.

If unresolved, create a deterministic unresolved module node ID:

```text
node:module-unresolved:<workspace_id>:<repository_id>:<hash(import_specifier)>
```

Qualified name:

```text
<repo_visible_id>:<specifier>
```

Metadata:

```json
{
  "source": "import_detection",
  "module_role": "unresolved_target",
  "specifier": "os",
  "resolution": "unresolved"
}
```

## File-to-Module Containment

Each source module should be connected to its file:

```text
File CONTAINS Module
```

Use the existing `CONTAINS` relation. This keeps ownership explicit without adding another relation type.

## Import Edge Identity

Use deterministic IDs based on:

```text
workspace_id + source_module_id + target_module_id + target_specifier + source_file_path + line_number
```

This prevents duplicates on repeated scans while allowing two separate import lines to remain distinct if they import the same target for different reasons.

Edge fields:

```text
relation_type: IMPORTS
origin_type: deterministic
origin_id: scan_id
confidence: 1.0 for detected import syntax
status: active if resolved locally, unverified if unresolved/external
```

Metadata:

```json
{
  "source": "shallow_import_detector",
  "specifier": "./types",
  "language": "typescript",
  "resolution": "resolved" | "unresolved"
}
```

## Evidence

Every `IMPORTS` edge should have evidence:

```text
repository_id: source repository UUID
file_path: source relative path
start_line: detected line number
end_line: detected line number
excerpt: raw import line trimmed
metadata_json: { "source": "shallow_import_detector", "scan_id": "..." }
```

Evidence ID should be deterministic for the same import line:

```text
evidence:import:<workspace_id>:<repository_id>:<source_relative_path>:<line_number>:<hash(excerpt)>
```

## Python Detector

Line-based rules:

1. Trim leading/trailing whitespace.
2. Skip empty lines.
3. Skip lines starting with `#`.
4. Match:
   - `import <module>`
   - `import <module> as <alias>`
   - `import <module>, <module2>` may be split into multiple facts if cheap; otherwise document first-token behavior.
   - `from <module> import <name>`
5. Preserve relative specifiers like `.models` and `..shared`.

Recommended initial behavior:

- For comma-separated `import a, b`, emit one fact per imported module.
- For `from x import y`, target specifier is `x`.

Do not parse multiline imports in this slice.

## TypeScript/JavaScript Detector

Line-based rules:

1. Trim whitespace.
2. Skip empty lines.
3. Skip lines starting with `//`.
4. Match import/export-from patterns:
   - `import ... from "specifier"`
   - `import ... from 'specifier'`
   - `import "specifier"`
   - `import 'specifier'`
   - `export ... from "specifier"`
   - `export ... from 'specifier'`
5. Target specifier is the quoted string.

Do not parse:

- multiline imports;
- dynamic `import()`;
- CommonJS `require()`;
- comments embedded after code except preserving line excerpt.

Those can be added later.

## Shallow Local Resolution

Resolution should be best-effort and local to the same repository.

### TypeScript/JavaScript relative imports

If specifier starts with `.`:

Try candidates relative to the source file directory:

```text
<specifier>
<specifier>.ts
<specifier>.tsx
<specifier>.js
<specifier>.jsx
<specifier>/index.ts
<specifier>/index.tsx
<specifier>/index.js
<specifier>/index.jsx
```

If a candidate exists in current scan file inventory, resolve to that file's module node.

### Python local imports

For this slice, keep resolution conservative.

Possible candidates:

- dotted absolute import: `app.models` → `app/models.py` or `app/models/__init__.py`
- relative import `.models` from `app/views.py` → `app/models.py` or `app/models/__init__.py`

If matching is not straightforward, leave unresolved.

Do not inspect Python path, virtualenvs, installed packages, or package metadata.

## Storage Additions

Existing `nodes`, `edges`, and `evidence` tables are sufficient.

Add storage methods:

```rust
materialize_shallow_imports(...)
list_import_edges(...)
```

`list_import_edges` should join:

- `edges` where `relation_type = 'IMPORTS'`;
- source node display/qualified name;
- target node display/qualified name;
- evidence file path and line;
- edge status.

Suggested read model:

```rust
pub struct ImportEdgeView {
    pub source: String,
    pub target: String,
    pub status: String,
    pub file_path: Option<String>,
    pub line: Option<i64>,
    pub excerpt: Option<String>,
}
```

## CLI: `relascope graph imports`

Add command:

```bash
relascope graph imports
```

Suggested output:

```text
Graph imports
  web/src/main.ts -> ./types [unverified]
    evidence: src/main.ts:1 import { User } from "./types";
```

If no imports:

```text
Graph imports
  no imports have been materialized yet
```

The command must not scan.

## Fixture Updates

Current fixture has simple files with no imports. Add minimal import examples:

### `fixtures/polyrepo-basic/api-python/app.py`

```python
import os
from .settings import CONFIG
```

Add:

```text
fixtures/polyrepo-basic/api-python/settings.py
```

### `fixtures/polyrepo-basic/web-typescript/src/main.ts`

```ts
import { message } from "./message";
export { message } from "./message";
```

Add:

```text
fixtures/polyrepo-basic/web-typescript/src/message.ts
```

This will change expected fixture counts from 0.0.2-a. Update docs/tests accordingly.

Expected additional source modules:

- `api-python/app.py`
- `api-python/settings.py`
- `web-typescript/src/main.ts`
- `web-typescript/src/message.ts`

Expected import edges:

- `app.py -> os` unresolved
- `app.py -> .settings` resolved if implemented conservatively enough; otherwise unverified
- `main.ts -> ./message` resolved
- `main.ts -> ./message` from export-from may produce a second edge if line-distinct edge identity is used

Because line-distinct edge identity is recommended, expected `IMPORTS` edges: 4.

## Graph Summary Impact

After fixture update and scan, graph summary should include:

```text
Module: 6? or 5? depending unresolved target deduplication
IMPORTS: 4
```

Clarification:

- Source modules: 4.
- Unresolved target `os`: 1.
- If `.settings` resolves to `settings.py`, no extra unresolved node.
- `./message` resolves to `message.ts`, no extra unresolved node.

Expected Module nodes if `.settings` and `./message` resolve: 5.

If `.settings` remains unresolved, expected Module nodes: 6.

Design target: resolve `./message`; allow `.settings` to be unresolved if Python relative resolution becomes risky. Tests should assert specific behavior chosen during implementation.

## Testing Strategy

Unit tests:

- Python detector ignores comments.
- Python detector emits `import` facts.
- Python detector emits `from` facts.
- TS/JS detector ignores comments.
- TS/JS detector detects named/default/side-effect/export-from imports.
- Resolver handles simple TS relative imports.
- Deterministic module/import IDs are stable.

Storage tests:

- Materializing imports creates module nodes and import edges.
- Repeated materialization does not duplicate import edges.
- Import evidence stores line and excerpt.
- Listing imports returns readable rows.

CLI integration tests:

- `graph imports` before scan reports no imports.
- After scan, `graph imports` prints detected imports.
- Repeated scan does not duplicate `IMPORTS` counts.
- Existing 0.0.1 and 0.0.2-a tests still pass after fixture count updates.

## Tradeoffs

### Regex/line-based detection first

Pros:

- Fast to implement.
- Deterministic.
- Easy to test.
- No native parser dependency yet.

Cons:

- Misses multiline/dynamic/complex imports.
- Can produce false positives or miss edge cases.

Decision: acceptable for 0.0.2-b. Tree-sitter belongs to a later analyzer milestone.

### Preserve unresolved imports

Pros:

- Keeps useful dependency intent.
- Makes missing resolution visible.
- Avoids false confidence.

Cons:

- Graph may include external/unverified modules.
- Users must understand status.

Decision: preserve with `unverified` status and evidence.

### Line-distinct import edges

Pros:

- Evidence maps cleanly to each import line.
- Re-export and import lines can both be represented.

Cons:

- Multiple edges may connect same source and target.
- Existing edge uniqueness on `(workspace_id, source, target, relation_type)` would collapse them.

Decision: for 0.0.2-b, either add import-line metadata to target node identity or adjust edge uniqueness. Preferred: update edge uniqueness strategy carefully to allow line-distinct import edges without breaking `CONTAINS` idempotency.

## Edge Uniqueness Adjustment

Current `edges` has uniqueness:

```sql
UNIQUE(workspace_id, source_node_id, target_node_id, relation_type)
```

That prevents two import lines from same module to same target.

Options:

1. Keep uniqueness and collapse duplicate source-target imports.
2. Change uniqueness to edge `id` only and rely on deterministic IDs.
3. Add `origin_id` to uniqueness.

Recommended for 0.0.2-b: keep current uniqueness for now and collapse same source-target imports. This is simpler and avoids migration complexity. Evidence upsert can keep the latest or first evidence. If line-distinct edges become necessary later, change schema in a dedicated migration.

Therefore fixture expected `IMPORTS` may be 3 instead of 4 if import and export-from target the same module.

Tests should assert collapsed behavior for this slice.

## Risks

- Scope creep into real parser work.
- Incorrect expected counts due to fixture changes.
- Import targets creating too many unresolved duplicate nodes.
- Confusing unresolved external packages with local missing files.
- Accidentally making `graph imports` scan the workspace.
- Breaking existing `CONTAINS` graph idempotency.

## Open Questions

None blocking. The design intentionally accepts shallow detection and collapsed same source-target import edges for 0.0.2-b.
