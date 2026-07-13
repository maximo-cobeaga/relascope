use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RelascopeError {
    #[error("workspace already exists at {0}")]
    WorkspaceAlreadyExists(PathBuf),

    #[error("no Relascope workspace found from {0}")]
    WorkspaceNotFound(PathBuf),

    #[error("unsupported repository source for 0.0.1: {0}")]
    UnsupportedRepositorySource(String),

    #[error("repository path does not exist or is not a directory: {0}")]
    RepositoryPathInvalid(PathBuf),

    #[error("invalid repository visible id: {0}")]
    InvalidVisibleId(String),

    #[error("duplicate repository visible id: {0}")]
    DuplicateVisibleId(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("glob pattern error: {0}")]
    Glob(#[from] globset::Error),
}

pub type Result<T> = std::result::Result<T, RelascopeError>;
