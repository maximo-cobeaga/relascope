use crate::config::WorkspaceConfig;
use crate::error::{RelascopeError, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "relascope.yaml";
pub const STATE_DIR: &str = ".relascope";
pub const DB_FILE: &str = "graph.db";

pub fn find_workspace_root(start: &Path) -> Result<PathBuf> {
    let mut current = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if current.join(CONFIG_FILE).is_file() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(RelascopeError::WorkspaceNotFound(start.to_path_buf()));
        }
    }
}

pub fn load_workspace_config(root: &Path) -> Result<WorkspaceConfig> {
    let raw = fs::read_to_string(root.join(CONFIG_FILE))?;
    let config = serde_yaml::from_str(&raw)?;
    Ok(config)
}

pub fn write_workspace_config(root: &Path, config: &WorkspaceConfig) -> Result<()> {
    let raw = serde_yaml::to_string(config)?;
    fs::write(root.join(CONFIG_FILE), raw)?;
    Ok(())
}

pub fn database_path(root: &Path) -> PathBuf {
    root.join(STATE_DIR).join(DB_FILE)
}

pub fn ensure_state_dir(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join(STATE_DIR))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkspaceConfig;
    use tempfile::tempdir;

    #[test]
    fn finds_workspace_from_child_directory() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let child = root.join("a/b");
        fs::create_dir_all(&child).unwrap();
        write_workspace_config(
            root,
            &WorkspaceConfig::new("workspace-id".to_string(), "test".to_string()),
        )
        .unwrap();

        assert_eq!(find_workspace_root(&child).unwrap(), root);
    }
}
