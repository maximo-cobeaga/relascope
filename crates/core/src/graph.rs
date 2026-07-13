use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const NODE_KIND_WORKSPACE: &str = "Workspace";
pub const NODE_KIND_REPOSITORY: &str = "Repository";
pub const NODE_KIND_FILE: &str = "File";
pub const NODE_KIND_MODULE: &str = "Module";
pub const RELATION_CONTAINS: &str = "CONTAINS";
pub const RELATION_IMPORTS: &str = "IMPORTS";
pub const ORIGIN_DETERMINISTIC: &str = "deterministic";
pub const STATUS_ACTIVE: &str = "active";
pub const STATUS_UNVERIFIED: &str = "unverified";
pub const STATUS_STALE: &str = "stale";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub workspace_id: String,
    pub repository_id: Option<String>,
    pub kind: String,
    pub qualified_name: String,
    pub display_name: String,
    pub fingerprint: Option<String>,
    pub status: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    pub id: String,
    pub workspace_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relation_type: String,
    pub origin_type: String,
    pub origin_id: Option<String>,
    pub confidence: f64,
    pub status: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEvidence {
    pub id: String,
    pub edge_id: Option<String>,
    pub node_id: Option<String>,
    pub repository_id: Option<String>,
    pub file_path: Option<String>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub content_hash: Option<String>,
    pub excerpt: Option<String>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GraphSummary {
    pub total_nodes: u64,
    pub nodes_by_kind: BTreeMap<String, u64>,
    pub total_edges: u64,
    pub edges_by_relation: BTreeMap<String, u64>,
}

impl GraphSummary {
    pub fn has_graph(&self) -> bool {
        self.total_nodes > 0 || self.total_edges > 0
    }
}

pub fn workspace_node_id(workspace_id: &str) -> String {
    format!("node:workspace:{workspace_id}")
}

pub fn repository_node_id(workspace_id: &str, repository_id: &str) -> String {
    format!("node:repository:{workspace_id}:{repository_id}")
}

pub fn file_node_id(workspace_id: &str, repository_id: &str, relative_path: &str) -> String {
    format!(
        "node:file:{workspace_id}:{repository_id}:{}",
        stable_hash(relative_path)
    )
}

pub fn module_node_id(workspace_id: &str, repository_id: &str, relative_path: &str) -> String {
    format!(
        "node:module:{workspace_id}:{repository_id}:{}",
        stable_hash(relative_path)
    )
}

pub fn unresolved_module_node_id(
    workspace_id: &str,
    repository_id: &str,
    specifier: &str,
) -> String {
    format!(
        "node:module-unresolved:{workspace_id}:{repository_id}:{}",
        stable_hash(specifier)
    )
}

pub fn contains_edge_id(workspace_id: &str, source_node_id: &str, target_node_id: &str) -> String {
    stable_fact_id(
        "edge:contains",
        &[workspace_id, source_node_id, target_node_id],
    )
}

pub fn imports_edge_id(workspace_id: &str, source_node_id: &str, target_node_id: &str) -> String {
    stable_fact_id(
        "edge:imports",
        &[workspace_id, source_node_id, target_node_id],
    )
}

pub fn evidence_id(parts: &[&str]) -> String {
    stable_fact_id("evidence", parts)
}

pub fn stable_hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn stable_fact_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    format!("{prefix}:{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_ids_are_stable() {
        assert_eq!(workspace_node_id("w1"), workspace_node_id("w1"));
        assert_eq!(
            repository_node_id("w1", "r1"),
            repository_node_id("w1", "r1")
        );
        assert_eq!(
            file_node_id("w1", "r1", "src/main.ts"),
            file_node_id("w1", "r1", "src/main.ts")
        );
        assert_eq!(
            contains_edge_id("w1", "node-a", "node-b"),
            contains_edge_id("w1", "node-a", "node-b")
        );
        assert_eq!(
            module_node_id("w1", "r1", "src/main.ts"),
            module_node_id("w1", "r1", "src/main.ts")
        );
        assert_eq!(
            unresolved_module_node_id("w1", "r1", "./missing"),
            unresolved_module_node_id("w1", "r1", "./missing")
        );
        assert_eq!(
            imports_edge_id("w1", "source", "target"),
            imports_edge_id("w1", "source", "target")
        );
    }

    #[test]
    fn graph_ids_change_for_different_file_paths() {
        assert_ne!(
            file_node_id("w1", "r1", "src/main.ts"),
            file_node_id("w1", "r1", "src/lib.ts")
        );
    }
}
