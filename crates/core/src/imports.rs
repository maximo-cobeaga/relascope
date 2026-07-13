use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportFact {
    pub source_relative_path: String,
    pub source_language: String,
    pub target_specifier: String,
    pub resolved_relative_path: Option<String>,
    pub line_number: u32,
    pub excerpt: String,
}

pub fn detect_imports(
    relative_path: &str,
    language: &str,
    content: &str,
    known_files: &[String],
) -> Vec<ImportFact> {
    content
        .lines()
        .enumerate()
        .flat_map(|(index, line)| {
            let line_number = (index + 1) as u32;
            match language {
                "python" => {
                    detect_python_line(relative_path, language, line, line_number, known_files)
                }
                "typescript" | "javascript" => {
                    detect_ts_js_line(relative_path, language, line, line_number, known_files)
                }
                _ => Vec::new(),
            }
        })
        .collect()
}

fn detect_python_line(
    relative_path: &str,
    language: &str,
    line: &str,
    line_number: u32,
    known_files: &[String],
) -> Vec<ImportFact> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Vec::new();
    }

    if let Some(rest) = trimmed.strip_prefix("import ") {
        return rest
            .split(',')
            .filter_map(|part| {
                let specifier = part.trim().split_whitespace().next()?.trim();
                if specifier.is_empty() {
                    return None;
                }
                Some(import_fact(
                    relative_path,
                    language,
                    specifier,
                    line_number,
                    trimmed,
                    resolve_python_import(relative_path, specifier, known_files),
                ))
            })
            .collect();
    }

    if let Some(rest) = trimmed.strip_prefix("from ") {
        if let Some((specifier, _)) = rest.split_once(" import ") {
            let specifier = specifier.trim();
            if !specifier.is_empty() {
                return vec![import_fact(
                    relative_path,
                    language,
                    specifier,
                    line_number,
                    trimmed,
                    resolve_python_import(relative_path, specifier, known_files),
                )];
            }
        }
    }

    Vec::new()
}

fn detect_ts_js_line(
    relative_path: &str,
    language: &str,
    line: &str,
    line_number: u32,
    known_files: &[String],
) -> Vec<ImportFact> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return Vec::new();
    }

    let specifier = if trimmed.starts_with("import ") {
        extract_from_clause_specifier(trimmed)
            .or_else(|| extract_side_effect_import_specifier(trimmed))
    } else if trimmed.starts_with("export ") {
        extract_from_clause_specifier(trimmed)
    } else {
        None
    };

    specifier
        .map(|specifier| {
            vec![import_fact(
                relative_path,
                language,
                specifier,
                line_number,
                trimmed,
                resolve_ts_js_import(relative_path, specifier, known_files),
            )]
        })
        .unwrap_or_default()
}

fn import_fact(
    relative_path: &str,
    language: &str,
    target_specifier: &str,
    line_number: u32,
    excerpt: &str,
    resolved_relative_path: Option<String>,
) -> ImportFact {
    ImportFact {
        source_relative_path: relative_path.to_string(),
        source_language: language.to_string(),
        target_specifier: target_specifier.to_string(),
        resolved_relative_path,
        line_number,
        excerpt: excerpt.to_string(),
    }
}

fn extract_from_clause_specifier(line: &str) -> Option<&str> {
    let (_, rest) = line.rsplit_once(" from ")?;
    extract_quoted_specifier(rest)
}

fn extract_side_effect_import_specifier(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("import ")?.trim();
    extract_quoted_specifier(rest)
}

fn extract_quoted_specifier(text: &str) -> Option<&str> {
    let quote_index = text.find(|ch| ch == '\'' || ch == '"')?;
    let quote = text.as_bytes()[quote_index] as char;
    let after_quote = &text[quote_index + 1..];
    let end = after_quote.find(quote)?;
    Some(&after_quote[..end])
}

pub fn resolve_ts_js_import(
    source_relative_path: &str,
    specifier: &str,
    known_files: &[String],
) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }

    let base = parent_dir(source_relative_path);
    let joined = normalize_relative_path(&base, specifier);
    let candidates = [
        joined.clone(),
        format!("{joined}.ts"),
        format!("{joined}.tsx"),
        format!("{joined}.js"),
        format!("{joined}.jsx"),
        format!("{joined}/index.ts"),
        format!("{joined}/index.tsx"),
        format!("{joined}/index.js"),
        format!("{joined}/index.jsx"),
    ];

    candidates
        .into_iter()
        .find(|candidate| known_files.iter().any(|known| known == candidate))
}

pub fn resolve_python_import(
    source_relative_path: &str,
    specifier: &str,
    known_files: &[String],
) -> Option<String> {
    let candidate_base = if specifier.starts_with('.') {
        let dots = specifier.chars().take_while(|ch| *ch == '.').count();
        let remainder = &specifier[dots..];
        let mut base_parts = parent_dir(source_relative_path)
            .split('/')
            .filter(|part| !part.is_empty())
            .map(String::from)
            .collect::<Vec<_>>();

        for _ in 1..dots {
            base_parts.pop();
        }

        if !remainder.is_empty() {
            base_parts.extend(remainder.split('.').map(String::from));
        }
        base_parts.join("/")
    } else {
        specifier.replace('.', "/")
    };

    if candidate_base.is_empty() {
        return None;
    }

    let candidates = [
        format!("{candidate_base}.py"),
        format!("{candidate_base}/__init__.py"),
    ];

    candidates
        .into_iter()
        .find(|candidate| known_files.iter().any(|known| known == candidate))
}

fn parent_dir(relative_path: &str) -> String {
    Path::new(relative_path)
        .parent()
        .map(normalize_path)
        .unwrap_or_default()
}

fn normalize_relative_path(base: &str, specifier: &str) -> String {
    let joined = if base.is_empty() {
        PathBuf::from(specifier)
    } else {
        Path::new(base).join(specifier)
    };
    normalize_path(&joined)
}

fn normalize_path(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            _ => {}
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_python_imports_and_ignores_comments() {
        let known = vec!["app/settings.py".to_string()];
        let imports = detect_imports(
            "app/main.py",
            "python",
            "# import ignored\nimport os, sys\nfrom .settings import CONFIG\n",
            &known,
        );

        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].target_specifier, "os");
        assert_eq!(imports[1].target_specifier, "sys");
        assert_eq!(imports[2].target_specifier, ".settings");
        assert_eq!(
            imports[2].resolved_relative_path.as_deref(),
            Some("app/settings.py")
        );
    }

    #[test]
    fn detects_ts_js_imports_and_ignores_comments() {
        let known = vec!["src/message.ts".to_string(), "src/setup.js".to_string()];
        let imports = detect_imports(
            "src/main.ts",
            "typescript",
            "// import ignored from './ignored'\nimport { message } from './message';\nimport './setup.js';\nexport { message } from './message';\n",
            &known,
        );

        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].target_specifier, "./message");
        assert_eq!(
            imports[0].resolved_relative_path.as_deref(),
            Some("src/message.ts")
        );
        assert_eq!(imports[1].target_specifier, "./setup.js");
        assert_eq!(
            imports[1].resolved_relative_path.as_deref(),
            Some("src/setup.js")
        );
    }
}
