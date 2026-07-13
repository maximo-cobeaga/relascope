# Onboarding de Relascope 0.0.3-a — Incrementalidad y stale graph

Relascope 0.0.3-a agrega comparación entre scans y marcado de hechos obsoletos como `stale`. El objetivo es proteger la confianza del grafo antes de sumar analizadores más profundos.

## Quick path

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

## Qué agrega

- Comparación contra el scan completado anterior.
- Conteos incrementales en `relascope scan`:
  - `added`
  - `modified`
  - `removed`
- Estado `stale` para nodos/aristas obsoletos.
- Columna `nodes.status`.
- Smoke test que copia fixtures a una carpeta temporal antes de mutarlos.

## Qué no agrega todavía

- No watcher.
- No detección inteligente de renames.
- No Tree-sitter.
- No package resolution.
- No cross-repo resolution.
- No impact analysis.

## Modelo mental

Antes, Relascope materializaba hechos actuales, pero podía dejar relaciones viejas activas.

Ahora el flujo conceptual es:

```text
scan actual
  ├─ carga scan anterior completed
  ├─ inventaría archivos actuales
  ├─ calcula added/modified/removed/unchanged
  ├─ marca hechos viejos como stale
  ├─ materializa hechos actuales
  └─ guarda resumen incremental
```

## Output esperado de scan

Primer scan:

```text
Scan completed
  files: 12
  added: 12
  modified: 0
  removed: 0
```

Segundo scan sin cambios:

```text
Scan completed
  files: 12
  added: 0
  modified: 0
  removed: 0
```

Después de modificar un archivo:

```text
Scan completed
  files: 12
  added: 0
  modified: 1
  removed: 0
```

Después de remover un archivo:

```text
Scan completed
  files: 11
  added: 0
  modified: 0
  removed: 1
```

## Cómo se ve stale en imports

Si un import desaparece de un archivo modificado, deja de quedar activo:

```text
api/app.py -> api:os [stale]
```

Si un archivo target desaparece, el import resuelto viejo queda stale y puede aparecer un nuevo import no resuelto:

```text
api/app.py -> api/settings.py [stale]
api/app.py -> api:.settings [unverified]
```

## Por qué no borramos

No borramos facts stale porque todavía son útiles como historia/evidencia. Borrar temprano hace más difícil auditar qué cambió y por qué.

## Renames

En esta versión, un rename se modela como:

```text
removed old/path
added new/path
```

No se afirma que sea el mismo archivo.

## Archivo clave

| Archivo | Responsabilidad |
|---|---|
| `crates/core/src/inventory.rs` | diff entre scans |
| `crates/core/src/graph.rs` | `STATUS_STALE` y `GraphNode.status` |
| `crates/storage-sqlite/src/lib.rs` | migración `nodes.status` y stale marking |
| `crates/cli/src/main.rs` | output incremental de `scan` |
| `scripts/smoke-graph.ps1` | smoke incremental con fixture copiado |
| `docs/adr/0007-stale-graph-marking.md` | decisión de marcar stale en vez de borrar |

## Próximo paso natural

Después de `0.0.3-a`, hay dos caminos razonables:

1. `0.0.3-b` — watcher manual/observación de cambios.
2. `0.0.4-a` — Tree-sitter parser foundation.

Mi recomendación: si el objetivo es una CLI útil rápido, hacer `0.0.3-b` watcher. Si el objetivo es precisión del grafo, empezar Tree-sitter.
