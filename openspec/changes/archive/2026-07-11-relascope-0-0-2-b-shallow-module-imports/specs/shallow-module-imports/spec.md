# Specification Delta: Shallow Module and Import Detection

## ADDED Requirements

### Requirement: Module graph nodes for recognized code files

Relascope SHALL create `Module` graph nodes for recognized code files during `relascope scan`.

#### Scenario: Python file creates a Module node

- **Given** a registered repository contains a non-excluded `.py` file
- **When** the user runs `relascope scan`
- **Then** Relascope persists a `File` node for the file
- **And** persists a `Module` node representing that Python file
- **And** persists a `CONTAINS` edge from the file node to the module node or an equivalent containment relationship preserving file-to-module ownership

#### Scenario: TypeScript file creates a Module node

- **Given** a registered repository contains a non-excluded `.ts` or `.tsx` file
- **When** the user runs `relascope scan`
- **Then** Relascope persists a `File` node for the file
- **And** persists a `Module` node representing that TypeScript file

#### Scenario: JavaScript file creates a Module node

- **Given** a registered repository contains a non-excluded `.js` or `.jsx` file
- **When** the user runs `relascope scan`
- **Then** Relascope persists a `File` node for the file
- **And** persists a `Module` node representing that JavaScript file

#### Scenario: Code file without imports still creates a Module node

- **Given** a recognized code file contains no import statements
- **When** the user runs `relascope scan`
- **Then** Relascope still persists a `Module` node for the file

### Requirement: Shallow Python import detection

Relascope SHALL detect simple Python imports with a deterministic line-based detector.

#### Scenario: Detect Python import statement

- **Given** a Python file contains `import app.services.users`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `app.services.users`
- **And** stores evidence containing repository ID, file path, line number, and the import line excerpt

#### Scenario: Detect Python from-import statement

- **Given** a Python file contains `from app.models import User`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `app.models`
- **And** stores line-level evidence for the import

#### Scenario: Detect relative Python import statement

- **Given** a Python file contains `from .models import User`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing the relative import specifier
- **And** marks the target or edge as unresolved or unverified unless it can be resolved by simple local file matching

#### Scenario: Ignore commented Python import

- **Given** a Python file contains `# import hidden.module`
- **When** the user runs `relascope scan`
- **Then** Relascope does not create an `IMPORTS` edge for that commented line

### Requirement: Shallow TypeScript and JavaScript import detection

Relascope SHALL detect simple TypeScript and JavaScript import/export-from statements with a deterministic line-based detector.

#### Scenario: Detect named import

- **Given** a TypeScript file contains `import { User } from "./types";`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `./types`
- **And** stores line-level evidence for the import

#### Scenario: Detect default import

- **Given** a TypeScript file contains `import api from "../api/client";`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `../api/client`

#### Scenario: Detect side-effect import

- **Given** a JavaScript file contains `import "./setup";`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `./setup`

#### Scenario: Detect export-from statement

- **Given** a TypeScript file contains `export { foo } from "./foo";`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an `IMPORTS` edge from the current module to a target `Module` node representing `./foo`

#### Scenario: Ignore commented TypeScript import

- **Given** a TypeScript file contains `// import { Hidden } from "./hidden";`
- **When** the user runs `relascope scan`
- **Then** Relascope does not create an `IMPORTS` edge for that commented line

### Requirement: Unresolved import targets are preserved

Relascope SHALL preserve unresolved imports as graph facts instead of dropping them.

#### Scenario: Unresolved import creates target Module node

- **Given** an import target cannot be matched to a known local file by the shallow resolver
- **When** the user runs `relascope scan`
- **Then** Relascope creates or updates a target `Module` node for the import specifier
- **And** marks the node or edge as `unverified` or equivalent
- **And** stores evidence for the import line

#### Scenario: External package import is unverified

- **Given** a Python file contains `import os`
- **When** the user runs `relascope scan`
- **Then** Relascope creates an import graph fact for `os`
- **And** marks it as unverified or external/unresolved in metadata

### Requirement: Import evidence

Relascope SHALL store line-level evidence for detected imports.

#### Scenario: Evidence contains source location and excerpt

- **Given** a detected import appears on line 3 of `src/main.ts`
- **When** Relascope persists the `IMPORTS` edge
- **Then** evidence stores the repository ID
- **And** stores `file_path` as `src/main.ts`
- **And** stores `start_line` and `end_line` as `3`
- **And** stores the import line excerpt

### Requirement: Graph imports command

Relascope SHALL provide a minimal command for inspecting persisted import relationships.

#### Scenario: Graph imports before scan

- **Given** an initialized workspace has no materialized graph yet
- **When** the user runs `relascope graph imports`
- **Then** Relascope reports that no imports have been materialized yet
- **And** exits successfully

#### Scenario: Graph imports after scan

- **Given** a workspace has detected import relationships
- **When** the user runs `relascope graph imports`
- **Then** Relascope reads persisted graph state
- **And** prints source module, target module, relation status, and evidence location for each import edge

#### Scenario: Graph imports does not scan

- **Given** a workspace has persisted import relationships
- **And** a repository path changes after the last scan
- **When** the user runs `relascope graph imports`
- **Then** Relascope reports persisted import state only
- **And** does not walk repository files
- **And** does not update repository availability

### Requirement: Preserve existing graph behavior

Relascope SHALL preserve 0.0.1 and 0.0.2-a behavior while adding shallow imports.

#### Scenario: Existing inventory and graph summary still work

- **Given** a user follows the existing workflow
- **When** they run `init`, `repo add`, `scan`, `status`, and `graph summary`
- **Then** inventory state remains available
- **And** minimal graph counts still include `Workspace`, `Repository`, `File`, and `CONTAINS`
- **And** graph summary also includes `Module` and `IMPORTS` when imports are detected

#### Scenario: Repeated scan does not duplicate import graph facts

- **Given** a workspace has already been scanned and import relationships were materialized
- **When** the user runs `relascope scan` again without changing files
- **Then** Relascope does not duplicate module nodes
- **And** does not duplicate `IMPORTS` edges for the same source, target, and evidence-equivalent import statement

### Requirement: Scope boundaries

Relascope SHALL keep shallow import detection local and deterministic.

#### Scenario: Scan does not perform semantic parsing

- **Given** a repository contains Python, TypeScript, or JavaScript files
- **When** the user runs `relascope scan`
- **Then** Relascope uses shallow line-based detection for this version
- **And** does not invoke Tree-sitter
- **And** does not create symbol, function, class, or interface nodes

#### Scenario: Scan does not use external capabilities

- **Given** a workspace has registered repositories
- **When** the user runs `relascope scan`
- **Then** Relascope does not use AI
- **And** does not require network access
- **And** does not execute code from registered repositories
