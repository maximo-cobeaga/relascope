use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileInventoryRecord {
    pub id: String,
    pub workspace_id: String,
    pub repository_id: String,
    pub relative_path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: Option<i64>,
    pub content_hash: Option<String>,
    pub modified_at: Option<String>,
    pub scan_id: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScanSummary {
    pub total_files: u64,
    pub by_repository: BTreeMap<String, u64>,
    pub by_kind: BTreeMap<String, u64>,
    pub by_language: BTreeMap<String, u64>,
    pub unavailable_repositories: Vec<String>,
    #[serde(default)]
    pub added: u64,
    #[serde(default)]
    pub modified: u64,
    #[serde(default)]
    pub removed: u64,
    #[serde(default)]
    pub unchanged: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreviousFileState {
    pub repository_id: String,
    pub relative_path: String,
    pub kind: String,
    pub language: Option<String>,
    pub size_bytes: Option<i64>,
    pub content_hash: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileIdentityKey {
    pub repository_id: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileChangeSummary {
    pub added: u64,
    pub modified: u64,
    pub removed: u64,
    pub unchanged: u64,
    pub removed_files: Vec<PreviousFileState>,
    pub modified_files: Vec<FileInventoryRecord>,
}

impl FileIdentityKey {
    pub fn new(repository_id: impl Into<String>, relative_path: impl Into<String>) -> Self {
        Self {
            repository_id: repository_id.into(),
            relative_path: relative_path.into(),
        }
    }
}

impl ScanSummary {
    pub fn add_file(&mut self, repository_visible_id: &str, kind: &str, language: Option<&str>) {
        self.total_files += 1;
        *self
            .by_repository
            .entry(repository_visible_id.to_string())
            .or_insert(0) += 1;
        *self.by_kind.entry(kind.to_string()).or_insert(0) += 1;
        if let Some(language) = language {
            *self.by_language.entry(language.to_string()).or_insert(0) += 1;
        }
    }

    pub fn apply_changes(&mut self, changes: &FileChangeSummary) {
        self.added = changes.added;
        self.modified = changes.modified;
        self.removed = changes.removed;
        self.unchanged = changes.unchanged;
    }

    pub fn add_unavailable_repository(&mut self, repository_visible_id: &str) {
        if !self
            .unavailable_repositories
            .iter()
            .any(|id| id == repository_visible_id)
        {
            self.unavailable_repositories
                .push(repository_visible_id.to_string());
        }
    }
}

pub fn compute_file_changes(
    previous_files: &[PreviousFileState],
    current_files: &[FileInventoryRecord],
) -> FileChangeSummary {
    let previous_by_key = previous_files
        .iter()
        .map(|file| {
            (
                FileIdentityKey::new(&file.repository_id, &file.relative_path),
                file,
            )
        })
        .collect::<HashMap<_, _>>();
    let current_by_key = current_files
        .iter()
        .map(|file| {
            (
                FileIdentityKey::new(&file.repository_id, &file.relative_path),
                file,
            )
        })
        .collect::<HashMap<_, _>>();

    let mut summary = FileChangeSummary::default();
    let mut seen = HashSet::new();

    for (key, current) in &current_by_key {
        seen.insert(key.clone());
        match previous_by_key.get(key) {
            None => {
                summary.added += 1;
            }
            Some(previous) if file_changed(previous, current) => {
                summary.modified += 1;
                summary.modified_files.push((**current).clone());
            }
            Some(_) => {
                summary.unchanged += 1;
            }
        }
    }

    for (key, previous) in previous_by_key {
        if !seen.contains(&key) {
            summary.removed += 1;
            summary.removed_files.push((*previous).clone());
        }
    }

    summary
}

fn file_changed(previous: &PreviousFileState, current: &FileInventoryRecord) -> bool {
    match (&previous.content_hash, &current.content_hash) {
        (Some(previous_hash), Some(current_hash)) => previous_hash != current_hash,
        (None, None) => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn previous(path: &str, hash: Option<&str>) -> PreviousFileState {
        PreviousFileState {
            repository_id: "repo".to_string(),
            relative_path: path.to_string(),
            kind: "code".to_string(),
            language: Some("typescript".to_string()),
            size_bytes: None,
            content_hash: hash.map(String::from),
            modified_at: None,
        }
    }

    fn current(path: &str, hash: Option<&str>) -> FileInventoryRecord {
        FileInventoryRecord {
            id: format!("file-{path}"),
            workspace_id: "workspace".to_string(),
            repository_id: "repo".to_string(),
            relative_path: path.to_string(),
            kind: "code".to_string(),
            language: Some("typescript".to_string()),
            size_bytes: None,
            content_hash: hash.map(String::from),
            modified_at: None,
            scan_id: "scan".to_string(),
            metadata_json: "{}".to_string(),
        }
    }

    #[test]
    fn first_scan_reports_all_files_added() {
        let changes = compute_file_changes(&[], &[current("src/main.ts", Some("a"))]);
        assert_eq!(changes.added, 1);
        assert_eq!(changes.modified, 0);
        assert_eq!(changes.removed, 0);
        assert_eq!(changes.unchanged, 0);
    }

    #[test]
    fn detects_added_modified_removed_and_unchanged_files() {
        let previous_files = vec![
            previous("unchanged.ts", Some("same")),
            previous("modified.ts", Some("old")),
            previous("removed.ts", Some("old")),
        ];
        let current_files = vec![
            current("unchanged.ts", Some("same")),
            current("modified.ts", Some("new")),
            current("added.ts", Some("new")),
        ];

        let changes = compute_file_changes(&previous_files, &current_files);
        assert_eq!(changes.added, 1);
        assert_eq!(changes.modified, 1);
        assert_eq!(changes.removed, 1);
        assert_eq!(changes.unchanged, 1);
        assert_eq!(changes.modified_files[0].relative_path, "modified.ts");
        assert_eq!(changes.removed_files[0].relative_path, "removed.ts");
    }
}
