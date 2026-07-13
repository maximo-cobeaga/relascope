# Onboarding de Relascope 0.0.2-a — Grafo mínimo

Este tutorial complementa el onboarding de `0.0.1`. Explica la primera capa de grafo de Relascope: qué se guarda, cómo se materializa y cómo inspeccionarlo.

## Quick path

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Si querés inspeccionar la base generada, conservá el workspace temporal:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1 -KeepWorkspace
```

Resultado esperado en el estado actual del fixture, después de 0.0.2-b:

```text
Graph summary
  nodes: 21
  by_kind:
    File: 12
    Module: 5
    Repository: 3
    Workspace: 1
  edges: 22
  by_relation:
    CONTAINS: 19
    IMPORTS: 3
```

Nota: en el cierre histórico de 0.0.2-a el fixture tenía menos archivos y todavía no había `Module`/`IMPORTS`. Los conteos actuales reflejan el fixture evolucionado.

---

## Qué agrega 0.0.2-a

`0.0.2-a` agrega una capa mínima de grafo persistido.

Antes teníamos inventario:

```text
workspaces
repositories
scans
files
```

Ahora además tenemos:

```text
nodes
edges
evidence
```

## Qué nodos existen

| Node kind | Qué representa |
|---|---|
| `Workspace` | El workspace Relascope actual |
| `Repository` | Cada repo registrado |
| `File` | Cada archivo inventariado por scan |

## Qué relación existe

| Edge | Significado |
|---|---|
| `CONTAINS` | Un workspace contiene repos; un repo contiene archivos |

No hay todavía `IMPORTS`, `CALLS`, módulos ni símbolos.

---

## Qué hace `scan` ahora

`relascope scan` sigue inventariando archivos como en `0.0.1`, pero además materializa el grafo mínimo.

Flujo conceptual:

```text
scan
  ├─ registra scan
  ├─ inventaría archivos
  ├─ guarda rows en files
  ├─ crea/actualiza nodes
  ├─ crea/actualiza edges CONTAINS
  ├─ guarda evidence básica
  └─ marca scan como completed
```

## Qué hace `graph summary`

`relascope graph summary` lee la base SQLite y muestra conteos persistidos.

No escanea.
No toca repositorios.
No actualiza disponibilidad.
No usa IA.
No necesita red.

Antes del primer scan:

```text
Graph summary
  no graph has been materialized yet
```

Después del scan:

```text
Graph summary
  nodes: ...
  by_kind:
    ...
  edges: ...
  by_relation:
    ...
```

---

## Identidad del grafo

Los IDs del grafo son determinísticos.

Eso significa: si corrés `scan` dos veces sobre el mismo workspace, no debería duplicar nodos ni edges.

Ejemplos conceptuales:

```text
node:workspace:<workspace_id>
node:repository:<workspace_id>:<repository_id>
node:file:<workspace_id>:<repository_id>:<hash(relative_path)>
```

Para archivos, la identidad inicial es:

```text
workspace_id + repository_id + relative_path
```

Esto todavía no resuelve renames. Es una limitación aceptada para esta etapa.

---

## Limitación importante: no hay pruning stale

Si un archivo existía en un scan anterior y después desaparece, el nodo puede quedar en el grafo.

Por ahora el grafo representa hechos conocidos/materializados, no una vista perfectamente podada del estado actual.

La invalidación incremental queda para un milestone posterior.

---

## Dónde está el código

| Área | Archivo |
|---|---|
| Tipos e IDs de grafo | `crates/core/src/graph.rs` |
| Migraciones/tablas graph | `crates/storage-sqlite/src/lib.rs` |
| Materialización graph | `crates/storage-sqlite/src/lib.rs` |
| CLI `graph summary` | `crates/cli/src/main.rs` |
| Tests CLI graph | `crates/cli/tests/cli.rs` |
| ADR | `docs/adr/0005-minimal-graph-persistence.md` |

---

## Qué sigue después

El próximo paso natural es `0.0.2-b`:

- detectar módulos de forma superficial;
- detectar imports Python y TypeScript;
- crear `Module` nodes;
- crear `IMPORTS` edges;
- adjuntar evidencia por archivo/línea.

No conviene hacer impacto hasta que el grafo tenga relaciones reales entre componentes.
