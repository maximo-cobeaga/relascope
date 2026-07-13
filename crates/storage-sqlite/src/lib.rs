use chrono::Utc;
use relascope_core::{
    contains_edge_id, detect_imports, evidence_id, file_node_id, imports_edge_id, module_node_id,
    repository_node_id, unresolved_module_node_id, workspace_node_id, FileChangeSummary,
    FileInventoryRecord, GraphEdge, GraphEvidence, GraphNode, GraphSummary, ImportFact,
    PreviousFileState, RepositoryRecord, ScanSummary, WorkspaceConfig, NODE_KIND_FILE,
    NODE_KIND_MODULE, NODE_KIND_REPOSITORY, NODE_KIND_WORKSPACE, ORIGIN_DETERMINISTIC,
    RELATION_CONTAINS, RELATION_IMPORTS, STATUS_ACTIVE, STATUS_STALE, STATUS_UNVERIFIED,
};
use serde::Serialize;
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct LastScan {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub summary: ScanSummary,
}

#[derive(Debug, Clone)]
pub struct WorkspaceStatus {
    pub workspace_id: String,
    pub workspace_name: String,
    pub root_path: PathBuf,
    pub repositories: Vec<RepositoryRecord>,
    pub last_scan: Option<LastScan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportEdgeView {
    pub source: String,
    pub target: String,
    pub status: String,
    pub file_path: Option<String>,
    pub line: Option<i64>,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub workspace_id: String,
    pub workspace_name: String,
    pub config_ok: bool,
    pub database_ok: bool,
    pub repositories_total: u64,
    pub repositories_available: u64,
    pub repositories_unavailable: u64,
    pub last_scan_status: Option<String>,
    pub stale_nodes: u64,
    pub stale_edges: u64,
    pub unverified_imports: u64,
    pub exclusions: Vec<String>,
}

impl SqliteStore {
    pub async fn open(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    pub async fn migrate(&self) -> Result<()> {
        let statements = [
            r#"
            CREATE TABLE IF NOT EXISTS workspaces(
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              root_path TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS repositories(
              id TEXT PRIMARY KEY,
              workspace_id TEXT NOT NULL,
              visible_id TEXT NOT NULL,
              display_name TEXT NOT NULL,
              path TEXT NOT NULL,
              path_kind TEXT NOT NULL,
              availability TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(workspace_id, visible_id)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS scans(
              id TEXT PRIMARY KEY,
              workspace_id TEXT NOT NULL,
              started_at TEXT NOT NULL,
              finished_at TEXT,
              status TEXT NOT NULL,
              summary_json TEXT NOT NULL
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS files(
              id TEXT PRIMARY KEY,
              workspace_id TEXT NOT NULL,
              repository_id TEXT NOT NULL,
              relative_path TEXT NOT NULL,
              kind TEXT NOT NULL,
              language TEXT,
              size_bytes INTEGER,
              content_hash TEXT,
              modified_at TEXT,
              scan_id TEXT NOT NULL,
              metadata_json TEXT NOT NULL,
              UNIQUE(repository_id, relative_path, scan_id)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS nodes(
              id TEXT PRIMARY KEY,
              workspace_id TEXT NOT NULL,
              repository_id TEXT,
              kind TEXT NOT NULL,
              qualified_name TEXT NOT NULL,
              display_name TEXT NOT NULL,
              fingerprint TEXT,
              status TEXT NOT NULL DEFAULT 'active',
              metadata_json TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS edges(
              id TEXT PRIMARY KEY,
              workspace_id TEXT NOT NULL,
              source_node_id TEXT NOT NULL,
              target_node_id TEXT NOT NULL,
              relation_type TEXT NOT NULL,
              origin_type TEXT NOT NULL,
              origin_id TEXT,
              confidence REAL NOT NULL,
              status TEXT NOT NULL,
              metadata_json TEXT NOT NULL,
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              UNIQUE(workspace_id, source_node_id, target_node_id, relation_type)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS evidence(
              id TEXT PRIMARY KEY,
              edge_id TEXT,
              node_id TEXT,
              repository_id TEXT,
              file_path TEXT,
              start_line INTEGER,
              end_line INTEGER,
              content_hash TEXT,
              excerpt TEXT,
              metadata_json TEXT NOT NULL
            );
            "#,
            "CREATE INDEX IF NOT EXISTS idx_nodes_workspace_kind ON nodes(workspace_id, kind);",
            "CREATE INDEX IF NOT EXISTS idx_edges_workspace_relation ON edges(workspace_id, relation_type);",
        ];

        for statement in statements {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        self.ensure_node_status_column().await?;

        Ok(())
    }

    async fn ensure_node_status_column(&self) -> Result<()> {
        let columns = sqlx::query("PRAGMA table_info(nodes)")
            .fetch_all(&self.pool)
            .await?;
        let has_status = columns.iter().any(|row| {
            let name: String = row.get("name");
            name == "status"
        });

        if !has_status {
            sqlx::query("ALTER TABLE nodes ADD COLUMN status TEXT NOT NULL DEFAULT 'active'")
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn upsert_workspace(&self, config: &WorkspaceConfig, root_path: &Path) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO workspaces(id, name, root_path, created_at, updated_at)
            VALUES(?1, ?2, ?3, ?4, ?4)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              root_path = excluded.root_path,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(&config.workspace.id)
        .bind(&config.workspace.name)
        .bind(root_path.to_string_lossy().to_string())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_repository(
        &self,
        workspace_id: &str,
        visible_id: &str,
        display_name: &str,
        path: &Path,
    ) -> Result<RepositoryRecord> {
        let now = Utc::now().to_rfc3339();
        let repository = RepositoryRecord {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            visible_id: visible_id.to_string(),
            display_name: display_name.to_string(),
            path: path.to_path_buf(),
            path_kind: "canonical-local".to_string(),
            availability: "available".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        sqlx::query(
            r#"
            INSERT INTO repositories(
              id, workspace_id, visible_id, display_name, path, path_kind,
              availability, created_at, updated_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&repository.id)
        .bind(&repository.workspace_id)
        .bind(&repository.visible_id)
        .bind(&repository.display_name)
        .bind(repository.path.to_string_lossy().to_string())
        .bind(&repository.path_kind)
        .bind(&repository.availability)
        .bind(&repository.created_at)
        .bind(&repository.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(repository)
    }

    pub async fn list_repositories(&self, workspace_id: &str) -> Result<Vec<RepositoryRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, visible_id, display_name, path, path_kind,
                   availability, created_at, updated_at
            FROM repositories
            WHERE workspace_id = ?1
            ORDER BY visible_id ASC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(repository_from_row).collect())
    }

    pub async fn visible_id_exists(&self, workspace_id: &str, visible_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM repositories WHERE workspace_id = ?1 AND visible_id = ?2",
        )
        .bind(workspace_id)
        .bind(visible_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn remove_repository_by_visible_id(
        &self,
        workspace_id: &str,
        visible_id: &str,
    ) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM repositories WHERE workspace_id = ?1 AND visible_id = ?2")
                .bind(workspace_id)
                .bind(visible_id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn doctor_report(&self, config: &WorkspaceConfig) -> Result<DoctorReport> {
        let repositories = self.list_repositories(&config.workspace.id).await?;
        let repositories_total = repositories.len() as u64;
        let repositories_available = repositories
            .iter()
            .filter(|repository| repository.path.is_dir())
            .count() as u64;
        let repositories_unavailable = repositories_total.saturating_sub(repositories_available);
        let last_scan_status = self
            .last_scan(&config.workspace.id)
            .await?
            .map(|scan| scan.status);
        let stale_nodes = self
            .count_nodes_by_status(&config.workspace.id, STATUS_STALE)
            .await?;
        let stale_edges = self
            .count_edges_by_status(&config.workspace.id, STATUS_STALE)
            .await?;
        let unverified_imports = self
            .count_edges_by_relation_and_status(
                &config.workspace.id,
                RELATION_IMPORTS,
                STATUS_UNVERIFIED,
            )
            .await?;

        Ok(DoctorReport {
            workspace_id: config.workspace.id.clone(),
            workspace_name: config.workspace.name.clone(),
            config_ok: true,
            database_ok: true,
            repositories_total,
            repositories_available,
            repositories_unavailable,
            last_scan_status,
            stale_nodes,
            stale_edges,
            unverified_imports,
            exclusions: config.analysis.exclude.clone(),
        })
    }

    async fn count_nodes_by_status(&self, workspace_id: &str, status: &str) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM nodes WHERE workspace_id = ?1 AND status = ?2",
        )
        .bind(workspace_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        Ok(count as u64)
    }

    async fn count_edges_by_status(&self, workspace_id: &str, status: &str) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM edges WHERE workspace_id = ?1 AND status = ?2",
        )
        .bind(workspace_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        Ok(count as u64)
    }

    async fn count_edges_by_relation_and_status(
        &self,
        workspace_id: &str,
        relation_type: &str,
        status: &str,
    ) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM edges WHERE workspace_id = ?1 AND relation_type = ?2 AND status = ?3",
        )
        .bind(workspace_id)
        .bind(relation_type)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        Ok(count as u64)
    }

    pub async fn update_repository_availability(
        &self,
        repository_id: &str,
        availability: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE repositories SET availability = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(availability)
            .bind(Utc::now().to_rfc3339())
            .bind(repository_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn begin_scan(&self, workspace_id: &str) -> Result<String> {
        let scan_id = Uuid::new_v4().to_string();
        let summary = serde_json::to_string(&ScanSummary::default())?;
        sqlx::query(
            r#"
            INSERT INTO scans(id, workspace_id, started_at, finished_at, status, summary_json)
            VALUES(?1, ?2, ?3, NULL, 'running', ?4)
            "#,
        )
        .bind(&scan_id)
        .bind(workspace_id)
        .bind(Utc::now().to_rfc3339())
        .bind(summary)
        .execute(&self.pool)
        .await?;
        Ok(scan_id)
    }

    pub async fn persist_scan_files(&self, files: &[FileInventoryRecord]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for file in files {
            sqlx::query(
                r#"
                INSERT INTO files(
                  id, workspace_id, repository_id, relative_path, kind, language,
                  size_bytes, content_hash, modified_at, scan_id, metadata_json
                ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
            )
            .bind(&file.id)
            .bind(&file.workspace_id)
            .bind(&file.repository_id)
            .bind(&file.relative_path)
            .bind(&file.kind)
            .bind(&file.language)
            .bind(file.size_bytes)
            .bind(&file.content_hash)
            .bind(&file.modified_at)
            .bind(&file.scan_id)
            .bind(&file.metadata_json)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn materialize_minimal_graph(
        &self,
        config: &WorkspaceConfig,
        repositories: &[RepositoryRecord],
        files: &[FileInventoryRecord],
        scan_id: &str,
    ) -> Result<()> {
        let workspace_node_id = workspace_node_id(&config.workspace.id);
        self.upsert_node(&GraphNode {
            id: workspace_node_id.clone(),
            workspace_id: config.workspace.id.clone(),
            repository_id: None,
            kind: NODE_KIND_WORKSPACE.to_string(),
            qualified_name: config.workspace.id.clone(),
            display_name: config.workspace.name.clone(),
            fingerprint: Some(config.workspace.id.clone()),
            status: STATUS_ACTIVE.to_string(),
            metadata_json: json!({ "source": "workspace_config" }).to_string(),
        })
        .await?;

        for repository in repositories {
            let repository_node_id = repository_node_id(&config.workspace.id, &repository.id);
            self.upsert_node(&GraphNode {
                id: repository_node_id.clone(),
                workspace_id: config.workspace.id.clone(),
                repository_id: Some(repository.id.clone()),
                kind: NODE_KIND_REPOSITORY.to_string(),
                qualified_name: repository.visible_id.clone(),
                display_name: repository.visible_id.clone(),
                fingerprint: Some(repository.id.clone()),
                status: STATUS_ACTIVE.to_string(),
                metadata_json: json!({
                    "source": "repository_registration",
                    "path": repository.path.to_string_lossy(),
                    "availability": &repository.availability,
                    "path_kind": &repository.path_kind,
                })
                .to_string(),
            })
            .await?;

            let edge_id = contains_edge_id(
                &config.workspace.id,
                &workspace_node_id,
                &repository_node_id,
            );
            self.upsert_edge(&GraphEdge {
                id: edge_id.clone(),
                workspace_id: config.workspace.id.clone(),
                source_node_id: workspace_node_id.clone(),
                target_node_id: repository_node_id.clone(),
                relation_type: RELATION_CONTAINS.to_string(),
                origin_type: ORIGIN_DETERMINISTIC.to_string(),
                origin_id: Some(repository.id.clone()),
                confidence: 1.0,
                status: STATUS_ACTIVE.to_string(),
                metadata_json: json!({ "source": "repository_registration" }).to_string(),
            })
            .await?;
            self.upsert_evidence(&GraphEvidence {
                id: evidence_id(&["repository_registration", scan_id, &repository.id]),
                edge_id: Some(edge_id),
                node_id: Some(repository_node_id),
                repository_id: Some(repository.id.clone()),
                file_path: None,
                start_line: None,
                end_line: None,
                content_hash: None,
                excerpt: None,
                metadata_json: json!({
                    "source": "repository_registration",
                    "scan_id": scan_id,
                    "path": repository.path.to_string_lossy(),
                })
                .to_string(),
            })
            .await?;
        }

        for file in files {
            let Some(repository) = repositories
                .iter()
                .find(|repo| repo.id.as_str() == file.repository_id.as_str())
            else {
                continue;
            };
            let repository_node_id = repository_node_id(&config.workspace.id, &repository.id);
            let file_node_id =
                file_node_id(&config.workspace.id, &repository.id, &file.relative_path);
            self.upsert_node(&GraphNode {
                id: file_node_id.clone(),
                workspace_id: config.workspace.id.clone(),
                repository_id: Some(repository.id.clone()),
                kind: NODE_KIND_FILE.to_string(),
                qualified_name: format!("{}/{}", repository.visible_id, file.relative_path),
                display_name: display_name_for_file(&file.relative_path),
                fingerprint: file.content_hash.clone(),
                status: STATUS_ACTIVE.to_string(),
                metadata_json: json!({
                    "source": "file_inventory",
                    "scan_id": scan_id,
                    "kind": &file.kind,
                    "language": &file.language,
                    "size_bytes": file.size_bytes,
                })
                .to_string(),
            })
            .await?;

            let edge_id =
                contains_edge_id(&config.workspace.id, &repository_node_id, &file_node_id);
            self.upsert_edge(&GraphEdge {
                id: edge_id.clone(),
                workspace_id: config.workspace.id.clone(),
                source_node_id: repository_node_id,
                target_node_id: file_node_id.clone(),
                relation_type: RELATION_CONTAINS.to_string(),
                origin_type: ORIGIN_DETERMINISTIC.to_string(),
                origin_id: Some(scan_id.to_string()),
                confidence: 1.0,
                status: STATUS_ACTIVE.to_string(),
                metadata_json: json!({ "source": "file_inventory" }).to_string(),
            })
            .await?;
            self.upsert_evidence(&GraphEvidence {
                id: evidence_id(&[
                    "file_inventory",
                    scan_id,
                    &repository.id,
                    &file.relative_path,
                ]),
                edge_id: Some(edge_id),
                node_id: Some(file_node_id),
                repository_id: Some(repository.id.clone()),
                file_path: Some(file.relative_path.clone()),
                start_line: None,
                end_line: None,
                content_hash: file.content_hash.clone(),
                excerpt: None,
                metadata_json: json!({ "source": "file_inventory", "scan_id": scan_id })
                    .to_string(),
            })
            .await?;
        }

        Ok(())
    }

    pub async fn materialize_shallow_imports(
        &self,
        config: &WorkspaceConfig,
        repositories: &[RepositoryRecord],
        files: &[FileInventoryRecord],
        scan_id: &str,
    ) -> Result<()> {
        let files_by_repository = files_by_repository(files);

        for repository in repositories {
            let Some(repository_files) = files_by_repository.get(&repository.id) else {
                continue;
            };
            if !repository.path.is_dir() {
                continue;
            }

            let known_files = repository_files
                .iter()
                .map(|file| file.relative_path.clone())
                .collect::<Vec<_>>();
            let code_files = repository_files
                .iter()
                .filter(|file| is_supported_code_language(file.language.as_deref()))
                .copied()
                .collect::<Vec<_>>();

            for file in &code_files {
                self.materialize_source_module(config, repository, file, scan_id)
                    .await?;
            }

            for file in code_files {
                let absolute_path = repository
                    .path
                    .join(path_from_relative(&file.relative_path));
                let Ok(content) = fs::read_to_string(&absolute_path) else {
                    continue;
                };
                let language = file.language.as_deref().unwrap_or_default();
                let import_facts =
                    detect_imports(&file.relative_path, language, &content, &known_files);
                for fact in import_facts {
                    self.materialize_import_fact(config, repository, file, &fact, scan_id)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn materialize_source_module(
        &self,
        config: &WorkspaceConfig,
        repository: &RepositoryRecord,
        file: &FileInventoryRecord,
        scan_id: &str,
    ) -> Result<String> {
        let file_node_id = file_node_id(&config.workspace.id, &repository.id, &file.relative_path);
        let source_module_node_id =
            module_node_id(&config.workspace.id, &repository.id, &file.relative_path);
        self.upsert_node(&GraphNode {
            id: source_module_node_id.clone(),
            workspace_id: config.workspace.id.clone(),
            repository_id: Some(repository.id.clone()),
            kind: NODE_KIND_MODULE.to_string(),
            qualified_name: format!("{}/{}", repository.visible_id, file.relative_path),
            display_name: file.relative_path.clone(),
            fingerprint: file.content_hash.clone(),
            status: STATUS_ACTIVE.to_string(),
            metadata_json: json!({
                "source": "file_inventory",
                "module_role": "source",
                "scan_id": scan_id,
                "language": &file.language,
                "relative_path": &file.relative_path,
            })
            .to_string(),
        })
        .await?;

        let edge_id = contains_edge_id(&config.workspace.id, &file_node_id, &source_module_node_id);
        self.upsert_edge(&GraphEdge {
            id: edge_id,
            workspace_id: config.workspace.id.clone(),
            source_node_id: file_node_id,
            target_node_id: source_module_node_id.clone(),
            relation_type: RELATION_CONTAINS.to_string(),
            origin_type: ORIGIN_DETERMINISTIC.to_string(),
            origin_id: Some(scan_id.to_string()),
            confidence: 1.0,
            status: STATUS_ACTIVE.to_string(),
            metadata_json: json!({ "source": "module_materialization" }).to_string(),
        })
        .await?;

        Ok(source_module_node_id)
    }

    async fn materialize_import_fact(
        &self,
        config: &WorkspaceConfig,
        repository: &RepositoryRecord,
        source_file: &FileInventoryRecord,
        fact: &ImportFact,
        scan_id: &str,
    ) -> Result<()> {
        let source_module_node_id = module_node_id(
            &config.workspace.id,
            &repository.id,
            &source_file.relative_path,
        );
        let (target_module_node_id, target_display, resolution, status) =
            if let Some(resolved_path) = &fact.resolved_relative_path {
                (
                    module_node_id(&config.workspace.id, &repository.id, resolved_path),
                    format!("{}/{}", repository.visible_id, resolved_path),
                    "resolved",
                    STATUS_ACTIVE,
                )
            } else {
                (
                    unresolved_module_node_id(
                        &config.workspace.id,
                        &repository.id,
                        &fact.target_specifier,
                    ),
                    format!("{}:{}", repository.visible_id, fact.target_specifier),
                    "unresolved",
                    STATUS_UNVERIFIED,
                )
            };

        if fact.resolved_relative_path.is_none() {
            self.upsert_node(&GraphNode {
                id: target_module_node_id.clone(),
                workspace_id: config.workspace.id.clone(),
                repository_id: Some(repository.id.clone()),
                kind: NODE_KIND_MODULE.to_string(),
                qualified_name: target_display.clone(),
                display_name: fact.target_specifier.clone(),
                fingerprint: None,
                status: STATUS_UNVERIFIED.to_string(),
                metadata_json: json!({
                    "source": "shallow_import_detector",
                    "module_role": "unresolved_target",
                    "specifier": &fact.target_specifier,
                    "resolution": resolution,
                })
                .to_string(),
            })
            .await?;
        }

        let edge_id = imports_edge_id(
            &config.workspace.id,
            &source_module_node_id,
            &target_module_node_id,
        );
        self.upsert_edge(&GraphEdge {
            id: edge_id.clone(),
            workspace_id: config.workspace.id.clone(),
            source_node_id: source_module_node_id,
            target_node_id: target_module_node_id.clone(),
            relation_type: RELATION_IMPORTS.to_string(),
            origin_type: ORIGIN_DETERMINISTIC.to_string(),
            origin_id: Some(scan_id.to_string()),
            confidence: 1.0,
            status: status.to_string(),
            metadata_json: json!({
                "source": "shallow_import_detector",
                "specifier": &fact.target_specifier,
                "language": &fact.source_language,
                "resolution": resolution,
            })
            .to_string(),
        })
        .await?;

        let line = i64::from(fact.line_number);
        self.upsert_evidence(&GraphEvidence {
            id: evidence_id(&[
                "shallow_import",
                &config.workspace.id,
                &repository.id,
                &fact.source_relative_path,
                &fact.line_number.to_string(),
                &fact.excerpt,
            ]),
            edge_id: Some(edge_id),
            node_id: Some(target_module_node_id),
            repository_id: Some(repository.id.clone()),
            file_path: Some(fact.source_relative_path.clone()),
            start_line: Some(line),
            end_line: Some(line),
            content_hash: source_file.content_hash.clone(),
            excerpt: Some(fact.excerpt.clone()),
            metadata_json: json!({
                "source": "shallow_import_detector",
                "scan_id": scan_id,
                "specifier": &fact.target_specifier,
                "resolution": resolution,
            })
            .to_string(),
        })
        .await?;

        Ok(())
    }

    async fn upsert_node(&self, node: &GraphNode) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO nodes(
              id, workspace_id, repository_id, kind, qualified_name, display_name,
              fingerprint, status, metadata_json, created_at, updated_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
            ON CONFLICT(id) DO UPDATE SET
              workspace_id = excluded.workspace_id,
              repository_id = excluded.repository_id,
              kind = excluded.kind,
              qualified_name = excluded.qualified_name,
              display_name = excluded.display_name,
              fingerprint = excluded.fingerprint,
              status = excluded.status,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(&node.id)
        .bind(&node.workspace_id)
        .bind(&node.repository_id)
        .bind(&node.kind)
        .bind(&node.qualified_name)
        .bind(&node.display_name)
        .bind(&node.fingerprint)
        .bind(&node.status)
        .bind(&node.metadata_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn upsert_edge(&self, edge: &GraphEdge) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO edges(
              id, workspace_id, source_node_id, target_node_id, relation_type,
              origin_type, origin_id, confidence, status, metadata_json,
              first_seen_at, last_seen_at
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
            ON CONFLICT(workspace_id, source_node_id, target_node_id, relation_type) DO UPDATE SET
              origin_type = excluded.origin_type,
              origin_id = excluded.origin_id,
              confidence = excluded.confidence,
              status = excluded.status,
              metadata_json = excluded.metadata_json,
              last_seen_at = excluded.last_seen_at
            "#,
        )
        .bind(&edge.id)
        .bind(&edge.workspace_id)
        .bind(&edge.source_node_id)
        .bind(&edge.target_node_id)
        .bind(&edge.relation_type)
        .bind(&edge.origin_type)
        .bind(&edge.origin_id)
        .bind(edge.confidence)
        .bind(&edge.status)
        .bind(&edge.metadata_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn upsert_evidence(&self, evidence: &GraphEvidence) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO evidence(
              id, edge_id, node_id, repository_id, file_path, start_line, end_line,
              content_hash, excerpt, metadata_json
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
              edge_id = excluded.edge_id,
              node_id = excluded.node_id,
              repository_id = excluded.repository_id,
              file_path = excluded.file_path,
              start_line = excluded.start_line,
              end_line = excluded.end_line,
              content_hash = excluded.content_hash,
              excerpt = excluded.excerpt,
              metadata_json = excluded.metadata_json
            "#,
        )
        .bind(&evidence.id)
        .bind(&evidence.edge_id)
        .bind(&evidence.node_id)
        .bind(&evidence.repository_id)
        .bind(&evidence.file_path)
        .bind(evidence.start_line)
        .bind(evidence.end_line)
        .bind(&evidence.content_hash)
        .bind(&evidence.excerpt)
        .bind(&evidence.metadata_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn graph_summary(&self, workspace_id: &str) -> Result<GraphSummary> {
        let total_nodes: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE workspace_id = ?1")
                .bind(workspace_id)
                .fetch_one(&self.pool)
                .await?;
        let total_edges: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM edges WHERE workspace_id = ?1")
                .bind(workspace_id)
                .fetch_one(&self.pool)
                .await?;

        let node_rows = sqlx::query(
            "SELECT kind, COUNT(*) AS count FROM nodes WHERE workspace_id = ?1 GROUP BY kind ORDER BY kind",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;
        let edge_rows = sqlx::query(
            "SELECT relation_type, COUNT(*) AS count FROM edges WHERE workspace_id = ?1 GROUP BY relation_type ORDER BY relation_type",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        let nodes_by_kind = rows_to_counts(node_rows, "kind");
        let edges_by_relation = rows_to_counts(edge_rows, "relation_type");

        Ok(GraphSummary {
            total_nodes: total_nodes as u64,
            nodes_by_kind,
            total_edges: total_edges as u64,
            edges_by_relation,
        })
    }

    pub async fn previous_completed_scan_files(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<PreviousFileState>> {
        let Some(scan_id) = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id
            FROM scans
            WHERE workspace_id = ?1 AND status = 'completed'
            ORDER BY finished_at DESC, started_at DESC
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(Vec::new());
        };

        let rows = sqlx::query(
            r#"
            SELECT repository_id, relative_path, kind, language, size_bytes, content_hash, modified_at
            FROM files
            WHERE workspace_id = ?1 AND scan_id = ?2
            "#,
        )
        .bind(workspace_id)
        .bind(scan_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| PreviousFileState {
                repository_id: row.get("repository_id"),
                relative_path: row.get("relative_path"),
                kind: row.get("kind"),
                language: row.get("language"),
                size_bytes: row.get("size_bytes"),
                content_hash: row.get("content_hash"),
                modified_at: row.get("modified_at"),
            })
            .collect())
    }

    pub async fn mark_stale_for_file_changes(
        &self,
        workspace_id: &str,
        changes: &FileChangeSummary,
    ) -> Result<()> {
        for removed in &changes.removed_files {
            self.mark_removed_file_facts_stale(workspace_id, removed)
                .await?;
        }
        for modified in &changes.modified_files {
            if is_supported_code_language(modified.language.as_deref()) {
                let module_id = module_node_id(
                    workspace_id,
                    &modified.repository_id,
                    &modified.relative_path,
                );
                self.mark_outgoing_imports_stale(&module_id).await?;
            }
        }
        Ok(())
    }

    async fn mark_removed_file_facts_stale(
        &self,
        workspace_id: &str,
        removed: &PreviousFileState,
    ) -> Result<()> {
        let file_id = file_node_id(workspace_id, &removed.repository_id, &removed.relative_path);
        self.mark_node_stale(&file_id).await?;
        self.mark_contains_edges_touching_node_stale(&file_id)
            .await?;

        if is_supported_code_language(removed.language.as_deref()) {
            let module_id =
                module_node_id(workspace_id, &removed.repository_id, &removed.relative_path);
            self.mark_node_stale(&module_id).await?;
            self.mark_contains_edges_touching_node_stale(&module_id)
                .await?;
            self.mark_imports_touching_module_stale(&module_id).await?;
        }

        Ok(())
    }

    async fn mark_node_stale(&self, node_id: &str) -> Result<()> {
        sqlx::query("UPDATE nodes SET status = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(STATUS_STALE)
            .bind(Utc::now().to_rfc3339())
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_contains_edges_touching_node_stale(&self, node_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE edges
            SET status = ?1, last_seen_at = ?2
            WHERE relation_type = ?3 AND (source_node_id = ?4 OR target_node_id = ?4)
            "#,
        )
        .bind(STATUS_STALE)
        .bind(Utc::now().to_rfc3339())
        .bind(RELATION_CONTAINS)
        .bind(node_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_imports_touching_module_stale(&self, module_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE edges
            SET status = ?1, last_seen_at = ?2
            WHERE relation_type = ?3 AND (source_node_id = ?4 OR target_node_id = ?4)
            "#,
        )
        .bind(STATUS_STALE)
        .bind(Utc::now().to_rfc3339())
        .bind(RELATION_IMPORTS)
        .bind(module_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_outgoing_imports_stale(&self, source_module_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE edges
            SET status = ?1, last_seen_at = ?2
            WHERE relation_type = ?3 AND source_node_id = ?4
            "#,
        )
        .bind(STATUS_STALE)
        .bind(Utc::now().to_rfc3339())
        .bind(RELATION_IMPORTS)
        .bind(source_module_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_import_edges(&self, workspace_id: &str) -> Result<Vec<ImportEdgeView>> {
        let rows = sqlx::query(
            r#"
            SELECT
                source.qualified_name AS source_name,
                target.qualified_name AS target_name,
                edges.status AS edge_status,
                evidence.file_path AS file_path,
                evidence.start_line AS line,
                evidence.excerpt AS excerpt
            FROM edges
            JOIN nodes source ON source.id = edges.source_node_id
            JOIN nodes target ON target.id = edges.target_node_id
            LEFT JOIN evidence ON evidence.edge_id = edges.id
            WHERE edges.workspace_id = ?1
              AND edges.relation_type = 'IMPORTS'
            ORDER BY source.qualified_name, target.qualified_name, evidence.file_path, evidence.start_line
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ImportEdgeView {
                source: row.get("source_name"),
                target: row.get("target_name"),
                status: row.get("edge_status"),
                file_path: row.get("file_path"),
                line: row.get("line"),
                excerpt: row.get("excerpt"),
            })
            .collect())
    }

    pub async fn finish_scan(
        &self,
        scan_id: &str,
        status: &str,
        summary: &ScanSummary,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE scans
            SET finished_at = ?1, status = ?2, summary_json = ?3
            WHERE id = ?4
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(status)
        .bind(serde_json::to_string(summary)?)
        .bind(scan_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn status(&self, config: &WorkspaceConfig) -> Result<WorkspaceStatus> {
        let workspace_row = sqlx::query("SELECT root_path FROM workspaces WHERE id = ?1 LIMIT 1")
            .bind(&config.workspace.id)
            .fetch_optional(&self.pool)
            .await?;

        let root_path = workspace_row
            .as_ref()
            .map(|row| PathBuf::from(row.get::<String, _>("root_path")))
            .unwrap_or_default();

        let repositories = self.list_repositories(&config.workspace.id).await?;
        let last_scan = self.last_scan(&config.workspace.id).await?;

        Ok(WorkspaceStatus {
            workspace_id: config.workspace.id.clone(),
            workspace_name: config.workspace.name.clone(),
            root_path,
            repositories,
            last_scan,
        })
    }

    async fn last_scan(&self, workspace_id: &str) -> Result<Option<LastScan>> {
        let Some(row) = sqlx::query(
            r#"
            SELECT id, started_at, finished_at, status, summary_json
            FROM scans
            WHERE workspace_id = ?1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let summary_json: String = row.get("summary_json");
        Ok(Some(LastScan {
            id: row.get("id"),
            started_at: row.get("started_at"),
            finished_at: row.get("finished_at"),
            status: row.get("status"),
            summary: serde_json::from_str(&summary_json)?,
        }))
    }
}

fn repository_from_row(row: sqlx::sqlite::SqliteRow) -> RepositoryRecord {
    RepositoryRecord {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        visible_id: row.get("visible_id"),
        display_name: row.get("display_name"),
        path: PathBuf::from(row.get::<String, _>("path")),
        path_kind: row.get("path_kind"),
        availability: row.get("availability"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn rows_to_counts(rows: Vec<sqlx::sqlite::SqliteRow>, key_column: &str) -> BTreeMap<String, u64> {
    rows.into_iter()
        .map(|row| {
            let key: String = row.get(key_column);
            let count: i64 = row.get("count");
            (key, count as u64)
        })
        .collect()
}

fn display_name_for_file(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(relative_path)
        .to_string()
}

fn files_by_repository(
    files: &[FileInventoryRecord],
) -> HashMap<String, Vec<&FileInventoryRecord>> {
    let mut grouped: HashMap<String, Vec<&FileInventoryRecord>> = HashMap::new();
    for file in files {
        grouped
            .entry(file.repository_id.clone())
            .or_default()
            .push(file);
    }
    grouped
}

fn is_supported_code_language(language: Option<&str>) -> bool {
    matches!(language, Some("python" | "typescript" | "javascript"))
}

fn path_from_relative(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for part in relative_path.split('/') {
        path.push(part);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use relascope_core::WorkspaceConfig;
    use tempfile::tempdir;

    #[tokio::test]
    async fn persists_repository_and_status() {
        let dir = tempdir().unwrap();
        let db = dir.path().join("graph.db");
        let repo_dir = dir.path().join("api-python");
        std::fs::create_dir_all(&repo_dir).unwrap();
        let config = WorkspaceConfig::new("workspace-id".to_string(), "test".to_string());
        let store = SqliteStore::open(&db).await.unwrap();
        store.upsert_workspace(&config, dir.path()).await.unwrap();
        let repository = store
            .add_repository(&config.workspace.id, "api", "api-python", &repo_dir)
            .await
            .unwrap();

        assert!(store
            .visible_id_exists(&config.workspace.id, "api")
            .await
            .unwrap());
        assert_eq!(repository.visible_id, "api");

        let reopened = SqliteStore::open(&db).await.unwrap();
        let status = reopened.status(&config).await.unwrap();
        assert_eq!(status.repositories.len(), 1);
        assert_eq!(status.repositories[0].id, repository.id);
    }

    #[tokio::test]
    async fn materializes_shallow_imports_and_lists_evidence() {
        let dir = tempdir().unwrap();
        let db = dir.path().join("graph.db");
        let repo_dir = dir.path().join("web-typescript");
        std::fs::create_dir_all(repo_dir.join("src")).unwrap();
        std::fs::write(
            repo_dir.join("src/main.ts"),
            "import { message } from './message';\nexport { message } from './message';\n",
        )
        .unwrap();
        std::fs::write(
            repo_dir.join("src/message.ts"),
            "export const message = 'hello';\n",
        )
        .unwrap();

        let config = WorkspaceConfig::new("workspace-id".to_string(), "test".to_string());
        let store = SqliteStore::open(&db).await.unwrap();
        store.upsert_workspace(&config, dir.path()).await.unwrap();
        let repository = store
            .add_repository(&config.workspace.id, "web", "web-typescript", &repo_dir)
            .await
            .unwrap();
        let files = vec![
            FileInventoryRecord {
                id: "file-row-1".to_string(),
                workspace_id: config.workspace.id.clone(),
                repository_id: repository.id.clone(),
                relative_path: "src/main.ts".to_string(),
                kind: "code".to_string(),
                language: Some("typescript".to_string()),
                size_bytes: Some(50),
                content_hash: Some("hash-1".to_string()),
                modified_at: None,
                scan_id: "scan-1".to_string(),
                metadata_json: "{}".to_string(),
            },
            FileInventoryRecord {
                id: "file-row-2".to_string(),
                workspace_id: config.workspace.id.clone(),
                repository_id: repository.id.clone(),
                relative_path: "src/message.ts".to_string(),
                kind: "code".to_string(),
                language: Some("typescript".to_string()),
                size_bytes: Some(30),
                content_hash: Some("hash-2".to_string()),
                modified_at: None,
                scan_id: "scan-1".to_string(),
                metadata_json: "{}".to_string(),
            },
        ];

        store
            .materialize_minimal_graph(&config, &[repository.clone()], &files, "scan-1")
            .await
            .unwrap();
        store
            .materialize_shallow_imports(&config, &[repository], &files, "scan-1")
            .await
            .unwrap();
        store
            .materialize_shallow_imports(
                &config,
                &store.list_repositories(&config.workspace.id).await.unwrap(),
                &files,
                "scan-1",
            )
            .await
            .unwrap();

        let summary = store.graph_summary(&config.workspace.id).await.unwrap();
        assert_eq!(summary.nodes_by_kind.get(NODE_KIND_MODULE), Some(&2));
        assert_eq!(summary.edges_by_relation.get(RELATION_IMPORTS), Some(&1));

        let imports = store.list_import_edges(&config.workspace.id).await.unwrap();
        assert_eq!(imports.len(), 2);
        assert!(imports.iter().any(|import| import.line == Some(1)));
        assert!(imports.iter().any(|import| import.line == Some(2)));
    }

    #[tokio::test]
    async fn materializes_minimal_graph_idempotently() {
        let dir = tempdir().unwrap();
        let db = dir.path().join("graph.db");
        let repo_dir = dir.path().join("api-python");
        std::fs::create_dir_all(&repo_dir).unwrap();
        let config = WorkspaceConfig::new("workspace-id".to_string(), "test".to_string());
        let store = SqliteStore::open(&db).await.unwrap();
        store.upsert_workspace(&config, dir.path()).await.unwrap();
        let repository = store
            .add_repository(&config.workspace.id, "api", "api-python", &repo_dir)
            .await
            .unwrap();
        let files = vec![
            FileInventoryRecord {
                id: "file-row-1".to_string(),
                workspace_id: config.workspace.id.clone(),
                repository_id: repository.id.clone(),
                relative_path: "app.py".to_string(),
                kind: "code".to_string(),
                language: Some("python".to_string()),
                size_bytes: Some(10),
                content_hash: Some("hash-1".to_string()),
                modified_at: None,
                scan_id: "scan-1".to_string(),
                metadata_json: "{}".to_string(),
            },
            FileInventoryRecord {
                id: "file-row-2".to_string(),
                workspace_id: config.workspace.id.clone(),
                repository_id: repository.id.clone(),
                relative_path: "README.md".to_string(),
                kind: "documentation".to_string(),
                language: Some("markdown".to_string()),
                size_bytes: Some(20),
                content_hash: Some("hash-2".to_string()),
                modified_at: None,
                scan_id: "scan-1".to_string(),
                metadata_json: "{}".to_string(),
            },
        ];

        store
            .materialize_minimal_graph(&config, &[repository], &files, "scan-1")
            .await
            .unwrap();
        store
            .materialize_minimal_graph(
                &config,
                &store.list_repositories(&config.workspace.id).await.unwrap(),
                &files,
                "scan-1",
            )
            .await
            .unwrap();

        let summary = store.graph_summary(&config.workspace.id).await.unwrap();
        assert_eq!(summary.total_nodes, 4);
        assert_eq!(summary.nodes_by_kind.get(NODE_KIND_WORKSPACE), Some(&1));
        assert_eq!(summary.nodes_by_kind.get(NODE_KIND_REPOSITORY), Some(&1));
        assert_eq!(summary.nodes_by_kind.get(NODE_KIND_FILE), Some(&2));
        assert_eq!(summary.total_edges, 3);
        assert_eq!(summary.edges_by_relation.get(RELATION_CONTAINS), Some(&3));
    }
}
