# Paso a paso para testear Relascope como usuario real interno

Esta guía es para probar Relascope sobre un workspace local real después de cerrar `0.0.3-b`. La idea no es probar una demo perfecta: la idea es hacer dogfooding, detectar fricción real y guardar evidencia.

## 0. Preparación

Desde PowerShell:

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
pwsh -ExecutionPolicy Bypass -File scripts/smoke-graph.ps1
```

Si esto falla, no sigas con un proyecto real. Primero corregí el fallo.

## 1. Compilar el binario local

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo build -p relascope-cli
```

Binario esperado:

```text
C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe
```

Para simplificar comandos en esta sesión:

```powershell
$relascope = "C:\Users\MAXIMO\Desktop\relascope\target\debug\relascope.exe"
```

## 2. Crear un workspace de prueba real

Usá una carpeta separada de los repos fuente.

```powershell
$workspace = "$env:USERPROFILE\Desktop\relascope-dogfood"
Remove-Item -Recurse -Force $workspace -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $workspace

& $relascope init $workspace
cd $workspace
```

Esperado:

```text
Initialized Relascope workspace
workspace_id: ...
config: ...\relascope.yaml
database: ...\.relascope\graph.db
```

## 3. Agregar repositorios reales

Ejemplo con nombres genéricos. Reemplazá las rutas por tus repos reales.

```powershell
& $relascope repo add C:\path\to\backend --id backend
& $relascope repo add C:\path\to\dashboard --id dashboard
& $relascope repo add C:\path\to\mobile --id mobile
& $relascope repo add C:\path\to\infra --id infra
```

Para ReservApp, usá IDs humanos y cortos:

```text
backend
dashboard
mobile
web
infra
```

## 4. Verificar repos registrados

```powershell
& $relascope repo list
```

Chequeá:

- IDs visibles correctos.
- Paths correctos.
- `availability: available`.
- No agregaste por error el repo de Relascope mismo salvo que quieras testearlo.

## 5. Ejecutar doctor antes del scan

```powershell
& $relascope doctor
```

Esperado:

```text
Relascope doctor
  config: ok
  database: ok
  repositories: ... total, ... available, 0 unavailable
  last_scan: none
```

Si hay repos unavailable, corregí paths o dejalo registrado como hallazgo.

## 6. Primer scan

```powershell
& $relascope scan
```

En repos grandes, Relascope debería mostrar progreso por default:

```text
Scanning repositories...
  backend: 500 files indexed, 120 skipped, elapsed 00:00:03
```

Eso significa que el scan sigue vivo. Si necesitás una salida más limpia:

```powershell
& $relascope scan --silent
```

En el primer scan, lo normal es:

```text
added: <cantidad de archivos>
modified: 0
removed: 0
```

Anotá:

- tiempo aproximado;
- repo que más tarda según el heartbeat;
- cantidad de archivos;
- si el conteo parece razonable;
- si escaneó cosas que deberían estar excluidas.

Nota: `skipped` es una señal de progreso; puede contar entradas excluidas podadas y no necesariamente todos los archivos dentro de una carpeta excluida.

## 7. Inspeccionar estado general

```powershell
& $relascope status
& $relascope doctor
```

Chequeá:

- último scan `completed`;
- repos siguen disponibles;
- `unverified_imports` es razonable;
- `stale_nodes` / `stale_edges` deberían ser bajos o cero en primer scan.

## 8. Inspeccionar grafo

```powershell
& $relascope graph summary
& $relascope graph imports
```

Buscá:

- `File` count razonable;
- `Module` count razonable;
- `IMPORTS` mayor a cero si hay Python/TS/JS;
- imports locales resueltos como `[active]`;
- imports externos como `[unverified]`;
- evidencia con línea correcta.

## 9. Guardar evidencia JSON

```powershell
& $relascope scan --format json > scan.json
& $relascope graph summary --format json > graph-summary.json
& $relascope graph imports --format json > graph-imports.json
```

`scan --format json` mantiene stdout como JSON limpio, sin líneas de progreso mezcladas.

Abrí los archivos y verificá que son JSON válido.

```powershell
Get-Content .\graph-summary.json
Get-Content .\graph-imports.json
```

Nota: JSON es experimental en `0.0.3-b`; sirve para dogfooding, no como API pública estable.

## 10. Probar incrementalidad sin romper nada importante

Hacé esto en una rama descartable o en un repo/copias donde no haya riesgo.

### 10.1 Modificar un import

Editá un archivo `.py`, `.ts` o `.js` y remové/agregá un import simple.

Después:

```powershell
& $relascope scan
& $relascope graph imports
```

Esperado:

```text
modified: 1
```

Y si removiste un import, el import viejo debería aparecer como:

```text
[stale]
```

### 10.2 Borrar un archivo no crítico

Solo si estás en una copia o rama segura.

```powershell
& $relascope scan
& $relascope graph summary
& $relascope graph imports
```

Esperado:

```text
removed: 1
```

Los facts asociados deberían quedar `stale`, no borrarse físicamente.

## 11. Probar repo remove sin borrar archivos

Elegí un repo que puedas volver a agregar.

```powershell
& $relascope repo remove infra
& $relascope repo list
```

Chequeá que:

- ya no aparece en `repo list`;
- la carpeta real sigue existiendo en disco;
- un scan posterior no lo escanea.

Para volver a agregarlo:

```powershell
& $relascope repo add C:\path\to\infra --id infra
```

## 12. Plantilla de hallazgos

Copiá esto en un archivo tipo `dogfood-notes.md` dentro del workspace:

```markdown
# Relascope dogfood notes

Date:
Workspace path:
Repos:
Relascope version/milestone: 0.0.3-b

## Commands run

- [ ] repo list
- [ ] doctor before scan
- [ ] scan
- [ ] status
- [ ] doctor after scan
- [ ] graph summary
- [ ] graph imports
- [ ] JSON exports
- [ ] incremental modification test
- [ ] removed file test
- [ ] repo remove test

## Useful findings

- 

## Confusing output

- 

## False positives

- 

## False negatives

- 

## Unresolved imports that should resolve

- 

## Stale behavior notes

- 

## Performance notes

- 

## Bugs

- 

## Next product improvements

- 
```

## 13. Criterio de éxito de esta prueba

La prueba es exitosa si podés responder:

- ¿Pude registrar repos reales sin fricción grave?
- ¿El scan terminó?
- ¿Los conteos son razonables?
- ¿`doctor` me dijo algo útil?
- ¿`graph imports` muestra relaciones verificables?
- ¿El JSON sirve para guardar evidencia?
- ¿La incrementalidad marca cambios/removidos sin mentir?

Si la respuesta es sí, Relascope está listo para seguir hacia el próximo hito técnico.
