//! Git attribution for verified external-agent effects.
//!
//! Attribution is opt-in because committing is a materially different action
//! from composing files. Once requested it fails closed: the workspace must be
//! a clean Git worktree, the pull-request URL must be canonical, and every
//! committed effect must have a native agent session plus committed evidence.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::process::{Command, Output};

use super::dispatch::DispatchError;
use super::runner::AgentSession;

const SESSION_TRAILER: &str = "Formal-AI-Session";
const EVIDENCE_TRAILER: &str = "Formal-AI-Evidence";
const PULL_REQUEST_TRAILER: &str = "Formal-AI-Pull-Request";

pub(super) fn prepare_attribution(
    workspace: &Path,
    pull_request: &str,
) -> Result<(), DispatchError> {
    if pull_request_number(pull_request).is_none() {
        return Err(attribution_error("invalid_pull_request_url"));
    }
    let inside = git(workspace, ["rev-parse", "--is-inside-work-tree"])?;
    if String::from_utf8_lossy(&inside.stdout).trim() != "true" {
        return Err(attribution_error("workspace_not_git_worktree"));
    }
    let status = git(
        workspace,
        ["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if !status.stdout.is_empty() {
        return Err(attribution_error("workspace_not_clean"));
    }
    Ok(())
}

pub(super) fn validate_effect_attribution(
    workspace: &Path,
    session_file: &Path,
    session: &AgentSession,
) -> Result<(), DispatchError> {
    if session.changes.is_empty() {
        return Ok(());
    }
    let native = session
        .native_session
        .as_ref()
        .ok_or_else(|| attribution_error("native_session_unavailable"))?;
    if native.id.is_empty() || native.id.chars().any(char::is_control) {
        return Err(attribution_error("invalid_native_session_id"));
    }
    evidence_path(workspace, session_file)?;
    Ok(())
}

pub(super) fn commit_verified_effect(
    workspace: &Path,
    session_file: &Path,
    session: &AgentSession,
    pull_request: &str,
) -> Result<(), DispatchError> {
    if session.changes.is_empty() {
        return Ok(());
    }
    validate_effect_attribution(workspace, session_file, session)?;
    let native = session
        .native_session
        .as_ref()
        .expect("validated non-empty effect has a native session");
    let evidence = evidence_path(workspace, session_file)?;

    let mut paths = session
        .changes
        .iter()
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();
    paths.sort_unstable();
    paths.dedup();
    let mut add_arguments = vec!["add", "-A", "--"];
    add_arguments.extend(paths.iter().copied());
    git(workspace, add_arguments)?;
    git(workspace, ["add", "-f", "--", evidence.as_str()])?;

    let staged = git(
        workspace,
        [
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--diff-filter=ACDMRTUXB",
        ],
    )?;
    let actual = staged
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8_lossy(path).into_owned())
        .collect::<BTreeSet<_>>();
    let mut expected = paths
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    expected.insert(evidence.clone());
    if actual != expected {
        return Err(attribution_error("staged_paths_mismatch"));
    }

    let trailers = [
        trailer(SESSION_TRAILER, &native.id),
        trailer(EVIDENCE_TRAILER, &evidence),
        trailer(PULL_REQUEST_TRAILER, pull_request),
    ]
    .join("\n");
    git(
        workspace,
        [
            "commit",
            "-m",
            "formal-ai: apply verified agent effect",
            "-m",
            &trailers,
        ],
    )?;
    Ok(())
}

fn evidence_path(workspace: &Path, session_file: &Path) -> Result<String, DispatchError> {
    if !session_file.is_file() {
        return Err(attribution_error("session_evidence_missing"));
    }
    let relative = session_file
        .strip_prefix(workspace)
        .map_err(|_| attribution_error("session_evidence_outside_workspace"))?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(attribution_error("invalid_session_evidence_path"));
    }
    let relative = relative
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| attribution_error("non_utf8_session_evidence_path"))?;
    if relative.contains(':') {
        return Err(attribution_error("invalid_session_evidence_path"));
    }
    Ok(relative)
}

fn pull_request_number(reference: &str) -> Option<u64> {
    let path = reference.strip_prefix("https://github.com/")?;
    let mut components = path.split('/');
    let owner = components.next()?;
    let repository = components.next()?;
    let pull = components.next()?;
    let number = components.next()?;
    let safe_component = |component: &str| {
        !component.is_empty()
            && component != "."
            && component != ".."
            && component.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            })
    };
    if !safe_component(owner)
        || !safe_component(repository)
        || pull != "pull"
        || components.next().is_some()
    {
        return None;
    }
    number.parse::<u64>().ok().filter(|number| *number > 0)
}

fn git<I, S>(workspace: &Path, args: I) -> Result<Output, DispatchError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let arguments = args
        .into_iter()
        .map(|argument| argument.as_ref().to_os_string())
        .collect::<Vec<_>>();
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(&arguments)
        .env("GIT_LITERAL_PATHSPECS", "1")
        .output()
        .map_err(|error| attribution_error(format!("git_process:{error}")))?;
    if !output.status.success() {
        let command = arguments
            .iter()
            .map(|argument| argument.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        return Err(attribution_error(format!(
            "git_{command}:{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output)
}

fn trailer(name: &str, value: &str) -> String {
    let mut trailer = String::with_capacity(name.len() + value.len() + 2);
    trailer.push_str(name);
    trailer.push(':');
    trailer.push(' ');
    trailer.push_str(value);
    trailer
}

fn attribution_error(message: impl Into<String>) -> DispatchError {
    DispatchError::Attribution(message.into())
}
