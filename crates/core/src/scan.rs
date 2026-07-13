use crate::classify::classify_path;
use crate::error::Result;
use crate::inventory::{FileInventoryRecord, ScanSummary};
use crate::repository::RepositoryRecord;
use chrono::{DateTime, Utc};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Component, Path};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

const HASH_SIZE_LIMIT_BYTES: u64 = 10 * 1024 * 1024;
const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".venv",
    "dist",
    "build",
    "target",
    ".relascope",
];

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub workspace_id: String,
    pub scan_id: String,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScanOutcome {
    pub files: Vec<FileInventoryRecord>,
    pub summary: ScanSummary,
    pub availability: Vec<(String, String)>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanProgressEventKind {
    RepositoryStarted,
    Heartbeat,
    RepositoryFinished,
}

#[derive(Debug, Clone)]
pub struct ScanProgressEvent {
    pub kind: ScanProgressEventKind,
    pub repository_visible_id: String,
    pub indexed_files: u64,
    pub skipped_entries: u64,
    pub elapsed: Duration,
}

pub trait ScanProgressReporter {
    fn report(&mut self, event: ScanProgressEvent);
}

#[derive(Debug, Default)]
pub struct NoopScanProgressReporter;

impl ScanProgressReporter for NoopScanProgressReporter {
    fn report(&mut self, _event: ScanProgressEvent) {}
}

#[derive(Debug, Clone, Default)]
pub struct ScanCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl ScanCancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub fn scan_repositories(
    repositories: &[RepositoryRecord],
    options: &ScanOptions,
) -> Result<ScanOutcome> {
    let mut reporter = NoopScanProgressReporter;
    scan_repositories_with_control(repositories, options, None, &mut reporter)
}

pub fn scan_repositories_with_control(
    repositories: &[RepositoryRecord],
    options: &ScanOptions,
    cancellation_token: Option<ScanCancellationToken>,
    reporter: &mut dyn ScanProgressReporter,
) -> Result<ScanOutcome> {
    let globset = build_globset(&options.exclusions)?;
    let mut files = Vec::new();
    let mut summary = ScanSummary::default();
    let mut availability = Vec::new();
    let started_at = Instant::now();

    for repository in repositories {
        if is_cancelled(cancellation_token.as_ref()) {
            return Ok(ScanOutcome {
                files,
                summary,
                availability,
                cancelled: true,
            });
        }

        if !repository.path.is_dir() {
            availability.push((repository.id.clone(), "unavailable".to_string()));
            summary.add_unavailable_repository(&repository.visible_id);
            continue;
        }

        availability.push((repository.id.clone(), "available".to_string()));

        let skipped_entries = Cell::new(0_u64);
        let mut indexed_files = 0_u64;
        let mut last_report_at = Instant::now();
        let mut last_report_events = 0_u64;

        reporter.report(ScanProgressEvent {
            kind: ScanProgressEventKind::RepositoryStarted,
            repository_visible_id: repository.visible_id.clone(),
            indexed_files,
            skipped_entries: skipped_entries.get(),
            elapsed: started_at.elapsed(),
        });

        let walker = WalkDir::new(&repository.path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let excluded = is_excluded_entry(entry, &repository.path, &globset);
                if excluded {
                    skipped_entries.set(skipped_entries.get().saturating_add(1));
                }
                !excluded
            });

        for entry in walker {
            if is_cancelled(cancellation_token.as_ref()) {
                return Ok(ScanOutcome {
                    files,
                    summary,
                    availability,
                    cancelled: true,
                });
            }

            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let relative = match entry.path().strip_prefix(&repository.path) {
                Ok(path) => path,
                Err(_) => continue,
            };
            if is_excluded_path(relative, &globset) {
                skipped_entries.set(skipped_entries.get().saturating_add(1));
                continue;
            }

            let normalized_relative = normalize_path(relative);
            let metadata = entry.metadata().ok();
            let size_bytes = metadata.as_ref().map(|m| m.len() as i64);
            let modified_at = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(|time| DateTime::<Utc>::from(time).to_rfc3339());
            let content_hash = metadata.as_ref().and_then(|m| {
                if is_cancelled(cancellation_token.as_ref()) {
                    return None;
                }
                hash_file_if_reasonable(entry.path(), m.len())
                    .ok()
                    .flatten()
            });
            if is_cancelled(cancellation_token.as_ref()) {
                return Ok(ScanOutcome {
                    files,
                    summary,
                    availability,
                    cancelled: true,
                });
            }
            let classification = classify_path(relative);

            summary.add_file(
                &repository.visible_id,
                &classification.kind,
                classification.language.as_deref(),
            );

            files.push(FileInventoryRecord {
                id: Uuid::new_v4().to_string(),
                workspace_id: options.workspace_id.clone(),
                repository_id: repository.id.clone(),
                relative_path: normalized_relative,
                kind: classification.kind,
                language: classification.language,
                size_bytes,
                content_hash,
                modified_at,
                scan_id: options.scan_id.clone(),
                metadata_json: json!({}).to_string(),
            });
            indexed_files = indexed_files.saturating_add(1);

            let total_events = indexed_files.saturating_add(skipped_entries.get());
            if total_events.saturating_sub(last_report_events) >= 500
                || last_report_at.elapsed() >= Duration::from_secs(2)
            {
                reporter.report(ScanProgressEvent {
                    kind: ScanProgressEventKind::Heartbeat,
                    repository_visible_id: repository.visible_id.clone(),
                    indexed_files,
                    skipped_entries: skipped_entries.get(),
                    elapsed: started_at.elapsed(),
                });
                last_report_at = Instant::now();
                last_report_events = total_events;
            }
        }

        reporter.report(ScanProgressEvent {
            kind: ScanProgressEventKind::RepositoryFinished,
            repository_visible_id: repository.visible_id.clone(),
            indexed_files,
            skipped_entries: skipped_entries.get(),
            elapsed: started_at.elapsed(),
        });
    }

    Ok(ScanOutcome {
        files,
        summary,
        availability,
        cancelled: false,
    })
}

fn is_cancelled(cancellation_token: Option<&ScanCancellationToken>) -> bool {
    cancellation_token.is_some_and(ScanCancellationToken::is_cancelled)
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn is_excluded_entry(entry: &DirEntry, root: &Path, globset: &GlobSet) -> bool {
    if entry.depth() == 0 {
        return false;
    }

    let path = entry.path();
    if entry.file_type().is_dir() && has_default_excluded_component(path) {
        return true;
    }

    match path.strip_prefix(root) {
        Ok(relative) => is_excluded_path(relative, globset),
        Err(_) => false,
    }
}

fn is_excluded_path(relative: &Path, globset: &GlobSet) -> bool {
    let normalized = normalize_path(relative);
    has_default_excluded_component(relative)
        || globset.is_match(relative)
        || globset.is_match(normalized.as_str())
}

fn has_default_excluded_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(value) => {
            let value = value.to_string_lossy();
            DEFAULT_EXCLUDED_DIRS
                .iter()
                .any(|excluded| value.eq_ignore_ascii_case(excluded))
        }
        _ => false,
    })
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn hash_file_if_reasonable(path: &Path, size: u64) -> std::io::Result<Option<String>> {
    if size > HASH_SIZE_LIMIT_BYTES {
        return Ok(None);
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(Some(hex::encode(hasher.finalize())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_exclusions;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Default)]
    struct RecordingReporter {
        events: Vec<ScanProgressEvent>,
    }

    impl ScanProgressReporter for RecordingReporter {
        fn report(&mut self, event: ScanProgressEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn scans_unknown_files_and_skips_defaults() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("api-python");
        fs::create_dir_all(repo.join(".venv")).unwrap();
        fs::write(repo.join("app.py"), "print('hello')").unwrap();
        fs::write(repo.join("data.unknownext"), "data").unwrap();
        fs::write(repo.join(".venv/ignored.py"), "ignored").unwrap();

        let repository = RepositoryRecord {
            id: "repo-uuid".to_string(),
            workspace_id: "workspace-uuid".to_string(),
            visible_id: "api-python".to_string(),
            display_name: "api-python".to_string(),
            path: repo,
            path_kind: "canonical-local".to_string(),
            availability: "available".to_string(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let outcome = scan_repositories(
            &[repository],
            &ScanOptions {
                workspace_id: "workspace-uuid".to_string(),
                scan_id: "scan-uuid".to_string(),
                exclusions: default_exclusions(),
            },
        )
        .unwrap();

        let paths = outcome
            .files
            .iter()
            .map(|file| file.relative_path.as_str())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"app.py"));
        assert!(paths.contains(&"data.unknownext"));
        assert!(!paths.contains(&".venv/ignored.py"));
        assert_eq!(outcome.summary.total_files, 2);
    }

    #[test]
    fn reports_progress_events_without_changing_scan_results() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("api-python");
        fs::create_dir_all(repo.join("node_modules")).unwrap();
        fs::write(repo.join("app.py"), "print('hello')").unwrap();
        fs::write(repo.join("node_modules/ignored.js"), "ignored").unwrap();

        let repository = RepositoryRecord {
            id: "repo-uuid".to_string(),
            workspace_id: "workspace-uuid".to_string(),
            visible_id: "api".to_string(),
            display_name: "api".to_string(),
            path: repo,
            path_kind: "canonical-local".to_string(),
            availability: "available".to_string(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        let mut reporter = RecordingReporter::default();

        let outcome = scan_repositories_with_control(
            &[repository],
            &ScanOptions {
                workspace_id: "workspace-uuid".to_string(),
                scan_id: "scan-uuid".to_string(),
                exclusions: default_exclusions(),
            },
            None,
            &mut reporter,
        )
        .unwrap();

        assert!(!outcome.cancelled);
        assert_eq!(outcome.summary.total_files, 1);
        assert!(reporter
            .events
            .iter()
            .any(|event| event.kind == ScanProgressEventKind::RepositoryStarted));
        assert!(reporter
            .events
            .iter()
            .any(|event| event.kind == ScanProgressEventKind::RepositoryFinished));
        assert!(reporter
            .events
            .iter()
            .any(|event| event.skipped_entries > 0));
    }

    #[test]
    fn cancellation_token_stops_scan_with_partial_outcome() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("api-python");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("app.py"), "print('hello')").unwrap();

        let repository = RepositoryRecord {
            id: "repo-uuid".to_string(),
            workspace_id: "workspace-uuid".to_string(),
            visible_id: "api".to_string(),
            display_name: "api".to_string(),
            path: repo,
            path_kind: "canonical-local".to_string(),
            availability: "available".to_string(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        let cancellation = ScanCancellationToken::default();
        cancellation.cancel();
        let mut reporter = RecordingReporter::default();

        let outcome = scan_repositories_with_control(
            &[repository],
            &ScanOptions {
                workspace_id: "workspace-uuid".to_string(),
                scan_id: "scan-uuid".to_string(),
                exclusions: default_exclusions(),
            },
            Some(cancellation),
            &mut reporter,
        )
        .unwrap();

        assert!(outcome.cancelled);
        assert_eq!(outcome.summary.total_files, 0);
        assert!(reporter.events.is_empty());
    }
}
