use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceConfig {
    pub version: u32,
    pub workspace: WorkspaceSection,
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisConfig {
    pub exclude: Vec<String>,
}

impl WorkspaceConfig {
    pub fn new(id: String, name: String) -> Self {
        Self {
            version: 1,
            workspace: WorkspaceSection { id, name },
            analysis: AnalysisConfig {
                exclude: default_exclusions(),
            },
        }
    }
}

pub fn default_exclusions() -> Vec<String> {
    vec![
        "**/.git/**",
        "**/node_modules/**",
        "**/.venv/**",
        "**/dist/**",
        "**/build/**",
        "**/target/**",
        "**/.relascope/**",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
