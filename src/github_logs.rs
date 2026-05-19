use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const REPO_VIEW_FIELDS: &str =
    "name,description,owner,url,defaultBranchRef,createdAt,pushedAt,isPrivate,licenseInfo";
const ISSUE_VIEW_FIELDS: &str =
    "number,title,body,author,state,labels,comments,createdAt,updatedAt,url";
const ISSUE_LIST_FIELDS: &str = "number,title,state,labels,createdAt,updatedAt,url,author";
const PR_VIEW_FIELDS: &str = "number,title,body,author,state,isDraft,headRefName,baseRefName,commits,comments,reviews,reviewDecision,mergeStateStatus,createdAt,updatedAt,url";
const PR_LIST_FIELDS: &str =
    "number,title,state,createdAt,updatedAt,url,author,headRefName,baseRefName,isDraft";
const RUN_LIST_FIELDS: &str =
    "databaseId,workflowName,status,conclusion,createdAt,updatedAt,headSha,headBranch,event,url";
const RUN_VIEW_FIELDS: &str =
    "databaseId,workflowName,status,conclusion,createdAt,updatedAt,headSha,headBranch,event,url,jobs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubLogCollectorConfig {
    pub repo: String,
    pub output_dir: PathBuf,
    pub issues: Vec<u64>,
    pub pulls: Vec<u64>,
    pub runs: Vec<u64>,
    pub recent_issues: usize,
    pub recent_pulls: usize,
    pub recent_runs: usize,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GithubLogCapture {
    pub kind: String,
    pub file: String,
    pub command: Vec<String>,
}

impl GithubLogCapture {
    #[must_use]
    pub fn command_line(&self) -> String {
        self.command
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GithubLogCapturedFile {
    pub kind: String,
    pub file: String,
    pub command: Vec<String>,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubLogCollectionSummary {
    pub repo: String,
    pub output_dir: PathBuf,
    pub captured: Vec<GithubLogCapturedFile>,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct GithubLogManifest<'a> {
    repo: &'a str,
    generated_by: &'a str,
    generated_at_unix_seconds: u64,
    captures: &'a [GithubLogCapturedFile],
}

pub fn github_log_capture_plan(
    config: &GithubLogCollectorConfig,
) -> Result<Vec<GithubLogCapture>, String> {
    let (owner, name) = split_repo(&config.repo)?;
    let api_repo = format!("repos/{owner}/{name}");
    let mut captures = Vec::new();

    push_capture(
        &mut captures,
        "repo",
        "repo.json",
        vec![
            "gh",
            "repo",
            "view",
            config.repo.as_str(),
            "--json",
            REPO_VIEW_FIELDS,
        ],
    );

    if config.recent_issues > 0 {
        push_capture(
            &mut captures,
            "recent_issues",
            "issues-recent.json",
            vec![
                String::from("gh"),
                String::from("issue"),
                String::from("list"),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--state"),
                String::from("all"),
                String::from("--limit"),
                config.recent_issues.to_string(),
                String::from("--json"),
                String::from(ISSUE_LIST_FIELDS),
            ],
        );
    }

    if config.recent_pulls > 0 {
        push_capture(
            &mut captures,
            "recent_pulls",
            "pulls-recent.json",
            vec![
                String::from("gh"),
                String::from("pr"),
                String::from("list"),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--state"),
                String::from("all"),
                String::from("--limit"),
                config.recent_pulls.to_string(),
                String::from("--json"),
                String::from(PR_LIST_FIELDS),
            ],
        );
    }

    if config.recent_runs > 0 {
        let mut command = vec![
            String::from("gh"),
            String::from("run"),
            String::from("list"),
            String::from("--repo"),
            config.repo.clone(),
            String::from("--limit"),
            config.recent_runs.to_string(),
        ];
        if let Some(branch) = config.branch.as_deref() {
            command.push(String::from("--branch"));
            command.push(branch.to_owned());
        }
        command.push(String::from("--json"));
        command.push(String::from(RUN_LIST_FIELDS));
        push_capture(
            &mut captures,
            "recent_runs",
            "actions-runs-recent.json",
            command,
        );
    }

    for issue in &config.issues {
        push_capture(
            &mut captures,
            "issue",
            &format!("issue-{issue}.json"),
            vec![
                String::from("gh"),
                String::from("issue"),
                String::from("view"),
                issue.to_string(),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--json"),
                String::from(ISSUE_VIEW_FIELDS),
            ],
        );
        push_capture(
            &mut captures,
            "issue_comments",
            &format!("issue-{issue}-comments.json"),
            vec![
                String::from("gh"),
                String::from("api"),
                format!("{api_repo}/issues/{issue}/comments"),
                String::from("--paginate"),
            ],
        );
    }

    for pull in &config.pulls {
        push_capture(
            &mut captures,
            "pull",
            &format!("pr-{pull}.json"),
            vec![
                String::from("gh"),
                String::from("pr"),
                String::from("view"),
                pull.to_string(),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--json"),
                String::from(PR_VIEW_FIELDS),
            ],
        );
        push_capture(
            &mut captures,
            "pull_conversation_comments",
            &format!("pr-{pull}-conversation-comments.json"),
            vec![
                String::from("gh"),
                String::from("api"),
                format!("{api_repo}/issues/{pull}/comments"),
                String::from("--paginate"),
            ],
        );
        push_capture(
            &mut captures,
            "pull_review_comments",
            &format!("pr-{pull}-review-comments.json"),
            vec![
                String::from("gh"),
                String::from("api"),
                format!("{api_repo}/pulls/{pull}/comments"),
                String::from("--paginate"),
            ],
        );
        push_capture(
            &mut captures,
            "pull_reviews",
            &format!("pr-{pull}-reviews.json"),
            vec![
                String::from("gh"),
                String::from("api"),
                format!("{api_repo}/pulls/{pull}/reviews"),
                String::from("--paginate"),
            ],
        );
        push_capture(
            &mut captures,
            "pull_diff",
            &format!("pr-{pull}.diff"),
            vec![
                String::from("gh"),
                String::from("pr"),
                String::from("diff"),
                pull.to_string(),
                String::from("--repo"),
                config.repo.clone(),
            ],
        );
    }

    for run in &config.runs {
        push_capture(
            &mut captures,
            "run",
            &format!("run-{run}.json"),
            vec![
                String::from("gh"),
                String::from("run"),
                String::from("view"),
                run.to_string(),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--json"),
                String::from(RUN_VIEW_FIELDS),
            ],
        );
        push_capture(
            &mut captures,
            "run_log",
            &format!("run-{run}.log"),
            vec![
                String::from("gh"),
                String::from("run"),
                String::from("view"),
                run.to_string(),
                String::from("--repo"),
                config.repo.clone(),
                String::from("--log"),
            ],
        );
    }

    Ok(captures)
}

pub fn render_github_log_plan(config: &GithubLogCollectorConfig) -> Result<String, String> {
    let plan = github_log_capture_plan(config)?;
    let mut output = String::new();
    output.push_str("# GitHub log capture plan\n");
    let _ = writeln!(output, "repo: {}", config.repo);
    let _ = writeln!(output, "output_dir: {}", config.output_dir.display());
    output.push('\n');
    for capture in plan {
        let _ = writeln!(output, "- {}", capture.kind);
        let _ = writeln!(output, "  file: {}", capture.file);
        let _ = writeln!(output, "  command: {}", capture.command_line());
    }
    Ok(output)
}

pub fn collect_github_logs(
    config: &GithubLogCollectorConfig,
) -> Result<GithubLogCollectionSummary, String> {
    collect_github_logs_with_runner(config, run_command)
}

pub fn collect_github_logs_with_runner(
    config: &GithubLogCollectorConfig,
    mut runner: impl FnMut(&[String]) -> Result<Vec<u8>, String>,
) -> Result<GithubLogCollectionSummary, String> {
    let plan = github_log_capture_plan(config)?;
    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            config.output_dir.display()
        )
    })?;

    let mut captured = Vec::new();
    for capture in plan {
        let output = runner(&capture.command)
            .map_err(|error| format!("{} failed: {error}", capture.command_line()))?;
        let output_path = config.output_dir.join(&capture.file);
        fs::write(&output_path, &output)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        captured.push(GithubLogCapturedFile {
            kind: capture.kind,
            file: capture.file,
            command: capture.command,
            bytes: output.len(),
        });
    }

    let manifest = GithubLogManifest {
        repo: &config.repo,
        generated_by: "formal-ai github-logs collect",
        generated_at_unix_seconds: unix_now(),
        captures: &captured,
    };
    let manifest_text = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("failed to format manifest: {error}"))?;
    let manifest_path = config.output_dir.join("manifest.json");
    fs::write(&manifest_path, manifest_text)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;

    Ok(GithubLogCollectionSummary {
        repo: config.repo.clone(),
        output_dir: config.output_dir.clone(),
        captured,
        manifest_path,
    })
}

fn push_capture<S>(captures: &mut Vec<GithubLogCapture>, kind: &str, file: &str, command: Vec<S>)
where
    S: Into<String>,
{
    captures.push(GithubLogCapture {
        kind: kind.to_owned(),
        file: file.to_owned(),
        command: command.into_iter().map(Into::into).collect(),
    });
}

fn run_command(args: &[String]) -> Result<Vec<u8>, String> {
    let Some((program, rest)) = args.split_first() else {
        return Err(String::from("empty command"));
    };
    let output = Command::new(program)
        .args(rest)
        .output()
        .map_err(|error| format!("failed to start {program}: {error}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("exit status {}: {stderr}", output.status))
    }
}

fn split_repo(repo: &str) -> Result<(&str, &str), String> {
    let mut parts = repo.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        return Err(format!(
            "GitHub repository must use OWNER/REPO format, got `{repo}`"
        ));
    }
    validate_repo_part("owner", owner)?;
    validate_repo_part("repo", name)?;
    Ok((owner, name))
}

fn validate_repo_part(label: &str, value: &str) -> Result<(), String> {
    if value.chars().all(is_repo_name_char) {
        return Ok(());
    }
    Err(format!(
        "GitHub repository {label} contains unsupported characters: `{value}`"
    ))
}

const fn is_repo_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:=,@".contains(character))
    {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
