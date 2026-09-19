//! The live authoring loop behind `scripts/author-change-with-formal-ai.sh`
//! (plan 03 L13).
//!
//! The script used to re-implement the whole loop in bash: spawn `serve`,
//! drive the real Agent CLI, harvest the session id, check the artifact
//! contract, and land a four-trailer commit. Every semantic of that loop now
//! lives here, where it is unit-testable against fake `serve` and `agent`
//! executables; the script is a thin translator onto `formal-ai solve`.
//!
//! The pinned semantics that forced the #1069 contract tests to exist survive
//! verbatim:
//!
//! * the four trailers the self-hosting metric reads land in one commit with
//!   the source bytes, never in a follow-up;
//! * one evidence file names the producer (`formal-ai`) and the session id
//!   together, because `commit_has_formal_ai_evidence` looks for exactly that;
//! * a pull-request reference the metric cannot parse is rejected here, not
//!   at release time;
//! * the workspace, the server's memory, and the raw agent stream stay outside
//!   the evidence directory, so the CLI cannot observe its own logs (issue
//!   #936's completion gate feeding back on itself);
//! * server readiness is an explicit bounded loop that fails fast when the
//!   server process exits (curl's retry behavior differs between platforms);
//! * a run that reproduced the committed bytes has authored nothing, and the
//!   script neither opens a pull request nor pushes.

use std::error::Error;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::cli_solve::SolveArgs;

/// What one live authoring run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringOutcome {
    /// The resumable session id the Agent CLI stream reported.
    pub session_id: String,
    /// The model identifier written into the evidence and the trailer.
    pub model: String,
    /// Repository-relative paths the authored artifacts landed at.
    pub destinations: Vec<String>,
    /// Whether the four-trailer commit was written (`--no-commit` leaves it).
    pub committed: bool,
    /// The commit message, including its trailers, when one was written.
    pub commit_message: String,
}

/// Reject a pull-request reference the self-hosting metric cannot parse.
///
/// Equivalent to `^https://github\.com/[^/]+/[^/]+/pull/[1-9][0-9]*$`.
fn canonical_pull_request(pull_request: &str) -> Result<(), Box<dyn Error>> {
    let rest = pull_request
        .strip_prefix("https://github.com/")
        .ok_or_else(|| {
            format!("--pull-request must be a canonical GitHub pull-request URL: {pull_request}")
        })?;
    let segments: Vec<&str> = rest.split('/').collect();
    let number = match segments.as_slice() {
        [_, _, "pull", number] => *number,
        _ => {
            return Err(format!(
                "--pull-request must be a canonical GitHub pull-request URL: {pull_request}"
            )
            .into());
        }
    };
    if number.is_empty()
        || !number.bytes().all(|byte| byte.is_ascii_digit())
        || number.starts_with('0')
    {
        return Err(format!(
            "--pull-request must be a canonical GitHub pull-request URL: {pull_request}"
        )
        .into());
    }
    Ok(())
}

fn unique_scratch(kind: &str) -> PathBuf {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "formal-ai-{kind}-{}-{sequence}",
        std::process::id()
    ))
}

struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(kind: &str) -> Self {
        let path = unique_scratch(kind);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch directory");
        Self { path }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn differs(left: &Path, right: &Path) -> Result<bool, Box<dyn Error>> {
    if !right.exists() {
        return Ok(true);
    }
    let left_bytes = std::fs::read(left)?;
    let right_bytes = std::fs::read(right)?;
    Ok(left_bytes != right_bytes)
}

/// Run the live authoring loop: serve, drive the real Agent CLI, enforce the
/// artifact contract, and land the four-trailer commit on the checked-out
/// branch.
///
/// # Errors
/// Every refusal is exact: a hosted model is refused by [`crate::cli_solve::run_solve`]
/// before this runs; a malformed pull-request reference, a missing artifact, a
/// `--contains` miss, an unchanged seed replay, and a run that reproduced the
/// committed bytes all abort before any commit.
pub fn run_authoring(args: &SolveArgs) -> Result<AuthoringOutcome, Box<dyn Error>> {
    let task = args
        .task
        .clone()
        .ok_or("--task is required for the live authoring loop")?;
    let message = args
        .message
        .clone()
        .ok_or("--message is required for the live authoring loop")?;
    let pull_request = args
        .pull_request
        .clone()
        .ok_or("--pull-request is required for the live authoring loop")?;
    canonical_pull_request(&pull_request)?;
    if args.produces.is_empty() {
        return Err("--produces is required for the live authoring loop".into());
    }
    if args.into.len() > args.produces.len() {
        return Err("more --into than --produces".into());
    }
    let mut into = args.into.clone();
    while into.len() < args.produces.len() {
        into.push(args.produces[into.len()].clone());
    }

    let repository = PathBuf::from(&args.repository);
    let evidence = if args.evidence.is_absolute() {
        args.evidence.clone()
    } else {
        repository.join(&args.evidence)
    };
    let evidence_relative = args.evidence.to_string_lossy().to_string();

    // The workspace, the server's memory, and the raw stream live outside the
    // evidence directory: issue #936 recorded a completion gate feeding back on
    // itself when the live log sat inside the watched worktree.
    let work = Scratch::new("authoring-workspace");
    let state = Scratch::new("authoring-state");

    if let Some(seed) = &args.seed {
        let seed_path = if Path::new(seed).is_absolute() {
            PathBuf::from(seed)
        } else {
            repository.join(seed)
        };
        if !seed_path.is_dir() {
            return Err(format!("--seed is not a directory: {}", seed_path.display()).into());
        }
        copy_dir(&seed_path, &work.path)?;
    }

    std::fs::create_dir_all(&evidence)?;
    std::fs::write(evidence.join("task.txt"), format!("{task}\n"))?;

    let port_string = args.port.to_string();
    let server_binary = args
        .server_executable
        .clone()
        .or_else(|| std::env::var("FORMAL_AI_SERVER").ok().map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_exe().expect("current executable"));
    let server_log = std::fs::File::create(evidence.join("formal-ai.log"))?;
    let server_process = Command::new(&server_binary)
        .args(["serve", "--host", "127.0.0.1", "--port", &port_string])
        .env("FORMAL_AI_AGENT_MODE", "1")
        .env("FORMAL_AI_TRACE_REQUESTS", "1")
        .env("FORMAL_AI_MEMORY_PATH", state.path.join("memory.lino"))
        .env("FORMAL_AI_DREAMING", "0")
        .stdout(Stdio::from(server_log.try_clone()?))
        .stderr(Stdio::from(server_log))
        .spawn()?;
    // The bash harness removed the server on every exit path (`trap cleanup
    // EXIT`); a drop guard keeps that promise through every early return.
    let mut server = ServerGuard(server_process);

    // Readiness is an explicit bounded loop: curl's retry behavior differs
    // between platforms, and a server that exits while starting must fail
    // immediately instead of burning the deadline.
    let mut server_ready = false;
    for attempt in 1..=30 {
        if TcpStream::connect(("127.0.0.1", args.port)).is_ok() {
            server_ready = true;
            break;
        }
        if matches!(server.0.try_wait()?, Some(_)) {
            return Err(
                format!("formal-ai serve exited during startup (attempt {attempt})").into(),
            );
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    if !server_ready {
        return Err(format!("formal-ai serve never came up on port {}", args.port).into());
    }

    let agent_config = format!(
        "{{\"provider\":{{\"formalai\":{{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{{\"baseURL\":\"http://127.0.0.1:{port}/api/openai/v1\",\"apiKey\":\"local\"}},\"models\":{{\"formal-ai\":{{\"name\":\"Formal AI\"}}}}}},\"model\":\"formalai/formal-ai\"}}}}",
        port = args.port
    );
    // `--summarize-session` and `--generate-title` default to true, and both
    // make a second model call that ignores `--model` and goes to the CLI's
    // own default provider; neither call is needed to author a change.
    let agent_binary = args
        .agent_executable
        .clone()
        .or_else(|| std::env::var("AGENT").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("agent"));
    let raw_stream_path = state.path.join("agent-stream.raw.log");
    let raw_stream = std::fs::File::create(&raw_stream_path)?;
    let agent_stderr = std::fs::File::create(evidence.join("agent-stderr.log"))?;
    let mut path_env = std::env::var("PATH").unwrap_or_default();
    if let Some(parent) = server_binary.parent() {
        path_env = format!("{}:{}", parent.display(), path_env);
    }
    let agent_status = Command::new(&agent_binary)
        .current_dir(&work.path)
        .args([
            "--model",
            "formalai/formal-ai",
            "--permission-mode",
            "auto",
            "--no-summarize-session",
            "--no-generate-title",
            "--output-format",
            "stream-json",
            "--compact-json",
            "--disable-stdin",
            "--prompt",
            &task,
        ])
        .env("PATH", &path_env)
        .env("FORMAL_AI_API_KEY", "local")
        .env("LINK_ASSISTANT_AGENT_CONFIG_CONTENT", &agent_config)
        .stdout(Stdio::from(raw_stream))
        .stderr(Stdio::from(agent_stderr))
        .status()?;
    if !agent_status.success() {
        return Err(format!("the Agent CLI exited {agent_status}").into());
    }

    let classifier = repository.join("scripts/classify-agent-cli-stderr.sh");
    if !classifier.exists() {
        return Err(format!("the stderr classifier is missing: {}", classifier.display()).into());
    }
    let classification = Command::new("sh")
        .arg(&classifier)
        .arg(evidence.join("agent-stderr.log"))
        .status()?;
    if !classification.success() {
        return Err("the Agent CLI stderr classified as a known fatal class".into());
    }

    // Only the framed events are kept: the raw stream repeats them verbatim
    // around the CLI's own progress chatter, and committing both would double
    // the evidence for no extra proof.
    let raw = std::fs::read_to_string(&raw_stream_path)?;
    let mut framed = String::new();
    for line in raw.lines() {
        if line.starts_with('{') {
            framed.push_str(line);
            framed.push('\n');
        }
    }
    std::fs::write(evidence.join("agent-stream.jsonl"), &framed)?;

    let marker = "\"session_id\":\"";
    let session_id = match framed.find(marker) {
        Some(start) => {
            let rest = &framed[start + marker.len()..];
            rest.split('"').next().unwrap_or_default().to_owned()
        }
        None => String::new(),
    };
    if !session_id.starts_with("ses_") || session_id.len() == "ses_".len() {
        return Err("the Agent CLI stream reported no resumable session id".into());
    }
    // One evidence file carries both markers the metric looks for: the literal
    // `formal-ai` that identifies the producer, and the session the trailer
    // names. Issue #1085 (D3.1): the metric attributes by the model that
    // produced the tokens, so the evidence names it next to the session.
    let model = format!("formal-ai/{}", env!("CARGO_PKG_VERSION"));
    std::fs::write(
        evidence.join("session-id.txt"),
        format!("formal-ai session {session_id}\nformal-ai model {model}\n"),
    )?;

    for produced in &args.produces {
        if !work.path.join(produced).is_file() {
            return Err(format!("the Agent CLI did not write {produced}").into());
        }
    }
    // Every `--contains` is checked against the whole set, because the text
    // that proves the change landed lives in one of the artifacts, not in each
    // of them.
    for expected in &args.contains {
        let mut found = false;
        for produced in &args.produces {
            let bytes = std::fs::read(work.path.join(produced))?;
            if contains_bytes(&bytes, expected.as_bytes()) {
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("no artifact contains: {expected}").into());
        }
    }

    // Seed files are context, not authored effects. New logs cannot turn an
    // unchanged seed or already-landed output into another contribution either.
    // Check the whole artifact set before copying anything, including
    // `--no-commit`: the seed/destination comparison decides *whether* this
    // run authored anything, never *what* lands — every produced artifact is
    // landed so an unchanged companion file still reaches its destination.
    let mut authored_change = false;
    for (index, produced) in args.produces.iter().enumerate() {
        let produced_path = work.path.join(produced);
        if let Some(seed) = &args.seed {
            let seed_path = if Path::new(seed).is_absolute() {
                PathBuf::from(seed)
            } else {
                repository.join(seed)
            };
            if !differs(&produced_path, &seed_path.join(produced))? {
                continue;
            }
        }
        let destination = &into[index];
        if differs(&produced_path, &repository.join(destination))? {
            authored_change = true;
        }
    }
    if !authored_change {
        return Err(
            "no produced artifact differs from both its seed and destination; no change was authored"
                .into(),
        );
    }
    let mut destinations = Vec::new();
    for (index, produced) in args.produces.iter().enumerate() {
        let destination = repository.join(&into[index]);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(work.path.join(produced), &destination)?;
        destinations.push(into[index].clone());
    }
    println!(
        "Formal AI wrote {} in session {session_id}; evidence in {evidence_relative}",
        destinations.join(" ")
    );

    if !args.commit {
        println!(
            "--no-commit: leaving {} and {evidence_relative} unstaged for review",
            destinations.join(" ")
        );
        return Ok(AuthoringOutcome {
            session_id,
            model,
            destinations,
            committed: false,
            commit_message: String::new(),
        });
    }

    let mut add = Command::new("git");
    add.current_dir(&repository).arg("add").arg("--");
    for destination in &destinations {
        add.arg(destination);
    }
    add.arg(&evidence_relative);
    if !add.status()?.success() {
        return Err("git add failed for the authored artifacts".into());
    }
    let reproduced = Command::new("git")
        .current_dir(&repository)
        .args(["diff", "--cached", "--quiet"])
        .status()?;
    if reproduced.success() {
        return Err("the run reproduced the committed bytes; nothing to author".into());
    }
    let trailers = format!(
        "Formal-AI-Session: {session_id}\nFormal-AI-Model: {model}\nFormal-AI-Evidence: {evidence_relative}\nFormal-AI-Pull-Request: {pull_request}\n"
    );
    let commit_message = format!("{message}\n\n{trailers}");
    let commit = Command::new("git")
        .current_dir(&repository)
        .args(["commit", "--quiet", "-m", &message, "-m", &trailers])
        .status()?;
    if !commit.success() {
        return Err("git commit failed for the authored change".into());
    }
    let _ = Command::new("git")
        .current_dir(&repository)
        .args(["--no-pager", "log", "-1", "--format=%h %s%n%b"])
        .status()?;

    Ok(AuthoringOutcome {
        session_id,
        model,
        destinations,
        committed: true,
        commit_message,
    })
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

struct ServerGuard(Child);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_pull_request;

    #[test]
    fn a_canonical_pull_request_url_is_accepted() {
        assert!(
            canonical_pull_request("https://github.com/link-assistant/formal-ai/pull/888").is_ok()
        );
    }

    #[test]
    fn a_pull_request_url_the_metric_cannot_parse_is_rejected_here() {
        for malformed in [
            "https://github.com/link-assistant/formal-ai/pull/0888",
            "https://github.com/link-assistant/formal-ai/pull/",
            "https://github.com/link-assistant/formal-ai/issues/888",
            "http://github.com/link-assistant/formal-ai/pull/888",
            "https://github.com/link-assistant/formal-ai/pull/888/comments",
        ] {
            assert!(
                canonical_pull_request(malformed).is_err(),
                "{malformed} must be rejected"
            );
        }
    }
}
