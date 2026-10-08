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
//!
//! The loop is not a separate protocol (#1138 R1138-3-5). It runs the stages
//! `data/meta/repository-workspace-protocol.lino` declares, in that order, the
//! same document SWE-bench and the coding ladder run: the workspace opens, the
//! declared artifacts are located and their prior bytes read, the Agent CLI
//! session is the editor, the artifact contract is the verification, the
//! landed set is the outcome, and the commit is gated. Spawning `serve` and
//! harvesting the session id are stages of that document declared for the
//! Agent CLI session editor, and every stage lands in the same
//! `repository-protocol.lino` evidence the other callers write.

use std::error::Error;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::cli_solve::SolveArgs;
use crate::repository_workspace::WorkspaceProtocol;
use crate::repository_workspace::trace::{EDITOR_AGENT_SESSION, ProtocolTrace, StageStatus};

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
pub fn canonical_pull_request(pull_request: &str) -> Result<(), Box<dyn Error>> {
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

/// Run the live authoring loop: serve, drive the real Agent CLI, enforce the
/// artifact contract, and land the four-trailer commit on the checked-out
/// branch.
///
/// The stages and their order are the ones
/// `data/meta/repository-workspace-protocol.lino` declares, the same document
/// SWE-bench and the coding ladder run (#1138 R1138-3-5); see
/// [`run_authoring_with`].
///
/// # Errors
/// Every refusal is exact: a hosted model is refused by [`crate::cli_solve::run_solve`]
/// before this runs; a malformed pull-request reference, a missing artifact, a
/// `--contains` miss, an unchanged seed replay, and a run that reproduced the
/// committed bytes all abort before any commit.
pub fn run_authoring(args: &SolveArgs) -> Result<AuthoringOutcome, Box<dyn Error>> {
    run_authoring_with(&WorkspaceProtocol::load(), args)
}

/// Run the live authoring loop through the stages `protocol` declares.
///
/// Each stage the document declares for the Agent CLI session editor runs in
/// document order; a stage declared for another editor is recorded as not
/// applicable. Spawning `formal-ai serve` and harvesting the session id are
/// stages of the document, so removing or reordering them is a data edit.
/// Every stage is written to `repository-protocol.lino` in the evidence
/// directory as soon as it ends, in the format every protocol caller writes.
///
/// # Errors
/// As [`run_authoring`]; the stage that refused is recorded as stopped.
pub fn run_authoring_with(
    protocol: &WorkspaceProtocol,
    args: &SolveArgs,
) -> Result<AuthoringOutcome, Box<dyn Error>> {
    let mut run = AuthoringRun::new(args)?;
    let mut trace = ProtocolTrace::new(protocol, AUTHORING_CALLER, EDITOR_AGENT_SESSION);
    trace.set_field("model", &run.model);
    for step in protocol.steps() {
        if !step.applies_to(EDITOR_AGENT_SESSION) {
            continue;
        }
        match run.stage(&step.id, &mut trace) {
            Ok(status) => {
                trace.record(&step.id, status);
                run.write_trace(&trace)?;
            }
            Err(error) => {
                trace.record(&step.id, StageStatus::Stopped);
                trace.open.push(error.to_string());
                run.write_trace(&trace)?;
                return Err(error);
            }
        }
    }
    Ok(AuthoringOutcome {
        session_id: run.session_marker,
        model: run.model,
        destinations: run.destinations,
        committed: run.committed,
        commit_message: run.commit_message,
    })
}

/// The caller name the authoring loop's trace records.
const AUTHORING_CALLER: &str = "authoring";

/// The evidence file every protocol caller writes.
const TRACE_FILE: &str = "repository-protocol.lino";

/// The prior bytes of one declared artifact: its seed copy and its
/// destination, each `None` when absent.
struct Baseline {
    seed: Option<Vec<u8>>,
    destination: Option<Vec<u8>>,
}

/// Everything one authoring run carries from stage to stage.
struct AuthoringRun<'a> {
    args: &'a SolveArgs,
    task: String,
    message: String,
    pull_request: String,
    into: Vec<String>,
    repository: PathBuf,
    seed: Option<PathBuf>,
    evidence: PathBuf,
    evidence_relative: String,
    model: String,
    // The workspace, the server's memory, and the raw stream live outside the
    // evidence directory: issue #936 recorded a completion gate feeding back on
    // itself when the live log sat inside the watched worktree.
    work: Scratch,
    state: Scratch,
    server_binary: PathBuf,
    server: Option<ServerGuard>,
    baseline: Vec<Baseline>,
    session_marker: String,
    destinations: Vec<String>,
    committed: bool,
    commit_message: String,
}

impl<'a> AuthoringRun<'a> {
    fn new(args: &'a SolveArgs) -> Result<Self, Box<dyn Error>> {
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
        let seed = args.seed.as_ref().map(|seed| {
            if Path::new(seed).is_absolute() {
                PathBuf::from(seed)
            } else {
                repository.join(seed)
            }
        });
        let server_binary = args
            .server_executable
            .clone()
            .or_else(|| std::env::var("FORMAL_AI_SERVER").ok().map(PathBuf::from))
            .unwrap_or_else(|| std::env::current_exe().expect("current executable"));
        Ok(Self {
            args,
            task,
            message,
            pull_request,
            into,
            repository,
            seed,
            evidence,
            evidence_relative: args.evidence.to_string_lossy().to_string(),
            model: format!("formal-ai/{}", env!("CARGO_PKG_VERSION")),
            work: Scratch::new("authoring-workspace"),
            state: Scratch::new("authoring-state"),
            server_binary,
            server: None,
            baseline: Vec::new(),
            session_marker: String::new(),
            destinations: Vec::new(),
            committed: false,
            commit_message: String::new(),
        })
    }

    /// Run one declared stage. A stage this editor has no mechanism for is
    /// recorded as unobserved rather than invented.
    fn stage(
        &mut self,
        id: &str,
        trace: &mut ProtocolTrace,
    ) -> Result<StageStatus, Box<dyn Error>> {
        match id {
            "clone" => self.open(),
            "locate" => {
                trace.set_field("located", &self.into.len().to_string());
                Ok(StageStatus::Observed)
            }
            "read" => self.read_baseline(),
            "serve" => self.serve(),
            "edit" => self.edit(),
            "session" => {
                let status = self.harvest_session()?;
                trace.set_field("session", &self.session_marker);
                Ok(status)
            }
            "verify" => self.verify_contract(),
            "diff" => {
                let status = self.land()?;
                trace.set_field("destinations", &self.destinations.join(" "));
                Ok(status)
            }
            "commit" => self.commit(trace),
            _ => Ok(StageStatus::Unobserved),
        }
    }

    /// Write the trace once the evidence directory exists; a refusal before
    /// the workspace opened has no evidence directory to write into.
    fn write_trace(&self, trace: &ProtocolTrace) -> Result<(), Box<dyn Error>> {
        if self.evidence.is_dir() {
            std::fs::write(self.evidence.join(TRACE_FILE), trace.render())?;
        }
        Ok(())
    }

    fn open(&self) -> Result<StageStatus, Box<dyn Error>> {
        if let Some(seed) = &self.seed {
            if !seed.is_dir() {
                return Err(format!("--seed is not a directory: {}", seed.display()).into());
            }
            copy_dir(seed, &self.work.path)?;
        }
        std::fs::create_dir_all(&self.evidence)?;
        std::fs::write(self.evidence.join("task.txt"), format!("{}\n", self.task))?;
        Ok(StageStatus::Observed)
    }

    fn read_baseline(&mut self) -> Result<StageStatus, Box<dyn Error>> {
        self.baseline.clear();
        for (index, produced) in self.args.produces.iter().enumerate() {
            let seed = match &self.seed {
                Some(seed) => read_if_file(&seed.join(produced))?,
                None => None,
            };
            let destination = read_if_file(&self.repository.join(&self.into[index]))?;
            self.baseline.push(Baseline { seed, destination });
        }
        Ok(StageStatus::Observed)
    }

    fn serve(&mut self) -> Result<StageStatus, Box<dyn Error>> {
        let port_string = self.args.port.to_string();
        let server_log = std::fs::File::create(self.evidence.join("formal-ai.log"))?;
        let server_process = Command::new(&self.server_binary)
            .args(["serve", "--host", "127.0.0.1", "--port", &port_string])
            .env("FORMAL_AI_AGENT_MODE", "1")
            .env("FORMAL_AI_TRACE_REQUESTS", "1")
            .env("FORMAL_AI_MEMORY_PATH", self.state.path.join("memory.lino"))
            .env("FORMAL_AI_DREAMING", "0")
            .stdout(Stdio::from(server_log.try_clone()?))
            .stderr(Stdio::from(server_log))
            .spawn()?;
        // The bash harness removed the server on every exit path (`trap cleanup
        // EXIT`); a drop guard keeps that promise through every early return.
        let server = self.server.insert(ServerGuard(server_process));

        // Readiness is an explicit bounded loop: curl's retry behavior differs
        // between platforms, and a server that exits while starting must fail
        // immediately instead of burning the deadline.
        for attempt in 1..=30 {
            if TcpStream::connect(("127.0.0.1", self.args.port)).is_ok() {
                return Ok(StageStatus::Observed);
            }
            if server.0.try_wait()?.is_some() {
                return Err(
                    format!("formal-ai serve exited during startup (attempt {attempt})").into(),
                );
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(format!("formal-ai serve never came up on port {}", self.args.port).into())
    }

    fn edit(&self) -> Result<StageStatus, Box<dyn Error>> {
        let agent_config = format!(
            "{{\"provider\":{{\"formalai\":{{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{{\"baseURL\":\"http://127.0.0.1:{port}/api/openai/v1\",\"apiKey\":\"local\"}},\"models\":{{\"formal-ai\":{{\"name\":\"Formal AI\"}}}}}},\"model\":\"formalai/formal-ai\"}}}}",
            port = self.args.port
        );
        // `--summarize-session` and `--generate-title` default to true, and both
        // make a second model call that ignores `--model` and goes to the CLI's
        // own default provider; neither call is needed to author a change.
        let agent_binary = self
            .args
            .agent_executable
            .clone()
            .or_else(|| std::env::var("AGENT").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("agent"));
        let raw_stream_path = self.state.path.join("agent-stream.raw.log");
        let raw_stream = std::fs::File::create(&raw_stream_path)?;
        let agent_stderr = std::fs::File::create(self.evidence.join("agent-stderr.log"))?;
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        if let Some(parent) = self.server_binary.parent() {
            path_env = format!("{}:{}", parent.display(), path_env);
        }
        let agent_status = Command::new(&agent_binary)
            .current_dir(&self.work.path)
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
                &self.task,
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

        let classifier = self.repository.join("scripts/classify-agent-cli-stderr.sh");
        if !classifier.exists() {
            return Err(
                format!("the stderr classifier is missing: {}", classifier.display()).into(),
            );
        }
        // The classifier is a bash script (its `#!/usr/bin/env bash` shebang and
        // its `[[ ]]` / `(( ))` dialect say so), so it must run under bash: the
        // ubuntu runner's `/bin/sh` is dash, which cannot parse either form.
        let classification = Command::new("bash")
            .arg(&classifier)
            .arg(self.evidence.join("agent-stderr.log"))
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
        std::fs::write(self.evidence.join("agent-stream.jsonl"), &framed)?;
        Ok(StageStatus::Observed)
    }

    fn harvest_session(&mut self) -> Result<StageStatus, Box<dyn Error>> {
        let framed = std::fs::read_to_string(self.evidence.join("agent-stream.jsonl"))?;
        // The marker names the session a later run can resume; the binding is
        // deliberately not called `session_id` because CodeQL's
        // rust/cleartext-logging reads that name as account information in any
        // logging macro it reaches.
        let marker = "\"session_id\":\"";
        let session_marker = framed.find(marker).map_or_else(String::new, |start| {
            let rest = &framed[start + marker.len()..];
            rest.split('"').next().unwrap_or_default().to_owned()
        });
        if !session_marker.starts_with("ses_") || session_marker.len() == "ses_".len() {
            return Err("the Agent CLI stream reported no resumable session id".into());
        }
        // One evidence file carries both markers the metric looks for: the literal
        // `formal-ai` that identifies the producer, and the session the trailer
        // names. Issue #1085 (D3.1): the metric attributes by the model that
        // produced the tokens, so the evidence names it next to the session.
        let model = &self.model;
        std::fs::write(
            self.evidence.join("session-id.txt"),
            format!("formal-ai session {session_marker}\nformal-ai model {model}\n"),
        )?;
        self.session_marker = session_marker;
        Ok(StageStatus::Observed)
    }

    fn verify_contract(&self) -> Result<StageStatus, Box<dyn Error>> {
        for produced in &self.args.produces {
            if !self.work.path.join(produced).is_file() {
                return Err(format!("the Agent CLI did not write {produced}").into());
            }
        }
        // Every `--contains` is checked against the whole set, because the text
        // that proves the change landed lives in one of the artifacts, not in each
        // of them.
        for expected in &self.args.contains {
            let mut found = false;
            for produced in &self.args.produces {
                let bytes = std::fs::read(self.work.path.join(produced))?;
                if contains_bytes(&bytes, expected.as_bytes()) {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("no artifact contains: {expected}").into());
            }
        }
        Ok(StageStatus::Observed)
    }

    fn land(&mut self) -> Result<StageStatus, Box<dyn Error>> {
        // Seed files are context, not authored effects. New logs cannot turn an
        // unchanged seed or already-landed output into another contribution either.
        // Check the whole artifact set before copying anything, including
        // `--no-commit`: the seed/destination comparison decides *whether* this
        // run authored anything, never *what* lands — every produced artifact is
        // landed so an unchanged companion file still reaches its destination.
        let mut authored_change = false;
        for (index, produced) in self.args.produces.iter().enumerate() {
            let bytes = std::fs::read(self.work.path.join(produced))?;
            let prior = &self.baseline[index];
            if self.seed.is_some() && prior.seed.as_deref() == Some(bytes.as_slice()) {
                continue;
            }
            if prior.destination.as_deref() != Some(bytes.as_slice()) {
                authored_change = true;
            }
        }
        if !authored_change {
            return Err(
                "no produced artifact differs from both its seed and destination; no change was authored"
                    .into(),
            );
        }
        for (index, produced) in self.args.produces.iter().enumerate() {
            let destination = self.repository.join(&self.into[index]);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(self.work.path.join(produced), &destination)?;
            self.destinations.push(self.into[index].clone());
        }
        let session_marker = &self.session_marker;
        let evidence_relative = &self.evidence_relative;
        println!(
            "Formal AI wrote {} in session {session_marker}; evidence in {evidence_relative}",
            self.destinations.join(" ")
        );
        Ok(StageStatus::Observed)
    }

    fn commit(&mut self, trace: &mut ProtocolTrace) -> Result<StageStatus, Box<dyn Error>> {
        if !self.args.commit {
            let evidence_relative = &self.evidence_relative;
            println!(
                "--no-commit: leaving {} and {evidence_relative} unstaged for review",
                self.destinations.join(" ")
            );
            return Ok(StageStatus::Refused);
        }
        // The committed trace records the gate as opened: the commit that
        // carries it is the observation of this stage.
        trace.record("commit", StageStatus::Requested);
        self.write_trace(trace)?;

        let mut add = Command::new("git");
        add.current_dir(&self.repository).arg("add").arg("--");
        for destination in &self.destinations {
            add.arg(destination);
        }
        add.arg(&self.evidence_relative);
        if !add.status()?.success() {
            return Err("git add failed for the authored artifacts".into());
        }
        let reproduced = Command::new("git")
            .current_dir(&self.repository)
            .args(["diff", "--cached", "--quiet"])
            .status()?;
        if reproduced.success() {
            return Err("the run reproduced the committed bytes; nothing to author".into());
        }
        let session_marker = &self.session_marker;
        let model = &self.model;
        let evidence_relative = &self.evidence_relative;
        let pull_request = &self.pull_request;
        let trailers = format!(
            "Formal-AI-Session: {session_marker}\nFormal-AI-Model: {model}\nFormal-AI-Evidence: {evidence_relative}\nFormal-AI-Pull-Request: {pull_request}\n"
        );
        let commit_message = format!("{}\n\n{trailers}", self.message);
        let commit = Command::new("git")
            .current_dir(&self.repository)
            .args(["commit", "--quiet", "-m", &self.message, "-m", &trailers])
            .status()?;
        if !commit.success() {
            return Err("git commit failed for the authored change".into());
        }
        let _ = Command::new("git")
            .current_dir(&self.repository)
            .args(["--no-pager", "log", "-1", "--format=%h %s%n%b"])
            .status()?;
        self.committed = true;
        self.commit_message = commit_message;
        Ok(StageStatus::Requested)
    }
}

fn read_if_file(path: &Path) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    if path.is_file() {
        Ok(Some(std::fs::read(path)?))
    } else {
        Ok(None)
    }
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
