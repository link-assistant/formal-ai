#!/usr/bin/env rust-script
//! Mine Hive Mind GitHub evidence into a case-study dataset.
//!
//! Usage:
//!   rust-script scripts/mine-hive-mind-dataset.rs --plan
//!   rust-script scripts/mine-hive-mind-dataset.rs --collect
//!   rust-script scripts/mine-hive-mind-dataset.rs --collect --issue 1814 --pull 1816 --run 26058054431
//!
//! The script wraps the `formal-ai github-logs` operator command. It keeps the
//! dataset-mining boundary outside the seed tool registry while making the
//! issue #115 Hive Mind capture repeatable.

use std::env;
use std::path::PathBuf;
use std::process::{exit, Command};

const DEFAULT_REPO: &str = "link-assistant/hive-mind";
const DEFAULT_OUTPUT_DIR: &str = "docs/case-studies/issue-115/raw-data/hive-mind";
const DEFAULT_ISSUES: &[&str] = &["1814", "1813", "1811"];
const DEFAULT_PULLS: &[&str] = &["1816", "1815", "1812"];
const DEFAULT_RUNS: &[&str] = &["26058054431", "25976224438"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Plan,
    Collect,
}

impl Mode {
    const fn as_arg(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Collect => "collect",
        }
    }
}

#[derive(Debug)]
struct Config {
    mode: Mode,
    repo: String,
    output_dir: String,
    issues: Vec<String>,
    pulls: Vec<String>,
    runs: Vec<String>,
    recent_issues: String,
    recent_pulls: String,
    recent_runs: String,
    branch: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Plan,
            repo: DEFAULT_REPO.to_owned(),
            output_dir: DEFAULT_OUTPUT_DIR.to_owned(),
            issues: DEFAULT_ISSUES.iter().map(ToString::to_string).collect(),
            pulls: DEFAULT_PULLS.iter().map(ToString::to_string).collect(),
            runs: DEFAULT_RUNS.iter().map(ToString::to_string).collect(),
            recent_issues: String::from("10"),
            recent_pulls: String::from("10"),
            recent_runs: String::from("10"),
            branch: None,
        }
    }
}

fn main() {
    let config = match parse_args(env::args().skip(1).collect()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            eprintln!();
            print_usage();
            exit(2);
        }
    };

    let repo_root = match find_repo_root() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            exit(2);
        }
    };

    let formal_ai_args = formal_ai_args(&config);
    if config.mode == Mode::Plan {
        eprintln!("Planning Hive Mind dataset capture through formal-ai github-logs.");
    } else {
        eprintln!("Collecting Hive Mind dataset capture through formal-ai github-logs.");
    }

    let status = Command::new("cargo")
        .current_dir(repo_root)
        .args(&formal_ai_args)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("failed to start cargo: {error}");
            exit(1);
        });

    if !status.success() {
        eprintln!("cargo exited with {status}");
        exit(status.code().unwrap_or(1));
    }
}

fn parse_args(args: Vec<String>) -> Result<Config, String> {
    let mut config = Config::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" => {
                print_usage();
                exit(0);
            }
            "--plan" => config.mode = Mode::Plan,
            "--collect" => config.mode = Mode::Collect,
            "--empty-focus" => {
                config.issues.clear();
                config.pulls.clear();
                config.runs.clear();
            }
            "--repo" => {
                index += 1;
                config.repo = next_value(&args, index, "--repo")?;
            }
            "--output-dir" => {
                index += 1;
                config.output_dir = next_value(&args, index, "--output-dir")?;
            }
            "--issue" => {
                index += 1;
                config.issues.push(next_value(&args, index, "--issue")?);
            }
            "--pull" => {
                index += 1;
                config.pulls.push(next_value(&args, index, "--pull")?);
            }
            "--run" => {
                index += 1;
                config.runs.push(next_value(&args, index, "--run")?);
            }
            "--recent-issues" => {
                index += 1;
                config.recent_issues = next_value(&args, index, "--recent-issues")?;
            }
            "--recent-pulls" => {
                index += 1;
                config.recent_pulls = next_value(&args, index, "--recent-pulls")?;
            }
            "--recent-runs" => {
                index += 1;
                config.recent_runs = next_value(&args, index, "--recent-runs")?;
            }
            "--branch" => {
                index += 1;
                config.branch = Some(next_value(&args, index, "--branch")?);
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
        index += 1;
    }

    Ok(config)
}

fn next_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index)
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn formal_ai_args(config: &Config) -> Vec<String> {
    let mut args = vec![
        String::from("run"),
        String::from("--quiet"),
        String::from("--"),
        String::from("github-logs"),
        String::from(config.mode.as_arg()),
        String::from("--repo"),
        config.repo.clone(),
        String::from("--output-dir"),
        config.output_dir.clone(),
        String::from("--recent-issues"),
        config.recent_issues.clone(),
        String::from("--recent-pulls"),
        config.recent_pulls.clone(),
        String::from("--recent-runs"),
        config.recent_runs.clone(),
    ];

    for issue in &config.issues {
        args.push(String::from("--issue"));
        args.push(issue.clone());
    }
    for pull in &config.pulls {
        args.push(String::from("--pull"));
        args.push(pull.clone());
    }
    for run in &config.runs {
        args.push(String::from("--run"));
        args.push(run.clone());
    }
    if let Some(branch) = &config.branch {
        args.push(String::from("--branch"));
        args.push(branch.clone());
    }

    args
}

fn find_repo_root() -> Result<PathBuf, String> {
    let mut current = env::current_dir().map_err(|error| format!("failed to read cwd: {error}"))?;

    loop {
        if current.join("Cargo.toml").is_file() && current.join("src/main.rs").is_file() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(String::from(
                "could not find formal-ai repository root from current directory",
            ));
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: rust-script scripts/mine-hive-mind-dataset.rs [--plan|--collect] [options]\n\
         \n\
         Defaults mine link-assistant/hive-mind into docs/case-studies/issue-115/raw-data/hive-mind.\n\
         \n\
         Options:\n\
           --repo OWNER/REPO          Repository to mine\n\
           --output-dir PATH          Capture output directory\n\
           --issue N                  Add a focused issue capture\n\
           --pull N                   Add a focused pull request capture\n\
           --run N                    Add a focused Actions run capture\n\
           --empty-focus              Clear the default issue/PR/run focus set\n\
           --recent-issues N          Recent issue list size, default 10\n\
           --recent-pulls N           Recent PR list size, default 10\n\
           --recent-runs N            Recent Actions run list size, default 10\n\
           --branch NAME              Optional branch filter for recent runs\n\
           -h, --help                 Show this help"
    );
}
