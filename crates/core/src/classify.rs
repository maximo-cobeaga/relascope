use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileClassification {
    pub kind: String,
    pub language: Option<String>,
}

pub fn classify_path(path: &Path) -> FileClassification {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let lower_name = file_name.to_ascii_lowercase();
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    if path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("generated")
    }) || lower_name.ends_with(".generated")
    {
        return FileClassification {
            kind: "generated".to_string(),
            language: ext.as_deref().and_then(language_for_extension),
        };
    }

    if is_binary_extension(ext.as_deref()) {
        return FileClassification {
            kind: "binary".to_string(),
            language: None,
        };
    }

    if lower_name == "dockerfile" || lower_name.ends_with(".dockerfile") {
        return FileClassification {
            kind: "configuration".to_string(),
            language: Some("dockerfile".to_string()),
        };
    }

    if let Some(language) = ext.as_deref().and_then(language_for_extension) {
        let kind = match language.as_str() {
            "markdown" => "documentation",
            "json" | "yaml" | "toml" => "configuration",
            _ => "code",
        };
        return FileClassification {
            kind: kind.to_string(),
            language: Some(language),
        };
    }

    if is_known_config_name(&lower_name) {
        return FileClassification {
            kind: "configuration".to_string(),
            language: None,
        };
    }

    if lower_name == "readme" || lower_name.starts_with("readme.") {
        return FileClassification {
            kind: "documentation".to_string(),
            language: None,
        };
    }

    FileClassification {
        kind: "unknown".to_string(),
        language: None,
    }
}

fn language_for_extension(ext: &str) -> Option<String> {
    match ext {
        "py" => Some("python".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "js" | "jsx" => Some("javascript".to_string()),
        "json" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "md" | "mdx" => Some("markdown".to_string()),
        "rs" => Some("rust".to_string()),
        "sql" => Some("sql".to_string()),
        _ => None,
    }
}

fn is_binary_extension(ext: Option<&str>) -> bool {
    matches!(
        ext,
        Some(
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "ico"
                | "pdf"
                | "zip"
                | "gz"
                | "tar"
                | "7z"
                | "rar"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
                | "bin"
                | "wasm"
                | "mp4"
                | "mov"
                | "mp3"
                | "wav"
        )
    )
}

fn is_known_config_name(name: &str) -> bool {
    matches!(
        name,
        ".env"
            | ".env.example"
            | "requirements.txt"
            | "package.json"
            | "docker-compose.yml"
            | "docker-compose.yaml"
            | "nginx.conf"
            | "cargo.lock"
            | "package-lock.json"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn classify(path: &str) -> FileClassification {
        classify_path(&PathBuf::from(path))
    }

    #[test]
    fn classifies_common_languages_and_kinds() {
        assert_eq!(classify("app.py").language.as_deref(), Some("python"));
        assert_eq!(
            classify("src/main.ts").language.as_deref(),
            Some("typescript")
        );
        assert_eq!(classify("README.md").kind, "documentation");
        assert_eq!(classify("docker-compose.yml").kind, "configuration");
        assert_eq!(classify("image.png").kind, "binary");
        assert_eq!(classify("data.unknownext").kind, "unknown");
    }
}
