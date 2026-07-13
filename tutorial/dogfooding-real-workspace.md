# Dogfooding Relascope on a real workspace

This guide explains how to run Relascope against a real local multi-repository project, collect evidence, and record product findings. The recommended first real target is ReservApp or another local workspace with backend/frontend/mobile/infra repositories.

## Goal

Use Relascope as an internal early user, not as a polished external product yet.

You are trying to answer:

- Did workspace setup feel clear?
- Did repository registration work?
- Did scan complete without touching repository code?
- Did graph summary look plausible?
- Did shallow imports expose useful relationships?
- Did `doctor` surface useful health signals?
- Were unresolved imports understandable?
- Did stale marking behave predictably after changes?

## 1. Create a dogfood workspace

Create the Relascope workspace outside your source repositories.

```powershell
mkdir C:\Users\MAXIMO\Desktop\relascope-dogfood
cd C:\Users\MAXIMO\Desktop\relascope
cargo run -p relascope-cli -- init C:\Users\MAXIMO\Desktop\relascope-dogfood
cd C:\Users\MAXIMO\Desktop\relascope-dogfood
```

## 2. Add local repositories

Use local paths. Example shape:

```powershell
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe repo add C:\path\to\backend --id backend
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe repo add C:\path\to\dashboard --id dashboard
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe repo add C:\path\to\mobile --id mobile
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe repo add C:\path\to\infra --id infra
```

Or keep using `cargo run --manifest-path` while developing:

```powershell
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- repo add C:\path\to\backend --id backend
```

## 3. Inspect repository registration

```powershell
relascope repo list
```

Expected:

```text
Repositories
  - backend
      internal_id: ...
      path: ...
      availability: available
```

## 4. Run doctor before scan

```powershell
relascope doctor
```

Doctor is read-only. It does not scan file trees and does not update repository availability.

Record:

- unavailable repositories;
- config/database issues;
- no-scan-yet state;
- stale/unverified graph counts if present.

## 5. Run scan

```powershell
relascope scan
```

For large repositories, scan prints terminal-friendly progress by default so you can tell it is alive:

```text
Scanning repositories...
  backend: 500 files indexed, 120 skipped, elapsed 00:00:03
  backend: 1000 files indexed, 340 skipped, elapsed 00:00:06
```

Expected final output includes:

```text
Scan completed
  files: ...
  added: ...
  modified: ...
  removed: ...
```

First scan should report all files as `added`. Use `relascope scan --silent` if you want to suppress progress and keep only the final summary. The skipped count is a progress signal; it may count pruned excluded entries rather than every file under an excluded directory.

## 6. Inspect graph

```powershell
relascope graph summary
relascope graph imports
```

Look for:

- plausible file count;
- `Module` count for recognized code files;
- `IMPORTS` count;
- imports marked `active`, `unverified`, or `stale`;
- evidence lines that point to real source lines.

## 7. Save JSON evidence

JSON output is experimental in `0.0.3-b`. Use it for local evidence capture, not as a stable API contract.

```powershell
relascope scan --format json > scan.json
relascope graph summary --format json > graph-summary.json
relascope graph imports --format json > graph-imports.json
```

`scan --format json` keeps stdout as JSON only, without progress text mixed into the file.

## 8. Try an incremental change

In a non-critical branch or disposable test copy, change one import line in a real repo, then run:

```powershell
relascope scan
relascope graph imports
```

Check whether old imports become `stale` and current imports remain `active` or `unverified`.

## 9. Findings template

Copy this into a note after each dogfood run.

```markdown
# Relascope dogfood run

Date:
Workspace:
Repositories:

## Commands run

- [ ] repo list
- [ ] doctor
- [ ] scan
- [ ] graph summary
- [ ] graph imports
- [ ] JSON exports

## Useful findings

- 

## False positives

- 

## False negatives

- 

## Unresolved imports that should resolve

- 

## Stale behavior notes

- 

## Slow or confusing output

- 

## Bugs / crashes

- 

## Product improvements suggested

- 
```

## Safety reminder

Relascope scan is designed to be read-only against registered repositories. It should not execute repository code, install dependencies, call network, or use AI.
