pub mod classify;
pub mod config;
pub mod error;
pub mod graph;
pub mod imports;
pub mod inventory;
pub mod repository;
pub mod scan;
pub mod workspace;

pub use classify::{classify_path, FileClassification};
pub use config::{default_exclusions, AnalysisConfig, WorkspaceConfig, WorkspaceSection};
pub use error::{RelascopeError, Result};
pub use graph::{
    contains_edge_id, evidence_id, file_node_id, imports_edge_id, module_node_id,
    repository_node_id, unresolved_module_node_id, workspace_node_id, GraphEdge, GraphEvidence,
    GraphNode, GraphSummary, NODE_KIND_FILE, NODE_KIND_MODULE, NODE_KIND_REPOSITORY,
    NODE_KIND_WORKSPACE, ORIGIN_DETERMINISTIC, RELATION_CONTAINS, RELATION_IMPORTS, STATUS_ACTIVE,
    STATUS_STALE, STATUS_UNVERIFIED,
};
pub use imports::{detect_imports, resolve_python_import, resolve_ts_js_import, ImportFact};
pub use inventory::{
    compute_file_changes, FileChangeSummary, FileIdentityKey, FileInventoryRecord,
    PreviousFileState, ScanSummary,
};
pub use repository::{slugify_visible_id, validate_visible_id, RepositoryRecord};
pub use scan::{
    scan_repositories, scan_repositories_with_control, NoopScanProgressReporter,
    ScanCancellationToken, ScanOptions, ScanOutcome, ScanProgressEvent, ScanProgressEventKind,
    ScanProgressReporter,
};
pub use workspace::{find_workspace_root, load_workspace_config, write_workspace_config};
