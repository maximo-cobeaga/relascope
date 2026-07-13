use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryRecord {
    pub id: String,
    pub workspace_id: String,
    pub visible_id: String,
    pub display_name: String,
    pub path: PathBuf,
    pub path_kind: String,
    pub availability: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn slugify_visible_id(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_separator = false;
        } else if (ch == '-' || ch == '_' || ch.is_whitespace()) && !last_was_separator {
            slug.push('-');
            last_was_separator = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "repo".to_string()
    } else {
        trimmed
    }
}

pub fn validate_visible_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 80 {
        return false;
    }

    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphanumeric() {
        return false;
    }

    id.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_folder_names() {
        assert_eq!(slugify_visible_id("API Python"), "api-python");
        assert_eq!(slugify_visible_id("web_typescript"), "web-typescript");
        assert_eq!(slugify_visible_id("***"), "repo");
    }

    #[test]
    fn validates_visible_ids() {
        assert!(validate_visible_id("api"));
        assert!(validate_visible_id("api-v2"));
        assert!(validate_visible_id("api_v2"));
        assert!(!validate_visible_id(""));
        assert!(!validate_visible_id("-api"));
        assert!(!validate_visible_id("api/v2"));
    }
}
