# ADR 0006: Shallow Line-Based Import Detection

## Status

Accepted for Relascope 0.0.2-b.

## Context

Relascope has a minimal graph foundation with workspace, repository, file, module-ready storage, edges, and evidence. The next useful graph relationship is imports between modules. A full parser stack such as Tree-sitter is valuable later, but introducing it before proving the graph relationship model would increase scope and dependency complexity.

## Decision

Implement shallow line-based import detection for Python and TypeScript/JavaScript first.

The detector recognizes simple forms:

- Python `import x`
- Python `import x, y`
- Python `from x import y`
- TypeScript/JavaScript `import ... from "x"`
- TypeScript/JavaScript side-effect imports
- TypeScript/JavaScript `export ... from "x"`

Relascope creates `Module` nodes and `IMPORTS` edges with line-level evidence. Unresolved targets are preserved as unverified module nodes rather than being dropped.

## Consequences

This provides the first dependency-oriented graph facts while remaining deterministic, local, and testable. It will miss multiline, dynamic, alias-heavy, and package-resolution cases. Those limitations are acceptable for 0.0.2-b and should be addressed by later analyzer milestones.
