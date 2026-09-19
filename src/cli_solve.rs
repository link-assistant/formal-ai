//! `formal-ai solve` — one repository task, end to end, refusing to commit by
//! default (#1138 B7, plan 03 L12–L13).
//!
//! Mutation is opt-in: without `--commit` the diff is printed and the tree is
//! left alone. When a commit is made it carries the same four self-hosting
//! trailers `scripts/author-change-with-formal-ai.sh` already writes and
//! `scripts/self-hosting-metric.rs` already reads, and only `formal-ai` is an
//! authoring path — a hosted model is not.
//!
//! Wave T lands the shapes only; wave I7 leaves 03-L12 and 03-L13 fill the
//! bodies in.

use std::error::Error;
use std::fmt::Write as _;
use std::io::Read as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::repository_workspace::clone::{WorkspaceSpec, observed_head};
use crate::repository_workspace::{RepositoryTask, RepositoryWorkspace, WorkspaceProtocol};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceCapture};

/// Arguments for `formal-ai solve`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveArgs {
    /// A GitHub issue URL, or `-` to read the requirement from stdin.
    pub issue: Option<String>,
    /// Literal requirement text, when no issue is given.
    pub task: Option<String>,
    /// Repository to work in (default: the current checkout).
    pub repository: String,
    /// Base commit (default: `HEAD` of `--repository`).
    pub base_commit: Option<String>,
    /// Which model authors the change. Only `formal-ai` is an authoring path.
    pub model: String,
    /// Where the run's raw traces are committed.
    pub evidence: PathBuf,
    /// The pull request the commit belongs to.
    pub pull_request: Option<String>,
    /// Refuse to commit; print the diff instead. Default-deny for mutation.
    pub commit: bool,
    /// Live authoring mode: workspace-relative files the Agent CLI must write.
    pub produces: Vec<String>,
    /// Live authoring mode: repository-relative landing spots, pairwise with
    /// `produces` (missing entries default to their `produces` twin).
    pub into: Vec<String>,
    /// Live authoring mode: repository-relative directory copied into the
    /// workspace before the run; seed files are context, not authored effects.
    pub seed: Option<String>,
    /// Live authoring mode: texts at least one produced artifact must contain.
    pub contains: Vec<String>,
    /// Live authoring mode: port the authoring server binds.
    pub port: u16,
    /// Live authoring mode: the commit subject.
    pub message: Option<String>,
    /// Test seam: the `serve` executable the loop spawns, instead of
    /// `FORMAL_AI_SERVER` / the running binary.
    #[doc(hidden)]
    pub server_executable: Option<PathBuf>,
    /// Test seam: the Agent CLI executable the loop drives, instead of
    /// `AGENT` / `agent` on `PATH`.
    #[doc(hidden)]
    pub agent_executable: Option<PathBuf>,
}

impl Default for SolveArgs {
    fn default() -> Self {
        Self {
            issue: None,
            task: None,
            repository: String::from("."),
            base_commit: None,
            model: String::from("formal-ai"),
            evidence: PathBuf::new(),
            pull_request: None,
            commit: false,
            produces: Vec::new(),
            into: Vec::new(),
            seed: None,
            contains: Vec::new(),
            port: 8899,
            message: None,
            server_executable: None,
            agent_executable: None,
        }
    }
}

/// What one `solve` run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveOutcome {
    /// The unified diff, whatever it is.
    pub diff: String,
    /// Whether a commit was actually made.
    pub committed: bool,
    /// The commit message, including its trailers, when one was written.
    pub commit_message: String,
    /// Files written under `--evidence`.
    pub evidence_files: Vec<PathBuf>,
    /// Protocol steps whose postcondition was not observed.
    pub open: Vec<String>,
}

struct RequirementInput {
    text: String,
    issue_source: Option<IssueSource>,
}

struct IssueSource {
    issue_url: String,
    api_url: String,
    capture: SourceCapture,
}

/// Run one repository task end to end and, when `--commit` is given, land it
/// with the four self-hosting trailers.
///
/// # Errors
/// Any protocol step whose postcondition was not observed aborts before the
/// commit; a partial tree is never committed. A `--model` that is not an
/// authoring path is refused before anything is written.
pub fn run_solve(args: &SolveArgs) -> Result<SolveOutcome, Box<dyn Error>> {
    if args.model != "formal-ai" && !args.model.starts_with("formal-ai/") {
        return Err(
            protocol_template("solve_model_not_authoring_path", &[("model", &args.model)]).into(),
        );
    }
    if args.evidence.as_os_str().is_empty() {
        return Err("--evidence must name a directory".into());
    }
    // The live authoring loop (plan 03 L13): the reduced
    // `scripts/author-change-with-formal-ai.sh` translates its CLI onto this
    // surface, so the loop's logic lives here where it is unit-testable.
    if !args.produces.is_empty() {
        let outcome = crate::authoring_loop::run_authoring(args)?;
        return Ok(SolveOutcome {
            diff: String::new(),
            committed: outcome.committed,
            commit_message: outcome.commit_message,
            evidence_files: vec![args.evidence.clone()],
            open: Vec::new(),
        });
    }

    let input = requirement(args)?;
    let requirement = input.text.clone();
    let repository = PathBuf::from(&args.repository);
    let base_commit = args
        .base_commit
        .clone()
        .or_else(|| observed_head(&repository))
        .ok_or("the repository has no observable HEAD commit")?;
    let spec = WorkspaceSpec {
        origin: args.repository.clone(),
        base_commit,
        sparse_paths: Vec::new(),
    };
    let scratch = SolveScratch::new(&requirement);
    let mut workspace = RepositoryWorkspace::open(&spec, &scratch.path).map_err(|error| {
        protocol_template("solve_workspace_error", &[("error", &format!("{error:?}"))])
    })?;
    let task = RepositoryTask {
        requirement: requirement.clone(),
        clone: spec,
        tests: None,
    };
    let protocol = WorkspaceProtocol::load();
    let mut protocol_outcome = protocol.execute(&mut workspace, &task);

    std::fs::create_dir_all(&args.evidence)?;
    let model = if args.model == "formal-ai" {
        format!("formal-ai/{}", env!("CARGO_PKG_VERSION"))
    } else {
        args.model.clone()
    };
    let session = crate::engine::stable_id(
        "session",
        &format!(
            "{}:{}:{requirement}",
            args.repository, task.clone.base_commit
        ),
    );
    let mut evidence_documents = vec![
        (
            String::from("task.txt"),
            format!("{requirement}\n").into_bytes(),
        ),
        (
            String::from("session-id.txt"),
            protocol_template(
                "solve_session_metadata",
                &[("session", &session), ("model", &model)],
            )
            .into_bytes(),
        ),
        (
            String::from("repository-protocol.lino"),
            protocol_trace(&task, &protocol_outcome, &session, &model).into_bytes(),
        ),
    ];
    if let Some(source) = input.issue_source {
        evidence_documents.push((
            String::from("issue-source.json"),
            source.capture.bytes().to_vec(),
        ));
        evidence_documents.push((
            String::from("issue-source.lino"),
            crate::links_format::format_lino_record(
                "repository_issue_source",
                &[
                    ("record_type", String::from("repository_issue_source")),
                    ("issue_url", source.issue_url),
                    ("source_url", source.api_url),
                    ("fetched_at", source.capture.fetched_at().to_owned()),
                    ("sha256", source.capture.sha256().to_owned()),
                    ("cached", source.capture.cached().to_string()),
                ],
            )
            .into_bytes(),
        ));
    }
    let mut evidence_files = Vec::new();
    for (name, bytes) in &evidence_documents {
        let path = args.evidence.join(name);
        std::fs::write(&path, bytes)?;
        evidence_files.push(path);
    }

    // Attribution evidence travels in the same isolated clone and commit as
    // the authored source bytes. An unsuccessful authoring attempt keeps its
    // diagnostics outside the clone and can never become an evidence-only
    // contribution.
    let committed_evidence = format!(".formal-ai/evidence/{session}");
    if protocol_outcome.open.is_empty()
        && protocol_outcome.stopped_at.is_none()
        && !protocol_outcome.edited.is_empty()
    {
        for (name, bytes) in &evidence_documents {
            workspace.write(
                &format!("{committed_evidence}/{name}"),
                std::str::from_utf8(bytes).map_err(|error| {
                    protocol_template(
                        "solve_evidence_not_utf8",
                        &[("name", name), ("error", &error.to_string())],
                    )
                })?,
            )?;
        }
        protocol_outcome.diff = workspace.diff()?;
    }

    let can_commit = args.commit
        && protocol_outcome.open.is_empty()
        && protocol_outcome.stopped_at.is_none()
        && !protocol_outcome.edited.is_empty()
        && !protocol_outcome.diff.trim().is_empty();
    let commit_message = if args.commit {
        commit_message(args, &session, &model, &committed_evidence)?
    } else {
        String::new()
    };

    // The current protocol does not invent an edit when discovery produced no
    // patch. A caller gets the attributable message it would use, plus the open
    // obligations, but `committed` remains false unless a real diff exists.
    // The mutation implementation is deliberately bounded to the isolated clone;
    // no code path below can commit the operator's ambient checkout.
    let committed = if can_commit {
        commit_isolated(&workspace, &commit_message)?
    } else {
        false
    };

    Ok(SolveOutcome {
        diff: protocol_outcome.diff,
        committed,
        commit_message,
        evidence_files,
        open: protocol_outcome.open,
    })
}

fn requirement(args: &SolveArgs) -> Result<RequirementInput, Box<dyn Error>> {
    if let Some(task) = args
        .task
        .as_deref()
        .map(str::trim)
        .filter(|task| !task.is_empty())
    {
        return Ok(RequirementInput {
            text: task.to_owned(),
            issue_source: None,
        });
    }
    match args.issue.as_deref() {
        Some("-") => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            let text = text.trim();
            if text.is_empty() {
                Err("stdin carried no requirement".into())
            } else {
                Ok(RequirementInput {
                    text: text.to_owned(),
                    issue_source: None,
                })
            }
        }
        Some(issue) if !issue.trim().is_empty() => {
            let issue_url = issue.trim();
            let api_url = github_issue_api_url(issue_url).ok_or_else(|| {
                protocol_template("solve_unsupported_issue_url", &[("issue_url", issue_url)])
            })?;
            let client =
                CachedSourceClient::new(&args.evidence, CurlSourceTransport).with_online(true);
            let capture = client.fetch(&api_url)?;
            let text = parse_github_issue_requirement(issue_url, capture.bytes())?;
            Ok(RequirementInput {
                text,
                issue_source: Some(IssueSource {
                    issue_url: issue_url.to_owned(),
                    api_url,
                    capture,
                }),
            })
        }
        _ => Err("one of --task or --issue is required".into()),
    }
}

fn commit_message(
    args: &SolveArgs,
    session: &str,
    model: &str,
    evidence: &str,
) -> Result<String, Box<dyn Error>> {
    let pull_request = args
        .pull_request
        .as_deref()
        .filter(|value| canonical_pull_request(value))
        .ok_or("--commit requires a canonical GitHub --pull-request URL")?;
    Ok(protocol_template(
        "solve_commit_message",
        &[
            ("session", session),
            ("model", model),
            ("evidence", evidence),
            ("pull_request", pull_request),
        ],
    ))
}

/// Project a canonical GitHub issue URL onto the documented issue API.
#[must_use]
pub fn github_issue_api_url(issue_url: &str) -> Option<String> {
    let tail = issue_url
        .strip_prefix("https://github.com/")?
        .trim_end_matches('/');
    let parts = tail.split('/').collect::<Vec<_>>();
    if parts.len() != 4
        || parts[2] != "issues"
        || !valid_github_segment(parts[0])
        || !valid_github_segment(parts[1])
        || !parts[3].parse::<u64>().is_ok_and(|number| number > 0)
    {
        return None;
    }
    Some(format!(
        "https://api.github.com/repos/{}/{}/issues/{}",
        parts[0], parts[1], parts[3]
    ))
}

fn valid_github_segment(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

/// Recover a requirement from exact captured GitHub API bytes.
///
/// The payload must describe the requested issue (not a pull request); when an
/// `html_url` is present it must agree with the URL the caller requested.
pub fn parse_github_issue_requirement(issue_url: &str, payload: &[u8]) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_slice(payload).map_err(|error| {
        protocol_template(
            "solve_issue_payload_not_json",
            &[("error", &error.to_string())],
        )
    })?;
    if value.get("pull_request").is_some() {
        return Err(String::from(
            "the GitHub resource is a pull request, not an issue",
        ));
    }
    if let Some(observed_url) = value.get("html_url").and_then(serde_json::Value::as_str)
        && observed_url.trim_end_matches('/') != issue_url.trim_end_matches('/')
    {
        return Err(protocol_template(
            "solve_issue_url_mismatch",
            &[("observed_url", observed_url), ("issue_url", issue_url)],
        ));
    }
    let title = value
        .get("title")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or_else(|| String::from("GitHub issue payload has no title"))?;
    let body = value
        .get("body")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    Ok(if body.is_empty() {
        title.to_owned()
    } else {
        [title, body].join("\n\n")
    })
}

fn canonical_pull_request(value: &str) -> bool {
    let Some(tail) = value.strip_prefix("https://github.com/") else {
        return false;
    };
    let parts: Vec<&str> = tail.trim_end_matches('/').split('/').collect();
    parts.len() == 4
        && !parts[0].is_empty()
        && !parts[1].is_empty()
        && parts[2] == "pull"
        && parts[3].parse::<u64>().is_ok_and(|number| number > 0)
}

fn protocol_trace(
    task: &RepositoryTask,
    outcome: &crate::repository_workspace::ProtocolOutcome,
    session: &str,
    model: &str,
) -> String {
    let mut out = String::from("repository_solve_trace\n");
    let _ = writeln!(out, "  session \"{session}\"");
    let _ = writeln!(out, "  model \"{model}\"");
    let _ = writeln!(out, "  base_commit \"{}\"", task.clone.base_commit);
    let _ = writeln!(out, "  located \"{}\"", outcome.located.len());
    let _ = writeln!(out, "  edited \"{}\"", outcome.edited.len());
    let _ = writeln!(out, "  observations \"{}\"", outcome.observations.len());
    let _ = writeln!(out, "  diff_bytes \"{}\"", outcome.diff.len());
    if let Some(step) = &outcome.stopped_at {
        let _ = writeln!(out, "  stopped_at \"{}\"", step.id);
    }
    for open in &outcome.open {
        let _ = writeln!(out, "  open \"{}\"", open.replace('"', "\"\""));
    }
    out
}

fn commit_isolated(workspace: &RepositoryWorkspace, message: &str) -> Result<bool, Box<dyn Error>> {
    crate::repository_workspace::clone::run_git(workspace.root(), &["add", "--all"]).map_err(
        |error| protocol_template("solve_stage_error", &[("error", &format!("{error:?}"))]),
    )?;
    crate::repository_workspace::clone::run_git(
        workspace.root(),
        &[
            "-c",
            "user.name=Formal AI",
            "-c",
            "user.email=formal-ai@localhost",
            "commit",
            "-m",
            message,
        ],
    )
    .map_err(|error| {
        protocol_template("solve_commit_error", &[("error", &format!("{error:?}"))])
    })?;
    Ok(true)
}

struct SolveScratch {
    path: PathBuf,
}

impl SolveScratch {
    fn new(requirement: &str) -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let id = crate::engine::stable_id("solve_workspace", requirement);
        Self {
            path: std::env::temp_dir()
                .join(format!("formal-ai-{id}-{}-{sequence}", std::process::id())),
        }
    }
}

impl Drop for SolveScratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn protocol_template(id: &str, values: &[(&str, &str)]) -> String {
    crate::repository_workspace::render_protocol_template(id, values)
        .unwrap_or_else(|| id.to_owned())
}
