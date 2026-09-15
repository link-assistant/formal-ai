//! Committing and pushing what a task produced (issue #1133).
//!
//! A request that names a pull request or an issue -- "Update the pull request
//! at …", "Keep the solution on branch …" -- is asking for the result to land
//! there, and a result that only exists in the working tree has not landed.
//! Hive Mind's Scala run wrote `Main.scala`, reported completion, and left the
//! file untracked; its restart prompt then asked to "review these changes and
//! commit them" and that sentence went to web search. Both halves live here:
//! the commit step a work item ends with, and the route that answers a request
//! to commit what is already in the tree.

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability};
use super::progress::Progress;
use super::write_request::{bare_surfaces, clean_cue_token, tokens};
use crate::engine::ExecutionRecipe;
use crate::protocol::ChatMessage;
use crate::seed;

/// Where a work item's result is to land.
pub(super) struct CommitTarget {
    /// The branch the request named, when it named one; otherwise the branch
    /// that is checked out (`HEAD`).
    pub(super) branch: Option<String>,
    /// The issue or pull request the work item resolves.
    pub(super) reference: String,
}

impl CommitTarget {
    /// The branch the push names.
    pub(super) fn push_ref(&self) -> &str {
        self.branch.as_deref().unwrap_or("HEAD")
    }
}

/// The commit target a request states, if it names a repository work item.
pub(super) fn target_of(request: &str) -> Option<CommitTarget> {
    let reference = super::general_planner::repository_work_reference(request)?;
    Some(CommitTarget {
        branch: named_branch(request),
        reference,
    })
}

/// The branch named right after a branch cue: "on branch issue-1-abc",
/// "в ветке issue-1-abc", "分支 issue-1-abc".
pub(super) fn named_branch(request: &str) -> Option<String> {
    let cues = bare_surfaces(seed::ROLE_GIT_BRANCH_CUE);
    let toks = tokens(request);
    toks.windows(2).find_map(|pair| {
        let word = clean_cue_token(pair[0].text);
        let is_cue = cues
            .iter()
            .any(|cue| *cue == word || (!cue.is_ascii() && word.ends_with(cue.as_str())));
        if !is_cue {
            return None;
        }
        let candidate = pair[1]
            .text
            .trim_matches(|character: char| matches!(character, '`' | '"' | '\'' | ',' | '.' | ';' | ':' | '(' | ')'));
        is_branch_name(candidate).then(|| candidate.to_owned())
    })
}

fn is_branch_name(word: &str) -> bool {
    !word.is_empty()
        && !word.starts_with('-')
        && word.chars().any(|character| character.is_ascii_alphanumeric())
        && word
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '/' | '.'))
}

/// The shell step that lands the tree: stage, commit, push, print the hash.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn commit_command(subject: &str, body: Option<&str>, push_ref: &str) -> String {
    let subject = shell_quote(subject);
    body.map_or_else(
        || {
            super::work_item_steps::fill(
                "commit_command_without_body",
                &[("{subject}", &subject), ("{branch}", push_ref)],
            )
        },
        |body| {
            super::work_item_steps::fill(
                "commit_command",
                &[
                    ("{subject}", &subject),
                    ("{body}", &shell_quote(body)),
                    ("{branch}", push_ref),
                ],
            )
        },
    )
}

/// The commit body that names what the work resolves.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn resolves_body(reference: &str) -> String {
    super::work_item_steps::fill("body_resolves", &[("{reference}", reference)])
}

fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// Commit only the recipe's declared source artifacts; compiler outputs and
/// unrelated staged edits are not part of the requested change.
#[allow(clippy::literal_string_with_formatting_args, reason = "bind quoted operands in the shared commit template")]
pub(super) fn recipe_commit_command(recipe: &ExecutionRecipe, target: &CommitTarget) -> String {
    let files = std::iter::once(recipe.path.as_str())
        .chain(recipe.supporting_files.iter().map(|file| file.path.as_str()))
        .map(shell_quote).collect::<Vec<_>>().join(" ");
    super::work_item_steps::fill("recipe_commit_command", &[
        ("{files}", &files), ("{subject}", &shell_quote(&recipe_subject(recipe))),
        ("{body}", &shell_quote(&resolves_body(&target.reference))),
        ("{branch}", &shell_quote(target.push_ref())),
    ])
}

/// The commit subject for a recipe's artifact.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn recipe_subject(recipe: &ExecutionRecipe) -> String {
    super::work_item_steps::fill("subject_added", &[("{path}", &recipe.path)])
}

/// Answer a request to commit what is already in the working tree.
///
/// The request's own listing of the tree (`?? Main.scala`, ` M README.md`)
/// names the files, and the subject is derived from it; a request that lists
/// nothing gets a subject that says only what was done.
pub(super) fn plan_commit_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let normalized = crate::engine::normalize_prompt(task);
    if !seed::lexicon().mentions_role(seed::ROLE_GIT_COMMIT_REQUEST, &normalized) {
        return None;
    }
    // Only a request that is *about* committing is answered here. One that
    // also names the work to do -- "Implement X … and commit the changes", a
    // work item with its issue URL -- is the work, with the commit as its
    // last step, and belongs to the routes that do the work.
    if super::general_planner::repository_work_reference(task).is_some()
        || super::general_planner::mentions_software_authoring(task)
    {
        return None;
    }
    let run = tool_for(tool_names, Capability::Run)?;
    let progress = Progress::scan(messages);
    if let Some(output) = progress.run_outputs.last() {
        return Some(AgenticPlan::Final(super::tool_result::render(
            "git commit",
            output,
            task,
        )));
    }
    let command = commit_command(&subject_for_listing(task), None, "HEAD");
    Some(plan_one(run, json!({ "command": command }).to_string()))
}

/// A commit subject read from the `git status --porcelain` lines in `task`.
#[allow(clippy::literal_string_with_formatting_args)]
fn subject_for_listing(task: &str) -> String {
    let mut added = Vec::new();
    let mut changed = Vec::new();
    for line in task.lines() {
        let Some((status, path)) = porcelain_entry(line) else {
            continue;
        };
        if matches!(status, "??" | "A") {
            added.push(path);
        } else {
            changed.push(path);
        }
    }
    let fill = super::work_item_steps::fill;
    match (added.as_slice(), changed.as_slice()) {
        ([], []) => fill("subject_none", &[]),
        ([path], []) => fill("subject_added", &[("{path}", path)]),
        ([], [path]) => fill("subject_updated", &[("{path}", path)]),
        (added, changed) => {
            let mut all: Vec<&str> = added.iter().chain(changed.iter()).copied().collect();
            all.truncate(5);
            let count = (added.len() + changed.len()).to_string();
            fill(
                "subject_many",
                &[("{count}", &count), ("{paths}", &all.join(", "))],
            )
        }
    }
}

/// One `git status --porcelain` line: its status and its path.
fn porcelain_entry(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_end();
    let raw = line.trim_start();
    let (status, path) = raw.split_once(char::is_whitespace)?;
    let status = status.trim_matches(|character: char| character == '?' && status != "??");
    let known = matches!(status, "??" | "A" | "M" | "MM" | "AM" | "D" | "R" | "RM");
    let path = path.trim();
    (known && !path.is_empty() && !path.contains(char::is_whitespace)).then_some((status, path))
}
