# Onboarding de Relascope 0.0.1 — Inventario

Este tutorial explica qué es Relascope hoy, cómo está armado el proyecto, qué hace cada comando y cómo trabajar con el código aunque todavía no sepas Rust. Está escrito para volver a entrar al proyecto rápido después de unos días, o para que una persona nueva entienda el primer milestone sin leer toda la arquitectura completa.

## Quick path

Si solo querés comprobar que el proyecto funciona:

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
cargo fmt
cargo test
```

Resultado esperado:

```text
9 tests passed
```

Para probar la CLI manualmente:

```powershell
$smoke = "$env:USERPROFILE\Desktop\relascope-smoke"
Remove-Item -Recurse -Force $smoke -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $smoke

cargo run -p relascope-cli -- init $smoke
cd $smoke

cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- repo add C:\Users\MAXIMO\Desktop\relascope\fixtures\polyrepo-basic\api-python --id api
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- repo add C:\Users\MAXIMO\Desktop\relascope\fixtures\polyrepo-basic\web-typescript --id web
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- repo add C:\Users\MAXIMO\Desktop\relascope\fixtures\polyrepo-basic\infra-config --id infra

cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- status
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- scan
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- status
```

---

## 1. Qué es Relascope

Relascope es una herramienta **local-first** para entender sistemas compuestos por varios repositorios.

La visión completa es construir un grafo arquitectónico verificable: repositorios, archivos, módulos, contratos, infraestructura, dependencias, evidencia e impacto de cambios.

Pero el estado actual es más chico.

## 2. Qué es Relascope 0.0.1

Relascope `0.0.1` es el primer milestone: **Inventario**.

Su objetivo no es analizar código semánticamente. Su objetivo es demostrar que Relascope puede:

1. Crear un workspace local.
2. Registrar varios repositorios locales.
3. Escanear archivos no excluidos.
4. Guardar el inventario en SQLite.
5. Reabrir el workspace sin perder identidad ni configuración.
6. Mostrar estado persistido sin volver a escanear.

### Incluido en 0.0.1

| Capacidad | Estado |
|---|---|
| CLI básica | Implementada |
| `init` | Implementado |
| `repo add` local | Implementado |
| `scan` shallow | Implementado |
| `status` persistido | Implementado |
| SQLite local | Implementado |
| Fixture polyrepo | Implementado |
| Tests con `cargo test` | Implementados |
| ADRs iniciales | Implementados |

### Fuera de 0.0.1

| Capacidad | Motivo |
|---|---|
| Grafo real de nodos/aristas | Empieza en 0.0.2 |
| Imports TypeScript/Python | Empieza después del inventario base |
| IA | No es necesaria para inventario |
| MCP | Futuro adaptador, no core inicial |
| Desktop/Tauri | Todavía no hay motor suficiente |
| Web UI | Todavía no hay grafo para explorar |
| Clonado Git por URL | Evita red/autenticación en la primera versión |
| Análisis semántico | 0.0.1 solo inventaría archivos |

---

## 3. Modelo mental del producto

Pensalo así:

```text
Workspace Relascope
  ├─ relascope.yaml              configuración visible
  └─ .relascope/graph.db         base local SQLite
       ├─ workspaces             identidad del workspace
       ├─ repositories           repos registrados
       ├─ scans                  corridas de scan
       └─ files                  archivos inventariados
```

Relascope no modifica los repositorios registrados. Solo los lee.

### Regla importante

`scan` lee archivos. `status` lee la base.

Eso significa:

- `scan` actualiza disponibilidad e inventario.
- `status` no reescanea el filesystem.
- Si un repo desaparece, `status` no lo detecta hasta que corras `scan`.

Esto es intencional: permite validar persistencia real.

---

## 4. Estructura del repositorio

```text
relascope/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── core/
│   ├── storage-sqlite/
│   └── cli/
├── fixtures/
│   └── polyrepo-basic/
├── docs/
│   └── adr/
├── openspec/
├── tutorial/
└── README.md
```

### Qué hay en cada carpeta

| Ruta | Para qué sirve |
|---|---|
| `Cargo.toml` | Define el workspace Rust y sus crates |
| `rust-toolchain.toml` | Indica usar Rust stable |
| `crates/core` | Lógica de dominio: config, clasificación, scan, workspace |
| `crates/storage-sqlite` | Persistencia SQLite |
| `crates/cli` | Binario `relascope` y comandos de usuario |
| `fixtures/polyrepo-basic` | Repos de prueba: Python, TypeScript e infraestructura |
| `docs/adr` | Decisiones arquitectónicas |
| `openspec` | Especificaciones y cambios SDD/OpenSpec |
| `tutorial` | Guías de onboarding y aprendizaje |

---

## 5. Rust mínimo para entender este proyecto

No necesitás dominar Rust para moverte. Primero entendé estas piezas.

### `crate`

Un `crate` es un paquete Rust.

Relascope tiene tres crates:

```text
relascope-core
relascope-storage-sqlite
relascope-cli
```

### `workspace`

Un workspace agrupa varios crates. El `Cargo.toml` raíz dice:

```toml
[workspace]
members = [
    "crates/core",
    "crates/storage-sqlite",
    "crates/cli",
]
```

Eso permite correr comandos sobre todo el proyecto:

```powershell
cargo test
```

### `Result<T>`

Rust no usa excepciones como JavaScript/Python. Muchas funciones devuelven:

```rust
Result<T, Error>
```

Significa:

- `Ok(valor)` si salió bien.
- `Err(error)` si falló.

En el código vas a ver `?`, por ejemplo:

```rust
let config = load_workspace_config(&root)?;
```

Eso significa: si hay error, salí de la función devolviendo ese error.

### `struct`

Un `struct` es una estructura de datos.

Ejemplo conceptual:

```rust
pub struct RepositoryRecord {
    pub id: String,
    pub visible_id: String,
    pub path: PathBuf,
}
```

Es parecido a un objeto plano o interface con datos.

### `async`

La persistencia SQLite usa funciones async porque SQLx trabaja con operaciones asíncronas.

Por eso el `main` de la CLI tiene:

```rust
#[tokio::main]
async fn main() { ... }
```

No necesitás tocar esto al principio. Solo recordá que llamadas a SQLite suelen llevar `.await`.

---

## 6. Qué hace cada crate

## 6.1 `relascope-core`

Ruta:

```text
crates/core
```

Contiene lógica pura o casi pura. No debería depender de la CLI.

Archivos importantes:

| Archivo | Responsabilidad |
|---|---|
| `config.rs` | Modelo de `relascope.yaml` y exclusiones default |
| `workspace.rs` | Buscar workspace, leer/escribir config, paths internos |
| `repository.rs` | Slugs, validación de visible IDs, modelo de repo |
| `classify.rs` | Clasificación shallow de archivos |
| `scan.rs` | Recorrido de repositorios e inventario de archivos |
| `inventory.rs` | Modelos de archivos inventariados y resumen de scan |
| `error.rs` | Errores del core |
| `lib.rs` | Expone módulos públicos |

### Responsabilidades del core

- No parsear argumentos CLI.
- No imprimir mensajes al usuario.
- No decidir formato de salida terminal.
- Sí saber cómo se clasifica un archivo.
- Sí saber cómo se escanea una lista de repositorios.

---

## 6.2 `relascope-storage-sqlite`

Ruta:

```text
crates/storage-sqlite
```

Contiene la capa SQLite.

Responsabilidades:

- Abrir `.relascope/graph.db`.
- Crear tablas si faltan.
- Guardar workspace.
- Guardar repositorios.
- Guardar scans.
- Guardar archivos inventariados.
- Leer estado para `status`.

Tablas actuales:

| Tabla | Contenido |
|---|---|
| `workspaces` | ID, nombre y root del workspace |
| `repositories` | Repos registrados |
| `scans` | Corridas de scan |
| `files` | Inventario de archivos por scan |

### Nota sobre el nombre `graph.db`

Aunque todavía no hay grafo, usamos:

```text
.relascope/graph.db
```

porque ese será el almacén local del conocimiento del producto.

---

## 6.3 `relascope-cli`

Ruta:

```text
crates/cli
```

Contiene el binario:

```text
relascope
```

Responsabilidades:

- Parsear comandos con `clap`.
- Mostrar mensajes al usuario.
- Conectar core + storage.
- Convertir errores internos en errores de terminal.

Archivo principal:

```text
crates/cli/src/main.rs
```

Tests de integración:

```text
crates/cli/tests/cli.rs
```

---

## 7. Comandos del proyecto para desarrollo

Estos comandos se ejecutan desde la raíz del repo:

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
```

### Formatear código

```powershell
cargo fmt
```

Qué hace:

- Reordena visualmente el código Rust.
- No cambia la lógica.
- Es como Prettier, pero para Rust.

### Correr tests

```powershell
cargo test
```

Qué hace:

- Compila el proyecto.
- Corre tests unitarios.
- Corre tests de integración.
- Corre doc-tests si existen.

### Compilar sin ejecutar

```powershell
cargo build
```

Qué hace:

- Genera binarios en `target/`.

### Ejecutar la CLI desde código fuente

```powershell
cargo run -p relascope-cli -- <comando>
```

Ejemplo:

```powershell
cargo run -p relascope-cli -- init C:\tmp\demo-relascope
```

Cuando estás fuera del repo y querés ejecutar usando el `Cargo.toml` del proyecto:

```powershell
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- status
```

---

## 8. Comandos de Relascope 0.0.1

## 8.1 `relascope init [path]`

Crea un workspace Relascope.

### Uso

```powershell
cargo run -p relascope-cli -- init
```

O:

```powershell
cargo run -p relascope-cli -- init C:\Users\MAXIMO\Desktop\relascope-smoke
```

### Qué crea

```text
relascope.yaml
.relascope/graph.db
```

### Qué guarda

- UUID interno del workspace.
- Nombre del workspace.
- Exclusiones default.
- Registro inicial del workspace en SQLite.

### Ejemplo de salida

```text
Initialized Relascope workspace
  root: C:\...
  workspace_id: ...
  config: ...\relascope.yaml
  database: ...\.relascope\graph.db
```

### Reglas

- Si no pasás path, usa el directorio actual.
- Si pasás path, inicializa ahí.
- Si ya existe `relascope.yaml`, no pisa el workspace.

---

## 8.2 `relascope repo add <path> [--id <id>]`

Registra un repositorio local existente.

### Uso

```powershell
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- repo add C:\ruta\a\repo --id api
```

### Qué guarda

- UUID interno estable del repositorio.
- ID visible (`api`, `web`, `infra`, etc.).
- Nombre visible.
- Path canonicalizado.
- Disponibilidad inicial `available`.

### Identidad interna vs ID visible

Relascope separa dos cosas:

| Concepto | Ejemplo | Para qué sirve |
|---|---|---|
| UUID interno | `dc803128-...` | Identidad estable del sistema |
| ID visible | `api` | Nombre cómodo para humanos |

El path **no** es la identidad del repo.

### Reglas

- Solo acepta carpetas locales existentes.
- No acepta URLs Git en 0.0.1.
- No permite IDs visibles duplicados dentro del workspace.
- `--id` es opcional.
- Si no pasás `--id`, genera un slug desde el nombre de la carpeta.

---

## 8.3 `relascope scan`

Escanea repositorios registrados y guarda inventario.

### Uso

```powershell
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- scan
```

### Qué hace

- Lee repositorios registrados.
- Recorre archivos no excluidos.
- Clasifica archivos de forma superficial.
- Guarda un registro por archivo.
- Guarda un resumen del scan.
- Marca repos faltantes como `unavailable`.

### Qué no hace

- No ejecuta código del repo.
- No instala dependencias.
- No llama a internet.
- No usa IA.
- No analiza semánticamente imports, funciones ni clases.

### Exclusiones default

```text
.git
node_modules
.venv
dist
build
target
.relascope
```

### Clasificaciones actuales

| Kind | Ejemplo |
|---|---|
| `code` | `.py`, `.ts`, `.js`, `.rs` |
| `configuration` | `.json`, `.yaml`, `.toml`, `docker-compose.yml` |
| `documentation` | `.md`, `.mdx` |
| `binary` | `.png`, `.zip`, `.exe`, etc. |
| `generated` | paths con `generated` o `.generated` |
| `unknown` | extensiones no reconocidas |

---

## 8.4 `relascope status`

Muestra el estado persistido del workspace.

### Uso

```powershell
cargo run --manifest-path C:\Users\MAXIMO\Desktop\relascope\Cargo.toml -p relascope-cli -- status
```

### Qué muestra

- Nombre del workspace.
- UUID del workspace.
- Root del workspace.
- Repos registrados.
- IDs internos.
- Paths.
- Disponibilidad.
- Último scan.
- Conteos por kind y lenguaje.

### Regla clave

`status` no escanea.

Si un repo fue borrado o renombrado, `status` seguirá mostrando el estado persistido hasta que corras:

```powershell
relascope scan
```

---

## 9. Fixture polyrepo

Ruta:

```text
fixtures/polyrepo-basic
```

Contiene tres repos chicos:

```text
api-python/
web-typescript/
infra-config/
```

### Para qué sirve

Validar que Relascope soporta:

- varios repositorios;
- Python;
- TypeScript;
- configuración;
- documentación;
- archivos desconocidos;
- exclusiones;
- persistencia.

### Conteo esperado actual

Después de agregar los tres repos y correr `scan`, el fixture actual inventaría:

```text
files: 12
```

Nota: el smoke histórico de 0.0.1 dio `files: 10`; el fixture creció en 0.0.2-b para incluir ejemplos de imports.

Por kind:

```text
code: 4
configuration: 4
documentation: 2
unknown: 2
```

Por lenguaje:

```text
json: 1
markdown: 2
python: 2
typescript: 2
yaml: 1
```

---

## 10. OpenSpec y estado del milestone

Relascope usa OpenSpec para no implementar desde chat suelto.

El cambio `0.0.1` fue cerrado y archivado en:

```text
openspec/changes/archive/2026-07-11-relascope-0-0-1-inventory
```

La spec activa quedó en:

```text
openspec/specs/workspace-inventory/spec.md
```

Reportes relevantes:

```text
verify-report.md
smoke-test-report.md
sync-report.md
```

---

## 11. ADRs iniciales

Los ADRs documentan decisiones importantes.

Ruta:

```text
docs/adr
```

| ADR | Decisión |
|---|---|
| `0001-rust-cli-workspace.md` | Usar Rust workspace con core/storage/cli |
| `0002-sqlite-local-persistence.md` | Usar SQLite en `.relascope/graph.db` |
| `0003-local-only-repositories.md` | Solo repos locales en 0.0.1 |
| `0004-canonical-path-limitation.md` | Paths canonicalizados con limitación de portabilidad |

---

## 12. Cómo leer el código por primera vez

No empieces por `main.rs` completo. Seguí este orden:

1. `README.md`
2. `tutorial/onboarding-relascope-0-0-1.md`
3. `openspec/specs/workspace-inventory/spec.md`
4. `crates/core/src/repository.rs`
5. `crates/core/src/classify.rs`
6. `crates/core/src/scan.rs`
7. `crates/storage-sqlite/src/lib.rs`
8. `crates/cli/src/main.rs`
9. `crates/cli/tests/cli.rs`

La razón: primero entendés el producto, después las reglas, después la implementación.

---

## 13. Cómo agregar una mejora chica

Ejemplo: agregar reconocimiento de `.env.example` como configuración.

### Paso 1: encontrar dónde vive la lógica

Archivo:

```text
crates/core/src/classify.rs
```

### Paso 2: agregar o ajustar test

Buscá:

```rust
#[cfg(test)]
mod tests
```

Agregá una expectativa nueva.

### Paso 3: implementar

Modificar la función correspondiente.

### Paso 4: verificar

```powershell
cargo fmt
cargo test
```

### Regla

Cada comportamiento nuevo debe tener test o evidencia manual.

---

## 14. Troubleshooting

## `cargo: command not found`

Rust no está instalado o no está en PATH.

Instalación recomendada en Windows:

```powershell
winget install Rustlang.Rustup
```

Cerrá y reabrí PowerShell, luego:

```powershell
cargo --version
rustc --version
```

## Error al borrar `relascope-smoke`

Si estás parado dentro de la carpeta, Windows no la deja borrar.

Solución:

```powershell
cd C:\Users\MAXIMO\Desktop\relascope
Remove-Item -Recurse -Force $smoke
```

## `repo add` dice que falta `<PATH>`

El comando necesita la ruta al repo:

```powershell
relascope repo add <path> --id api
```

Con `cargo run`, todo lo que va después de `--` es para Relascope:

```powershell
cargo run -p relascope-cli -- repo add C:\ruta\repo --id api
```

## `status` muestra un repo como available aunque lo moví

Eso es correcto si no corriste `scan`.

`status` muestra estado persistido. Para actualizar disponibilidad:

```powershell
relascope scan
```

---

## 15. Qué sigue después de 0.0.1

El próximo paso recomendado es `0.0.2-a — Minimal Graph Schema`.

No conviene saltar directo a imports complejos. Primero hay que crear el esqueleto del grafo:

- tabla `nodes`;
- tabla `edges`;
- tabla `evidence`;
- nodos `Workspace`, `Repository`, `File`;
- relación `CONTAINS`;
- evidencia básica.

Después recién:

- módulos;
- imports Python/TypeScript;
- consulta de dependencias.

---

## Checklist de onboarding

- [ ] Sé qué problema resuelve Relascope.
- [ ] Entiendo que 0.0.1 solo hace inventario.
- [ ] Corrí `cargo fmt`.
- [ ] Corrí `cargo test`.
- [ ] Sé ejecutar `relascope init`.
- [ ] Sé agregar repos con `repo add`.
- [ ] Sé correr `scan`.
- [ ] Sé que `status` no reescanea.
- [ ] Sé dónde están core, storage y CLI.
- [ ] Sé dónde está la spec activa.
- [ ] Sé que el próximo milestone es grafo mínimo.
