use assert_cmd::Command;
use predicates::prelude::*;
use relascope_core::inventory::ScanSummary;
use relascope_core::workspace::{database_path, load_workspace_config};
use relascope_storage_sqlite::SqliteStore;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn relascope() -> Command {
    Command::cargo_bin("relascope").expect("relascope binary")
}

#[test]
fn init_repo_scan_status_round_trip() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(repo.join(".venv")).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();
    fs::write(repo.join("data.unknownext"), "data").unwrap();
    fs::write(repo.join(".venv/ignored.py"), "ignored").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Relascope workspace"));

    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success()
        .stdout(predicate::str::contains("internal_id:"));

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scan completed"));

    relascope()
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace_id:"))
        .stdout(predicate::str::contains("api"))
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("unknown"))
        .stdout(predicate::str::contains("files: 2"));
}

#[test]
fn repo_list_and_remove_work_without_deleting_files() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no repositories registered"));

    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api"));

    relascope()
        .current_dir(&workspace)
        .args(["repo", "remove", "api"])
        .assert()
        .success()
        .stdout(predicate::str::contains("files_on_disk: unchanged"));

    assert!(repo.exists());

    relascope()
        .current_dir(&workspace)
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no repositories registered"));
}

#[test]
fn rejects_duplicate_visible_id() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo_one = dir.path().join("api-one");
    let repo_two = dir.path().join("api-two");
    fs::create_dir_all(&repo_one).unwrap();
    fs::create_dir_all(&repo_two).unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo_one.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo_two.to_str().unwrap(), "--id", "api"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn graph_summary_reports_no_graph_before_scan_and_counts_after_scan() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();
    fs::write(repo.join("README.md"), "# API").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["graph", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "no graph has been materialized yet",
        ));

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["graph", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nodes: 5"))
        .stdout(predicate::str::contains("Workspace: 1"))
        .stdout(predicate::str::contains("Repository: 1"))
        .stdout(predicate::str::contains("File: 2"))
        .stdout(predicate::str::contains("Module: 1"))
        .stdout(predicate::str::contains("edges: 4"))
        .stdout(predicate::str::contains("CONTAINS: 4"));
}

#[test]
fn repeated_scan_does_not_duplicate_minimal_graph() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["graph", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nodes: 4"))
        .stdout(predicate::str::contains("edges: 3"));
}

#[test]
fn graph_imports_reports_import_evidence_after_scan() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("web-typescript");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("src/main.ts"),
        "import { message } from './message';\nexport { message } from './message';\n",
    )
    .unwrap();
    fs::write(
        repo.join("src/message.ts"),
        "export const message = 'hello';\n",
    )
    .unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "web"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["graph", "imports"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "no imports have been materialized yet",
        ));

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .args(["graph", "imports"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "web/src/main.ts -> web/src/message.ts [active]",
        ))
        .stdout(predicate::str::contains("evidence: src/main.ts:1"))
        .stdout(predicate::str::contains("evidence: src/main.ts:2"));

    relascope()
        .current_dir(&workspace)
        .args(["graph", "summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IMPORTS: 1"));
}

#[test]
fn json_outputs_are_valid_for_scan_summary_and_imports() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("web-typescript");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("src/main.ts"),
        "import { message } from './message';\n",
    )
    .unwrap();
    fs::write(
        repo.join("src/message.ts"),
        "export const message = 'hello';\n",
    )
    .unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "web"])
        .assert()
        .success();

    let scan_output = relascope()
        .current_dir(&workspace)
        .args(["scan", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let scan_json: Value = serde_json::from_slice(&scan_output).unwrap();
    assert_eq!(scan_json["status"], "completed");
    assert_eq!(scan_json["files"], 2);

    let summary_output = relascope()
        .current_dir(&workspace)
        .args(["graph", "summary", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summary_json: Value = serde_json::from_slice(&summary_output).unwrap();
    assert_eq!(summary_json["total_nodes"], 6);

    let imports_output = relascope()
        .current_dir(&workspace)
        .args(["graph", "imports", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let imports_json: Value = serde_json::from_slice(&imports_output).unwrap();
    assert_eq!(imports_json.as_array().unwrap().len(), 1);
    assert_eq!(imports_json[0]["status"], "active");
}

#[test]
fn scan_progress_is_default_and_silent_suppresses_it() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning repositories..."))
        .stdout(predicate::str::contains("api:"))
        .stdout(predicate::str::contains("Persisting inventory..."))
        .stdout(predicate::str::contains("Materializing graph..."))
        .stdout(predicate::str::contains("graph:"))
        .stdout(predicate::str::contains("Materializing shallow imports..."))
        .stdout(predicate::str::contains("Scan completed"));

    relascope()
        .current_dir(&workspace)
        .args(["scan", "--silent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning repositories...").not())
        .stdout(predicate::str::contains("Materializing graph...").not())
        .stdout(predicate::str::contains("Materializing shallow imports...").not())
        .stdout(predicate::str::contains("Scan completed"));
}

#[test]
fn cancelled_last_scan_is_visible_in_status_and_doctor() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();

    let config = load_workspace_config(&workspace).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let store = SqliteStore::open(&database_path(&workspace)).await.unwrap();
        let scan_id = store.begin_scan(&config.workspace.id).await.unwrap();
        store
            .finish_scan(&scan_id, "cancelled", &ScanSummary::default())
            .await
            .unwrap();
    });

    relascope()
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status: cancelled"));

    relascope()
        .current_dir(&workspace)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("last_scan: cancelled"));
}

#[test]
fn doctor_reports_health_without_mutating_availability() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();

    fs::remove_dir_all(&repo).unwrap();

    relascope()
        .current_dir(&workspace)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "repositories: 1 total, 0 available, 1 unavailable",
        ));

    relascope()
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("availability: available"));
}

#[test]
fn status_does_not_rescan_missing_repository() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let repo = dir.path().join("api-python");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hello')").unwrap();

    relascope()
        .args(["init", workspace.to_str().unwrap()])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .args(["repo", "add", repo.to_str().unwrap(), "--id", "api"])
        .assert()
        .success();
    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success();

    fs::remove_dir_all(&repo).unwrap();

    relascope()
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("availability: available"))
        .stdout(predicate::str::contains("files: 1"));

    relascope()
        .current_dir(&workspace)
        .arg("scan")
        .assert()
        .success()
        .stdout(predicate::str::contains("unavailable_repositories: api"));

    relascope()
        .current_dir(&workspace)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("availability: unavailable"));
}
