use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use relascope_core::config::WorkspaceConfig;
use relascope_core::inventory::compute_file_changes;
use relascope_core::repository::{slugify_visible_id, validate_visible_id};
use relascope_core::scan::{
    scan_repositories_with_control, ScanCancellationToken, ScanOptions, ScanProgressEvent,
    ScanProgressEventKind, ScanProgressReporter,
};
use relascope_core::workspace::{
    database_path, ensure_state_dir, find_workspace_root, load_workspace_config,
    write_workspace_config, CONFIG_FILE,
};
use relascope_storage_sqlite::SqliteStore;
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(name = "relascope")]
#[command(version)]
#[command(about = "Local-first cross-repository architecture inventory")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a Relascope workspace.
    Init(InitArgs),
    /// Manage repositories.
    Repo(RepoCommand),
    /// Inspect the persisted knowledge graph.
    Graph(GraphCommand),
    /// Inventory registered repositories.
    Scan(ScanArgs),
    /// Show persisted workspace status without scanning.
    Status,
    /// Diagnose workspace health without scanning.
    Doctor,
}

#[derive(Debug, Args)]
struct InitArgs {
    /// Target path. Defaults to the current directory.
    path: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RepoCommand {
    #[command(subcommand)]
    command: RepoSubcommand,
}

#[derive(Debug, Subcommand)]
enum RepoSubcommand {
    /// Add an existing local repository directory.
    Add(RepoAddArgs),
    /// List registered repositories.
    List,
    /// Remove a repository registration without deleting files.
    Remove(RepoRemoveArgs),
}

#[derive(Debug, Args)]
struct GraphCommand {
    #[command(subcommand)]
    command: GraphSubcommand,
}

#[derive(Debug, Subcommand)]
enum GraphSubcommand {
    /// Show persisted graph node and edge counts.
    Summary(FormatArgs),
    /// Show persisted module import relationships.
    Imports(FormatArgs),
}

#[derive(Debug, Args)]
struct FormatArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct ScanArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,
    /// Suppress scan progress output.
    #[arg(long)]
    silent: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Args)]
struct RepoAddArgs {
    /// Existing local repository path.
    path: PathBuf,
    /// Optional visible repository ID.
    #[arg(long = "id")]
    id: Option<String>,
}

#[derive(Debug, Args)]
struct RepoRemoveArgs {
    /// Visible repository ID to remove.
    id: String,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => init(args).await,
        Command::Repo(command) => match command.command {
            RepoSubcommand::Add(args) => repo_add(args).await,
            RepoSubcommand::List => repo_list().await,
            RepoSubcommand::Remove(args) => repo_remove(args).await,
        },
        Command::Graph(command) => match command.command {
            GraphSubcommand::Summary(args) => graph_summary(args).await,
            GraphSubcommand::Imports(args) => graph_imports(args).await,
        },
        Command::Scan(args) => scan(args).await,
        Command::Status => status().await,
        Command::Doctor => doctor().await,
    }
}

async fn init(args: InitArgs) -> Result<()> {
    let root = match args.path {
        Some(path) => path,
        None => std::env::current_dir().context("failed to read current directory")?,
    };

    std::fs::create_dir_all(&root)
        .with_context(|| format!("failed to create workspace path {}", root.display()))?;
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize workspace path {}", root.display()))?;

    if root.join(CONFIG_FILE).exists() {
        return Err(anyhow!(
            "workspace already exists at {} and will not be overwritten",
            root.display()
        ));
    }

    ensure_state_dir(&root).context("failed to create .relascope state directory")?;

    let workspace_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("relascope-workspace")
        .to_string();
    let config = WorkspaceConfig::new(Uuid::new_v4().to_string(), workspace_name);
    write_workspace_config(&root, &config).context("failed to write relascope.yaml")?;

    let store = SqliteStore::open(&database_path(&root))
        .await
        .context("failed to open local SQLite store")?;
    store
        .upsert_workspace(&config, &root)
        .await
        .context("failed to persist workspace")?;

    println!("Initialized Relascope workspace");
    println!("  root: {}", root.display());
    println!("  workspace_id: {}", config.workspace.id);
    println!("  config: {}", root.join(CONFIG_FILE).display());
    println!("  database: {}", database_path(&root).display());
    Ok(())
}

async fn repo_add(args: RepoAddArgs) -> Result<()> {
    reject_remote_source(&args.path)?;
    let context = open_workspace().await?;

    if !args.path.is_dir() {
        return Err(anyhow!(
            "repository path does not exist or is not a directory: {}",
            args.path.display()
        ));
    }

    let canonical_path = args
        .path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", args.path.display()))?;
    let folder_name = canonical_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo");
    let visible_id = args.id.unwrap_or_else(|| slugify_visible_id(folder_name));

    if !validate_visible_id(&visible_id) {
        return Err(anyhow!(
            "invalid repository visible id `{visible_id}`; use letters, numbers, '-' or '_' and start with a letter or number"
        ));
    }

    if context
        .store
        .visible_id_exists(&context.config.workspace.id, &visible_id)
        .await
        .context("failed to check repository visible id uniqueness")?
    {
        return Err(anyhow!(
            "repository visible id `{visible_id}` already exists in this workspace"
        ));
    }

    let repository = context
        .store
        .add_repository(
            &context.config.workspace.id,
            &visible_id,
            folder_name,
            &canonical_path,
        )
        .await
        .context("failed to persist repository")?;

    println!("Added repository");
    println!("  id: {}", repository.visible_id);
    println!("  internal_id: {}", repository.id);
    println!("  path: {}", repository.path.display());
    Ok(())
}

async fn repo_list() -> Result<()> {
    let context = open_workspace().await?;
    let repositories = context
        .store
        .list_repositories(&context.config.workspace.id)
        .await
        .context("failed to load repositories")?;

    println!("Repositories");
    if repositories.is_empty() {
        println!("  no repositories registered");
        return Ok(());
    }

    for repository in repositories {
        println!("  - {}", repository.visible_id);
        println!("      internal_id: {}", repository.id);
        println!("      path: {}", repository.path.display());
        println!("      availability: {}", repository.availability);
    }

    Ok(())
}

async fn repo_remove(args: RepoRemoveArgs) -> Result<()> {
    let context = open_workspace().await?;
    let removed = context
        .store
        .remove_repository_by_visible_id(&context.config.workspace.id, &args.id)
        .await
        .context("failed to remove repository registration")?;

    if !removed {
        return Err(anyhow!(
            "repository visible id `{}` is not registered in this workspace",
            args.id
        ));
    }

    println!("Removed repository registration");
    println!("  id: {}", args.id);
    println!("  files_on_disk: unchanged");
    Ok(())
}

async fn scan(args: ScanArgs) -> Result<()> {
    let context = open_workspace().await?;
    let repositories = context
        .store
        .list_repositories(&context.config.workspace.id)
        .await
        .context("failed to load repositories")?;

    if repositories.is_empty() {
        match args.format {
            OutputFormat::Human => {
                println!("No repositories registered. Use `relascope repo add <path>` first.");
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": "no_repositories",
                        "files": 0,
                        "added": 0,
                        "modified": 0,
                        "removed": 0,
                        "unchanged": 0,
                        "unavailable_repositories": Vec::<String>::new(),
                    }))?
                );
            }
        }
        return Ok(());
    }

    let previous_files = context
        .store
        .previous_completed_scan_files(&context.config.workspace.id)
        .await
        .context("failed to load previous completed scan files")?;
    let scan_id = context
        .store
        .begin_scan(&context.config.workspace.id)
        .await
        .context("failed to create scan record")?;

    let progress_enabled = args.format == OutputFormat::Human && !args.silent;
    if progress_enabled {
        println!("Scanning repositories...");
    }

    let cancellation = ScanCancellationToken::default();
    let scan_options = ScanOptions {
        workspace_id: context.config.workspace.id.clone(),
        scan_id: scan_id.clone(),
        exclusions: context.config.analysis.exclude.clone(),
    };
    let cancellation_listener = tokio::spawn({
        let cancellation = cancellation.clone();
        async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                cancellation.cancel();
            }
        }
    });
    let scan_repositories = repositories.clone();
    let scan_cancellation = cancellation.clone();
    let scan_task = tokio::task::spawn_blocking(move || {
        let mut reporter = CliScanProgressReporter::new(progress_enabled);
        scan_repositories_with_control(
            &scan_repositories,
            &scan_options,
            Some(scan_cancellation),
            &mut reporter,
        )
    });

    let mut outcome = scan_task
        .await
        .context("scan task failed")?
        .context("failed to scan repositories")?;

    if outcome.cancelled || cancellation.is_cancelled() {
        context
            .store
            .finish_scan(&scan_id, "cancelled", &outcome.summary)
            .await
            .context("failed to mark scan as cancelled")?;
        eprintln!("Scan cancelled");
        return Ok(());
    }

    let available_repository_ids = outcome
        .availability
        .iter()
        .filter_map(|(repository_id, availability)| {
            (availability == "available").then_some(repository_id.as_str())
        })
        .collect::<HashSet<_>>();
    let comparable_previous_files = previous_files
        .into_iter()
        .filter(|file| available_repository_ids.contains(file.repository_id.as_str()))
        .collect::<Vec<_>>();
    let changes = compute_file_changes(&comparable_previous_files, &outcome.files);
    outcome.summary.apply_changes(&changes);

    for (repository_id, availability) in &outcome.availability {
        context
            .store
            .update_repository_availability(repository_id, availability)
            .await
            .context("failed to update repository availability")?;
    }

    if cancellation.is_cancelled() {
        context
            .store
            .finish_scan(&scan_id, "cancelled", &outcome.summary)
            .await
            .context("failed to mark scan as cancelled")?;
        eprintln!("Scan cancelled");
        return Ok(());
    }

    context
        .store
        .persist_scan_files(&outcome.files)
        .await
        .context("failed to persist file inventory")?;
    if cancellation.is_cancelled() {
        context
            .store
            .finish_scan(&scan_id, "cancelled", &outcome.summary)
            .await
            .context("failed to mark scan as cancelled")?;
        eprintln!("Scan cancelled");
        return Ok(());
    }

    context
        .store
        .mark_stale_for_file_changes(&context.config.workspace.id, &changes)
        .await
        .context("failed to mark stale graph facts")?;
    if cancellation.is_cancelled() {
        context
            .store
            .finish_scan(&scan_id, "cancelled", &outcome.summary)
            .await
            .context("failed to mark scan as cancelled")?;
        eprintln!("Scan cancelled");
        return Ok(());
    }

    context
        .store
        .materialize_minimal_graph(&context.config, &repositories, &outcome.files, &scan_id)
        .await
        .context("failed to materialize minimal graph")?;
    if cancellation.is_cancelled() {
        context
            .store
            .finish_scan(&scan_id, "cancelled", &outcome.summary)
            .await
            .context("failed to mark scan as cancelled")?;
        eprintln!("Scan cancelled");
        return Ok(());
    }

    context
        .store
        .materialize_shallow_imports(&context.config, &repositories, &outcome.files, &scan_id)
        .await
        .context("failed to materialize shallow imports")?;
    context
        .store
        .finish_scan(&scan_id, "completed", &outcome.summary)
        .await
        .context("failed to finish scan")?;
    cancellation_listener.abort();

    match args.format {
        OutputFormat::Human => {
            println!("Scan completed");
            println!("  scan_id: {scan_id}");
            println!("  files: {}", outcome.summary.total_files);
            println!("  added: {}", outcome.summary.added);
            println!("  modified: {}", outcome.summary.modified);
            println!("  removed: {}", outcome.summary.removed);
            if !outcome.summary.unavailable_repositories.is_empty() {
                println!(
                    "  unavailable_repositories: {}",
                    outcome.summary.unavailable_repositories.join(", ")
                );
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "scan_id": scan_id,
                    "status": "completed",
                    "files": outcome.summary.total_files,
                    "added": outcome.summary.added,
                    "modified": outcome.summary.modified,
                    "removed": outcome.summary.removed,
                    "unchanged": outcome.summary.unchanged,
                    "unavailable_repositories": outcome.summary.unavailable_repositories,
                }))?
            );
        }
    }
    Ok(())
}

struct CliScanProgressReporter {
    enabled: bool,
}

impl CliScanProgressReporter {
    fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl ScanProgressReporter for CliScanProgressReporter {
    fn report(&mut self, event: ScanProgressEvent) {
        if !self.enabled {
            return;
        }

        match event.kind {
            ScanProgressEventKind::RepositoryStarted => {
                println!(
                    "  {}: starting, {} files indexed, {} skipped, elapsed {}",
                    event.repository_visible_id,
                    event.indexed_files,
                    event.skipped_entries,
                    format_duration(event.elapsed)
                );
            }
            ScanProgressEventKind::Heartbeat | ScanProgressEventKind::RepositoryFinished => {
                println!(
                    "  {}: {} files indexed, {} skipped, elapsed {}",
                    event.repository_visible_id,
                    event.indexed_files,
                    event.skipped_entries,
                    format_duration(event.elapsed)
                );
            }
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

async fn graph_summary(args: FormatArgs) -> Result<()> {
    let context = open_workspace().await?;
    let summary = context
        .store
        .graph_summary(&context.config.workspace.id)
        .await
        .context("failed to read graph summary")?;

    match args.format {
        OutputFormat::Human => {
            println!("Graph summary");
            if !summary.has_graph() {
                println!("  no graph has been materialized yet");
                return Ok(());
            }

            println!("  nodes: {}", summary.total_nodes);
            if !summary.nodes_by_kind.is_empty() {
                println!("  by_kind:");
                for (kind, count) in summary.nodes_by_kind {
                    println!("    {kind}: {count}");
                }
            }
            println!("  edges: {}", summary.total_edges);
            if !summary.edges_by_relation.is_empty() {
                println!("  by_relation:");
                for (relation, count) in summary.edges_by_relation {
                    println!("    {relation}: {count}");
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
    }

    Ok(())
}

async fn graph_imports(args: FormatArgs) -> Result<()> {
    let context = open_workspace().await?;
    let imports = context
        .store
        .list_import_edges(&context.config.workspace.id)
        .await
        .context("failed to read graph imports")?;

    match args.format {
        OutputFormat::Human => {
            println!("Graph imports");
            if imports.is_empty() {
                println!("  no imports have been materialized yet");
                return Ok(());
            }

            for import in imports {
                println!(
                    "  {} -> {} [{}]",
                    import.source, import.target, import.status
                );
                if let Some(file_path) = import.file_path {
                    let location = import
                        .line
                        .map(|line| format!("{file_path}:{line}"))
                        .unwrap_or(file_path);
                    if let Some(excerpt) = import.excerpt {
                        println!("    evidence: {location} {excerpt}");
                    } else {
                        println!("    evidence: {location}");
                    }
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&imports)?);
        }
    }

    Ok(())
}

async fn doctor() -> Result<()> {
    let context = open_workspace().await?;
    let report = context
        .store
        .doctor_report(&context.config)
        .await
        .context("failed to build doctor report")?;

    println!("Relascope doctor");
    println!(
        "  config: {}",
        if report.config_ok { "ok" } else { "error" }
    );
    println!(
        "  database: {}",
        if report.database_ok { "ok" } else { "error" }
    );
    println!(
        "  workspace: {} ({})",
        report.workspace_name, report.workspace_id
    );
    println!(
        "  repositories: {} total, {} available, {} unavailable",
        report.repositories_total, report.repositories_available, report.repositories_unavailable
    );
    match &report.last_scan_status {
        Some(status) => println!("  last_scan: {status}"),
        None => println!("  last_scan: none"),
    }
    println!("  graph:");
    println!("    stale_nodes: {}", report.stale_nodes);
    println!("    stale_edges: {}", report.stale_edges);
    println!("    unverified_imports: {}", report.unverified_imports);
    println!("  exclusions: {} configured", report.exclusions.len());

    Ok(())
}

async fn status() -> Result<()> {
    let context = open_workspace().await?;
    let status = context
        .store
        .status(&context.config)
        .await
        .context("failed to read workspace status")?;

    println!("Relascope workspace");
    println!("  name: {}", status.workspace_name);
    println!("  workspace_id: {}", status.workspace_id);
    println!("  root: {}", status.root_path.display());
    println!("  repositories: {}", status.repositories.len());

    for repository in status.repositories {
        println!("  - {}", repository.visible_id);
        println!("      internal_id: {}", repository.id);
        println!("      path: {}", repository.path.display());
        println!("      availability: {}", repository.availability);
    }

    match status.last_scan {
        Some(scan) => {
            println!("  last_scan:");
            println!("      id: {}", scan.id);
            println!("      status: {}", scan.status);
            println!("      started_at: {}", scan.started_at);
            if let Some(finished_at) = scan.finished_at {
                println!("      finished_at: {finished_at}");
            }
            println!("      files: {}", scan.summary.total_files);
            if !scan.summary.by_kind.is_empty() {
                println!("      by_kind:");
                for (kind, count) in scan.summary.by_kind {
                    println!("        {kind}: {count}");
                }
            }
            if !scan.summary.by_language.is_empty() {
                println!("      by_language:");
                for (language, count) in scan.summary.by_language {
                    println!("        {language}: {count}");
                }
            }
            if !scan.summary.unavailable_repositories.is_empty() {
                println!("      unavailable_repositories:");
                for repository in scan.summary.unavailable_repositories {
                    println!("        {repository}");
                }
            }
        }
        None => println!("  last_scan: none"),
    }

    Ok(())
}

struct WorkspaceContext {
    config: WorkspaceConfig,
    store: SqliteStore,
}

async fn open_workspace() -> Result<WorkspaceContext> {
    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    let root = find_workspace_root(&current_dir).map_err(|error| anyhow!(error))?;
    let config = load_workspace_config(&root).map_err(|error| anyhow!(error))?;
    let store = SqliteStore::open(&database_path(&root))
        .await
        .context("failed to open local SQLite store")?;
    store
        .upsert_workspace(&config, &root)
        .await
        .context("failed to persist workspace metadata")?;

    Ok(WorkspaceContext { config, store })
}

fn reject_remote_source(path: &Path) -> Result<()> {
    let raw = path.to_string_lossy();
    if raw.contains("://") || raw.starts_with("git@") {
        return Err(anyhow!(
            "Git URLs and cloning are out of scope for Relascope 0.0.1: {raw}"
        ));
    }
    Ok(())
}
