# Tasks: Relascope 0.0.2-b Shallow Module and Import Detection

## Implementation Plan

Keep this slice limited to shallow line-based import detection and graph inspection. Do not implement Tree-sitter, full parser semantics, symbol nodes, package resolution, cross-repo resolution, impact analysis, AI, MCP, web UI, or desktop UI.

## 1. Core graph constants and IDs

- [x] Add `Module` node kind constant.
- [x] Add `IMPORTS` relation constant.
- [x] Add `unverified` status constant.
- [x] Add deterministic module node ID helper for source modules.
- [x] Add deterministic unresolved module node ID helper for unresolved target modules.
- [x] Add deterministic import edge/evidence ID helpers or reuse existing stable evidence helpers safely.
- [x] Add unit tests for stable module/import IDs.

## 2. Import detector module

- [x] Create `crates/core/src/imports.rs`.
- [x] Define `ImportFact` read model.
- [x] Implement Python line-based detection for `import x`.
- [x] Implement Python line-based detection for `import x, y`.
- [x] Implement Python line-based detection for `import x as alias`.
- [x] Implement Python line-based detection for `from x import y`.
- [x] Preserve relative Python specifiers such as `.models` and `..shared`.
- [x] Ignore Python comment lines.
- [x] Implement TS/JS detection for `import ... from "specifier"`.
- [x] Implement TS/JS side-effect import detection.
- [x] Implement TS/JS `export ... from "specifier"` detection.
- [x] Ignore TS/JS comment lines.
- [x] Do not parse multiline imports in this slice.
- [x] Add unit tests for Python and TS/JS detectors.

## 3. Shallow local resolver

- [x] Build a per-repository current-scan file index by relative path.
- [x] Resolve TypeScript/JavaScript relative imports using simple candidates:
  - [x] `<specifier>`
  - [x] `<specifier>.ts`
  - [x] `<specifier>.tsx`
  - [x] `<specifier>.js`
  - [x] `<specifier>.jsx`
  - [x] `<specifier>/index.ts`
  - [x] `<specifier>/index.tsx`
  - [x] `<specifier>/index.js`
  - [x] `<specifier>/index.jsx`
- [x] Add conservative Python local resolution for simple dotted/relative imports if safe.
- [x] Mark unresolved/external targets as `unverified` in node or edge metadata/status.
- [x] Add resolver unit tests for simple TS relative imports.

## 4. Fixture updates

- [x] Update `fixtures/polyrepo-basic/api-python/app.py` with simple Python imports.
- [x] Add `fixtures/polyrepo-basic/api-python/settings.py`.
- [x] Update `fixtures/polyrepo-basic/web-typescript/src/main.ts` with simple import/export-from lines.
- [x] Add `fixtures/polyrepo-basic/web-typescript/src/message.ts`.
- [x] Update expected fixture counts in tests and smoke script.

## 5. Storage materialization

- [x] Materialize source `Module` nodes for all recognized code files in current scan.
- [x] Materialize `File CONTAINS Module` edges.
- [x] Materialize unresolved target `Module` nodes for unresolved imports.
- [x] Materialize `IMPORTS` edges with `active` status for resolved local imports and `unverified` status for unresolved/external imports.
- [x] Store line-level evidence for each detected import with file path, line number, and excerpt.
- [x] Ensure repeated scans do not duplicate module nodes or import edges.
- [x] Preserve existing `CONTAINS` graph behavior.

## 6. Graph imports read model

- [x] Add storage read model for import edges.
- [x] Join source module, target module, edge status, evidence file path, evidence line, and excerpt.
- [x] Sort output deterministically by source, target, and evidence location.
- [x] Add storage tests for listing import edges.

## 7. CLI command

- [x] Add `relascope graph imports` command parsing.
- [x] Print a clear no-imports-yet message when no imports exist.
- [x] Print source, target, status, and evidence for each import.
- [x] Ensure `graph imports` does not scan or update availability.
- [x] Add CLI integration tests for before/after scan behavior.

## 8. Documentation and ADRs

- [x] Update `README.md` with `relascope graph imports`.
- [x] Update or add tutorial for 0.0.2-b shallow imports.
- [x] Add ADR for shallow line-based import detection before Tree-sitter.
- [x] Update smoke script expectations and docs for changed fixture counts.

## 9. Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run automated graph smoke script.
- [x] Run or create automated imports smoke path.
- [x] Verify graph summary includes `Module` and `IMPORTS` when fixture imports exist.
- [x] Verify `graph imports` shows evidence lines.
- [x] Verify repeated scan does not duplicate `IMPORTS` edges.
- [x] Verify existing inventory/status behavior still works.

## Review Workload Forecast

This is likely larger than 0.0.2-a. Keep it bounded by avoiding parser scope creep. If it grows too much, split into:

1. Core detector + fixture/tests.
2. Storage materialization + summary/import read models.
3. CLI/docs/smoke script updates.

The user has previously accepted larger local change sets, but this should still remain reviewable by behavior unit.
