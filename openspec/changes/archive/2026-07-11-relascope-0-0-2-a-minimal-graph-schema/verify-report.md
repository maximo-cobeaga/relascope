# Verify Report: Relascope 0.0.2-a Minimal Graph Schema

## Status

Verified by user-side PowerShell execution.

## Verification Commands

The user reported that tests completed successfully:

```powershell
cargo fmt
cargo test
```

The Pi harness shell still cannot execute Cargo directly, so command evidence is user-provided.

## Smoke Test Evidence

The user ran a manual smoke test from:

```text
C:\Users\MAXIMO\Desktop\relascope-graph-smoke
```

After a graph-materializing scan, `relascope graph summary` returned:

```text
Graph summary
  nodes: 14
  by_kind:
    File: 10
    Repository: 3
    Workspace: 1
  edges: 13
  by_relation:
    CONTAINS: 13
```

After running `relascope scan` again, `relascope graph summary` returned the same counts:

```text
Graph summary
  nodes: 14
  by_kind:
    File: 10
    Repository: 3
    Workspace: 1
  edges: 13
  by_relation:
    CONTAINS: 13
```

## Acceptance Coverage

- Minimal graph was materialized from the polyrepo fixture.
- Expected fixture node counts were verified:
  - `Workspace`: 1
  - `Repository`: 3
  - `File`: 10
  - total nodes: 14
- Expected fixture edge counts were verified:
  - `CONTAINS`: 13
  - total edges: 13
- Repeated scan did not duplicate graph nodes or edges.
- `graph summary` reads persisted graph state.
- 0.0.1 scan behavior remained functional: scan completed with `files: 10`.

## Notes

The pasted smoke output starts from an already materialized graph summary, so the explicit `no graph has been materialized yet` output was not included in this evidence. The CLI integration tests added for this change cover the no-graph-before-scan scenario.

## Conclusion

Relascope 0.0.2-a satisfies the approved minimal graph schema scope.
