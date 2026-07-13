# Onboarding de Relascope 0.0.2-b — Imports superficiales

Relascope 0.0.2-b agrega las primeras relaciones de dependencia de código: `Module` nodes y `IMPORTS` edges. Sigue siendo un análisis superficial, determinístico y local.

## Quick path

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
powershell -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

## Qué agrega

- `Module` nodes para archivos `.py`, `.ts`, `.tsx`, `.js` y `.jsx`.
- `IMPORTS` edges entre módulos.
- Evidencia con archivo, línea y excerpt.
- Targets no resueltos como módulos `unverified`.
- Comando:

```powershell
cargo run -p relascope-cli -- graph imports
```

## Qué no hace todavía

- No usa Tree-sitter.
- No resuelve paquetes instalados.
- No interpreta TSConfig aliases.
- No hace cross-repo resolution.
- No crea símbolos, funciones ni clases.
- No hace impacto.

## Modelo mental

Después de `scan`, el grafo tiene algo así:

```text
Workspace CONTAINS Repository
Repository CONTAINS File
File CONTAINS Module
Module IMPORTS Module
```

Ejemplo:

```text
web/src/main.ts -> web/src/message.ts [active]
api/app.py -> api:os [unverified]
```

## Estados

| Status | Significado |
|---|---|
| `active` | Relación detectada y target local resuelto |
| `unverified` | Relación detectada, pero el target no fue resuelto localmente |

Un import externo como `os` se conserva como `unverified` porque igual es evidencia útil.

## Fixture actualizado

El fixture ahora incluye imports reales:

```text
api-python/app.py
api-python/settings.py
web-typescript/src/main.ts
web-typescript/src/message.ts
```

Conteos esperados con el fixture completo:

```text
files: 12
nodes: 21
File: 12
Module: 5
Repository: 3
Workspace: 1
edges: 22
CONTAINS: 19
IMPORTS: 3
```

`graph imports` debería mostrar evidencia como:

```text
api/app.py -> api/settings.py [active]
api/app.py -> api:os [unverified]
web/src/main.ts -> web/src/message.ts [active]
```

## Archivos clave

| Archivo | Responsabilidad |
|---|---|
| `crates/core/src/imports.rs` | Detector shallow y resolver simple |
| `crates/core/src/graph.rs` | Constantes e IDs de grafo |
| `crates/storage-sqlite/src/lib.rs` | Materialización de módulos/imports y read model |
| `crates/cli/src/main.rs` | `graph imports` |
| `scripts/smoke-graph.ps1` | Smoke test automatizado |
| `docs/adr/0006-shallow-line-based-import-detection.md` | Decisión arquitectónica |

## Próximo paso natural

Después de esto, el camino lógico es mejorar resolución y analizadores:

1. Resolver imports relativos Python con más cobertura.
2. Agregar Tree-sitter para parsing robusto.
3. Crear `Module`/`Symbol` más precisos.
4. Recién después avanzar hacia impacto.
