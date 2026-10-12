//! The `formal-ai repository-history` subcommand (issue #1180 R9): import the
//! working repository's history into the repository-history memory store,
//! or query that store, following the `cli_github_logs` pattern.

use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Subcommand};
use formal_ai::history_context::{
    HistoryRules, import_incremental, query_repository_history, store_paths,
};

#[derive(Debug, Subcommand)]
pub enum RepositoryHistoryAction {
    /// Formalize new commits (and, with `--logs-dir`, the issues, pull
    /// requests, reviews and Actions runs a `github-logs` capture holds) into
    /// the store; a re-run with no new history appends nothing.
    Import(RepositoryHistoryOptions),
    /// Run an ANSI-SQL memory query over the store, read-only.
    Query {
        #[command(flatten)]
        options: RepositoryHistoryOptions,
        /// The query, e.g. `SELECT id FROM memory WHERE evidence LIKE '%issue:1014%'`.
        #[arg(long)]
        sql: String,
    },
}

#[derive(Debug, Clone, ClapArgs)]
pub struct RepositoryHistoryOptions {
    /// The repository whose history is formalized.
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// A `formal-ai github-logs collect` output directory to import too.
    #[arg(long)]
    logs_dir: Option<PathBuf>,

    /// Directory the `repository-history/<owner>-<repo>/` store lives under
    /// (default: beside the shared memory file).
    #[arg(long)]
    memory_dir: Option<PathBuf>,
}

impl RepositoryHistoryOptions {
    fn memory_dir(&self) -> PathBuf {
        self.memory_dir.clone().unwrap_or_else(|| {
            formal_ai::shared_memory_path()
                .parent()
                .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
        })
    }
}

pub fn run_repository_history(action: RepositoryHistoryAction) -> Result<(), Box<dyn Error>> {
    match action {
        RepositoryHistoryAction::Import(options) => {
            let rules = HistoryRules::load(Some(&options.repo));
            let memory_dir = options.memory_dir();
            let appended = import_incremental(
                &options.repo,
                options.logs_dir.as_deref(),
                &memory_dir,
                &rules,
            )?;
            let (store, cursor) = store_paths(&memory_dir, &options.repo);
            println!("appended={appended}");
            println!("store={}", store.display());
            println!("cursor={}", cursor.display());
        }
        RepositoryHistoryAction::Query { options, sql } => {
            let (store, _) = store_paths(&options.memory_dir(), &options.repo);
            for row in query_repository_history(&store, &sql)? {
                println!("{row}");
            }
        }
    }
    Ok(())
}
