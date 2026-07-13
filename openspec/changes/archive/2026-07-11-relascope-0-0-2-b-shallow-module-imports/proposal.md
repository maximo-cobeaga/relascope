# Change Proposal: Relascope 0.0.2-b Shallow Module and Import Detection

## Status

Draft for approval before specification, design, tasks, and implementation.

## Problem Statement

Relascope 0.0.2-a introduced a minimal graph with `Workspace`, `Repository`, and `File` nodes connected by `CONTAINS` edges. That proves graph persistence and scan materialization, but the graph still does not express code relationships.

The next useful step is to add shallow module and import detection for Python and TypeScript/JavaScript files. This should create the first dependency-oriented graph relationships without committing to full semantic parsing, Tree-sitter, package resolution, TSConfig path aliases, or cross-repository dependency matching.

## Goals

- Create `Module` graph nodes for recognized code files.
- Detect simple Python import statements line by line.
- Detect simple TypeScript/JavaScript import/export-from statements line by line.
- Create `IMPORTS` edges between module nodes.
- Create unresolved target `Module` nodes when an import target cannot be resolved to a known file.
- Mark unresolved import target nodes/edges as unverified or equivalent in metadata/status.
- Attach evidence with repository ID, file path, line number, and excerpt.
- Keep the parser deterministic, local, and shallow.
- Extend graph summary counts to naturally include `Module` and `IMPORTS` totals.
- Add a lightweight inspection command:
  - `relascope graph imports`
- Preserve 0.0.1 inventory behavior and 0.0.2-a minimal graph behavior.

## Non-Goals

- No Tree-sitter integration yet.
- No full Python or TypeScript parser.
- No semantic symbol extraction.
- No function/class/interface nodes.
- No TSConfig path alias resolution.
- No Node package resolution.
- No Python virtualenv/package resolution.
- No cross-repository import resolution yet.
- No impact analysis.
- No AI enrichment.
- No MCP, web UI, or desktop UI.
- No stale graph pruning beyond existing behavior.

## Initial Detection Scope

### Python

Recognize simple forms such as:

```python
import os
import app.services.users
from app.models import User
from .models import User
from ..shared import helpers
```

### TypeScript / JavaScript

Recognize simple forms such as:

```ts
import { User } from "./types";
import api from "../api/client";
import "./setup";
export { foo } from "./foo";
export * from "./bar";
```

## Key Decisions

- Every recognized code file creates a `Module` node, even if it has no imports.
- Imports can create unresolved target `Module` nodes to preserve evidence.
- The initial detector is regex/line-based and intentionally shallow.
- Import resolution is best-effort only for local relative imports where simple path matching is available.
- Unresolved imports remain useful graph facts because they expose dependency intent.
- `relascope graph imports` provides a quick human inspection surface.

## User-Facing Commands

Existing commands remain:

```bash
relascope init [path]
relascope repo add <path> [--id <id>]
relascope scan
relascope status
relascope graph summary
```

New command:

```bash
relascope graph imports
```

Example output shape:

```text
Graph imports
  api:app.py -> os (unverified)
  web:src/main.ts -> ./types (unverified)
```

The exact formatting can be refined in design, but it must be deterministic and readable.

## Acceptance Summary

Relascope 0.0.2-b is acceptable when scanning the fixture creates `Module` nodes for recognized code files, creates `IMPORTS` edges for simple Python and TypeScript/JavaScript import statements, persists line-level evidence, and allows a user to inspect import relationships through `relascope graph imports`. Existing inventory, minimal graph summary, and idempotent repeated scan behavior must continue to pass.

## Review Workload Note

This change is likely larger than 0.0.2-a because it introduces detection logic, graph expansion, evidence excerpts, and a new CLI view. Keep implementation bounded to shallow line-based detection. If the scope grows, split parser/detector infrastructure from CLI inspection.
