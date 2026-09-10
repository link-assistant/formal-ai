//! Who produced a change, and which changed lines are behaviour (issue #1085).
//!
//! Split out of `self-hosting-metric.rs` to keep that file under the per-file
//! ceiling `scripts/check-file-size.rs` enforces.
//!
//! Metric version 3 changes two things about attribution:
//!
//! * **The model, not the trailer.** Version 2 credited any commit whose
//!   `Formal-AI-Session` id appeared in a committed file containing the string
//!   `formal-ai`. 72 of the 125 attributed commits on `main` carried session ids
//!   of the form `issue-NNN-claude-2026MMDD`, and the sampled evidence file
//!   opened with `Agent: formal-ai (Claude Opus 4.8) via /solve`; release
//!   `v0.307.0` credited 63% of its lines to Formal AI on that basis. A commit
//!   now also has to carry `Formal-AI-Model`, the model has to be `formal-ai`,
//!   the committed evidence has to name it, and a session or model that names a
//!   hosted model is simply not self-authored.
//! * **Behaviour, not paperwork.** Paths under `docs/`, `dev/`, `experiments/`
//!   and `changelog.d/` describe Formal AI; they do not change what it does.
//!   They leave both the numerator and the denominator, the same symmetric
//!   treatment version 2 gave captured artifacts and lockfiles.

use std::path::Path;
use std::process::Command;

use super::{git, trailer_values};

pub const MODEL_TRAILER: &str = "Formal-AI-Model";

/// A model name or session id containing one of these was produced by a hosted
/// model, whatever the trailers beside it claim.
pub const HOSTED_MODEL_MARKERS: &[&str] = &[
    "claude",
    "codex",
    "gemini",
    "opencode",
    "qwen",
    "gpt-",
    "openai",
    "anthropic",
];

/// Paths that describe the system rather than change its behaviour.
pub const NON_BEHAVIOUR_PREFIXES: &[&str] = &["docs/", "dev/", "experiments/", "changelog.d/"];

/// File extensions whose content is captured, not authored: CI run logs, agent
/// transcripts, saved diffs and process output. Committing them is deliberate
/// (they are the evidence bundles this repository requires), but they are not
/// release work and must not move the metric (issue #812).
pub const CAPTURED_ARTIFACT_EXTENSIONS: &[&str] =
    &["log", "jsonl", "diff", "patch", "stderr", "stdout"];

/// Dependency lockfiles: written by a package manager, never hand-authored.
pub const LOCKFILE_NAMES: &[&str] = &[
    "Cargo.lock",
    "bun.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "poetry.lock",
    "uv.lock",
    "composer.lock",
    "Gemfile.lock",
];

/// Whether a path holds something other than authored behaviour.
///
/// Applied symmetrically to the numerator and the denominator, so a commit that
/// only files evidence contributes nothing either way and the share it reports
/// is unchanged by how much paperwork happened to be attached to it.
pub fn is_non_authored_path(path: &str) -> bool {
    // `git show --numstat` C-quotes any path with non-printable or non-ASCII
    // bytes, so the raw field can arrive wrapped in double quotes.
    let path = path.trim().trim_matches('"');
    if NON_BEHAVIOUR_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return true;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    if LOCKFILE_NAMES.contains(&name) {
        return true;
    }
    name.rsplit_once('.').is_some_and(|(_, extension)| {
        CAPTURED_ARTIFACT_EXTENSIONS
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    })
}

/// The hosted-model marker a value names, if any.
pub fn hosted_model_marker(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    HOSTED_MODEL_MARKERS
        .iter()
        .copied()
        .find(|marker| lower.contains(marker))
}

/// Decide whether a session-backed commit was produced by the formal-ai model.
///
/// `Ok(Some(model))` attributes the commit. `Ok(None)` does not, and says why on
/// stderr: the claim is well formed but names a hosted model, so under either
/// evidence policy the commit is ordinary work, not an error. `Err` is a
/// malformed claim -- no `Formal-AI-Model`, more than one, or a model the
/// committed evidence never names -- which the strict pull-request gate
/// rejects and the lenient release path reports and skips.
pub fn model_attribution(
    repo: &Path,
    commit: &str,
    sessions: &[String],
    evidence: &[(String, String)],
) -> Result<Option<String>, String> {
    for session in sessions {
        if let Some(marker) = hosted_model_marker(session) {
            eprintln!(
                "not attributing {commit}: session {session} names the hosted model marker `{marker}`"
            );
            return Ok(None);
        }
    }
    let models = trailer_values(repo, commit, MODEL_TRAILER)?;
    if models.is_empty() {
        return Err(format!(
            "commit {commit} must record {MODEL_TRAILER} (issue #1085): the model that produced \
             the change, `formal-ai/<version>` for this loop"
        ));
    }
    if models.len() > 1 {
        return Err(format!(
            "commit {commit} records more than one {MODEL_TRAILER}"
        ));
    }
    let model = models[0].clone();
    if !model.to_ascii_lowercase().starts_with("formal-ai") || hosted_model_marker(&model).is_some()
    {
        eprintln!("not attributing {commit}: {MODEL_TRAILER} names `{model}`, not formal-ai");
        return Ok(None);
    }
    if !evidence
        .iter()
        .any(|(_, content)| content.contains(model.as_str()))
    {
        return Err(format!(
            "no committed evidence in {commit} records model {model}"
        ));
    }
    Ok(Some(model))
}

/// Who opened a pull request, read through `gh` when the caller opted in with
/// `FORMAL_AI_PULL_REQUEST_AUTHOR_LOOKUP=gh`; `unrecorded` otherwise, so an
/// offline fixture and a release without the tool both stay deterministic.
pub fn pull_request_author(reference: &str) -> String {
    const UNRECORDED: &str = "unrecorded";
    if std::env::var("FORMAL_AI_PULL_REQUEST_AUTHOR_LOOKUP").as_deref() != Ok("gh") {
        return UNRECORDED.to_owned();
    }
    let Some(path) = reference.strip_prefix("https://github.com/") else {
        return UNRECORDED.to_owned();
    };
    let parts = path.split('/').collect::<Vec<_>>();
    let [owner, repository, "pull", number] = parts.as_slice() else {
        return UNRECORDED.to_owned();
    };
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            number,
            "--repo",
            &format!("{owner}/{repository}"),
            "--json",
            "author",
            "--jq",
            ".author.login",
        ])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let login = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if login.is_empty() {
                UNRECORDED.to_owned()
            } else {
                login
            }
        }
        _ => UNRECORDED.to_owned(),
    }
}

/// Whether `revision` resolves in this checkout; a replay over history has to
/// skip a range whose ends are not present rather than fail the whole epoch.
pub fn revision_present(repo: &Path, revision: &str) -> bool {
    git(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{revision}^{{commit}}"),
        ],
    )
    .is_ok()
}
