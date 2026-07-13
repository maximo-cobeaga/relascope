# Relascope

Relascope is a local-first cross-repository architecture inventory and impact-analysis tool.

This repository currently implements the first local-first engine milestones through **Relascope 0.0.3-b — Real Workspace Dogfooding**.

## Current Scope

Included:

- `relascope init [path]`
- `relascope repo add <path> [--id <id>]`
- `relascope repo list`
- `relascope repo remove <id>`
- `relascope scan [--format human|json] [--silent]`
- `relascope status`
- `relascope doctor`
- `relascope graph summary [--format human|json]`
- `relascope graph imports [--format human|json]`
- local SQLite persistence at `.relascope/graph.db`
- workspace configuration in `relascope.yaml`
- stable workspace and repository UUIDs
- visible repository IDs generated from folder slugs or provided with `--id`
- shallow inventory of all non-excluded files
- default and configurable scan exclusions
- missing repository paths reported as unavailable without silent deletion

Not included yet:

- Git URL registration or cloning
- semantic code analysis
- semantic graph edges beyond shallow `CONTAINS` and `IMPORTS`
- cross-repository impact analysis
- AI/model integration
- MCP
- web UI
- desktop app

## Onboarding

Start here if you are new to the project or new to Rust:

- [Relascope 0.0.1 onboarding tutorial](tutorial/onboarding-relascope-0-0-1.md)
- [Relascope 0.0.2-a minimal graph tutorial](tutorial/onboarding-relascope-0-0-2-a.md)
- [Relascope 0.0.2-b shallow imports tutorial](tutorial/onboarding-relascope-0-0-2-b.md)
- [Relascope 0.0.3-a incremental stale graph tutorial](tutorial/onboarding-relascope-0-0-3-a.md)
- [Current state and next steps](tutorial/current-state-and-next-steps.md)
- [Dogfooding Relascope on a real workspace](tutorial/dogfooding-real-workspace.md)
- [Step-by-step real workspace test guide](tutorial/test-real-workspace-step-by-step.md)

## Build and Test

```bash
cargo fmt
cargo test
```

## Smoke Test

On Windows PowerShell, run the automated graph smoke test:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Keep the generated workspace for inspection:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1 -KeepWorkspace
```

## Example

```bash
cargo run -p relascope-cli -- init /tmp/relascope-workspace
cd /tmp/relascope-workspace
cargo run -p relascope-cli -- repo add ../some-local-repo --id api
cargo run -p relascope-cli -- repo list
cargo run -p relascope-cli -- scan
# Use --silent when you want only the final summary.
# cargo run -p relascope-cli -- scan --silent
cargo run -p relascope-cli -- status
cargo run -p relascope-cli -- doctor
cargo run -p relascope-cli -- graph summary
cargo run -p relascope-cli -- graph imports
cargo run -p relascope-cli -- graph imports --format json
```

## Path Portability Limitation

Relascope 0.0.1 stores canonical local repository paths for operation. Repository identity is not the path: each repository has a stable internal UUID and a visible ID. However, relocating a workspace or its repositories may require reconfiguration until future versions add portable path anchors or source descriptors.
