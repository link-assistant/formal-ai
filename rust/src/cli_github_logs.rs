//! The `formal-ai github-logs` subcommand: plan or collect `gh` evidence
//! captures for issues, pull requests, and Actions runs.

use std::error::Error;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};
use formal_ai::{GithubLogCollectorConfig, collect_github_logs, render_github_log_plan};

#[derive(Debug, Subcommand)]
pub enum GithubLogsAction {
    /// Print the exact `gh` commands and output files without executing them.
    Plan(GithubLogsOptions),
    /// Execute the `gh` command plan and write captures plus `manifest.json`.
    Collect(GithubLogsOptions),
}

#[derive(Debug, Clone, ClapArgs)]
pub struct GithubLogsOptions {
    /// Repository in OWNER/REPO format.
    #[arg(long)]
    repo: String,

    /// Directory where captured JSON, diff, and log files are written.
    #[arg(long, default_value = "docs/case-studies/github-logs/raw-data")]
    output_dir: PathBuf,

    /// Issue number to capture. Repeat for multiple issues.
    #[arg(long = "issue")]
    issues: Vec<u64>,

    /// Pull request number to capture. Repeat for multiple pull requests.
    #[arg(long = "pull")]
    pulls: Vec<u64>,

    /// GitHub Actions run database id to capture. Repeat for multiple runs.
    #[arg(long = "run")]
    runs: Vec<u64>,

    /// Number of recent issues to list for repository context.
    #[arg(long, default_value_t = 10)]
    recent_issues: usize,

    /// Number of recent pull requests to list for repository context.
    #[arg(long, default_value_t = 10)]
    recent_pulls: usize,

    /// Number of recent Actions runs to list for repository context.
    #[arg(long, default_value_t = 5)]
    recent_runs: usize,

    /// Optional branch filter for recent Actions runs.
    #[arg(long)]
    branch: Option<String>,
}

impl GithubLogsOptions {
    fn into_config(self) -> GithubLogCollectorConfig {
        GithubLogCollectorConfig {
            repo: self.repo,
            output_dir: self.output_dir,
            issues: self.issues,
            pulls: self.pulls,
            runs: self.runs,
            recent_issues: self.recent_issues,
            recent_pulls: self.recent_pulls,
            recent_runs: self.recent_runs,
            branch: self.branch,
        }
    }
}

pub fn run_github_logs(action: GithubLogsAction) -> Result<(), Box<dyn Error>> {
    match action {
        GithubLogsAction::Plan(options) => {
            let config = options.into_config();
            print!("{}", render_github_log_plan(&config)?);
        }
        GithubLogsAction::Collect(options) => {
            let config = options.into_config();
            let summary = collect_github_logs(&config)?;
            eprintln!(
                "Captured {} file(s) into {}; manifest: {}",
                summary.captured.len(),
                summary.output_dir.display(),
                summary.manifest_path.display()
            );
            for capture in summary.captured {
                eprintln!("  {}", capture.file);
            }
        }
    }
    Ok(())
}
