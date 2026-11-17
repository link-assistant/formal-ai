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

use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::engine::ExecutionRecipe;
use crate::protocol::ChatMessage;
use crate::seed;

/// Where a work item's result is to land.
pub(super) struct CommitTarget {
    binding_refused: bool,
    /// The branch the request named, when it named one; otherwise the branch
    /// that is checked out (`HEAD`).
    pub(super) branch: Option<String>,
    /// The issue or pull request the work item resolves.
    pub(super) reference: String,
}

/// Classify the already validated work-item destination for all recipe consumers.
#[must_use]
pub(super) fn is_pull_request_target(target: &CommitTarget) -> bool {
    is_pull_request_reference(&target.reference)
}

/// Whether a work-item reference names a pull request. Every reference
/// `general_planner::repository_work_reference` yields is a GitHub `issues` or
/// `pull` URL, so any other reference names an issue.
pub(super) fn is_pull_request_reference(reference: &str) -> bool {
    reference.contains("/pull/")
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
    let binding = request_target_binding(request, "branch-binding-patterns", valid_branch_target);
    let binding_refused = matches!(binding, RequestTargetBinding::Refused);
    let branch = named_branch(request);
    Some(CommitTarget {
        binding_refused,
        branch,
        reference,
    })
}

/// The branch named right after a branch cue: "on branch issue-1-abc",
/// "в ветке issue-1-abc", "分支 issue-1-abc".
pub(super) fn named_branch(request: &str) -> Option<String> {
    match request_target_binding(request, "branch-binding-patterns", valid_branch_target) {
        RequestTargetBinding::Bound(value) => Some(value),
        _ => None,
    }
}

pub(super) enum RequestTargetBinding {
    Absent,
    Bound(String),
    Refused,
}

fn target_policy_values(key: &str) -> Vec<String> {
    serde_json::from_str(&super::work_item_steps::fill(key, &[]))
        .expect("valid typed target policy")
}

pub(super) fn valid_branch_target(value: &str) -> bool {
    !value.is_empty()
        && value != "@"
        && !value.starts_with('-')
        && !value.ends_with('.')
        && !target_policy_values("branch-forbidden-substrings")
            .iter()
            .any(|part| value.contains(part.as_str()))
        && !value
            .chars()
            .any(|ch| ch <= ' ' || ch == '\u{7f}' || "~^:?*[\\".contains(ch))
        && value.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && std::path::Path::new(part).extension() != Some(std::ffi::OsStr::new("lock"))
        })
}

pub(super) fn valid_workflow_target(value: &str) -> bool {
    let lower = value.to_lowercase();
    let file = std::path::Path::new(&lower);
    (file.extension().is_some_and(|extension| {
        extension == std::ffi::OsStr::new("yml") || extension == std::ffi::OsStr::new("yaml")
    }) || file.file_name() == Some(std::ffi::OsStr::new(".yml"))
        || file.file_name() == Some(std::ffi::OsStr::new(".yaml")))
        && !value.starts_with('/')
        && !value
            .chars()
            .any(|ch| ch <= ' ' || ch == '\u{7f}' || "\\:;|&$<>\x60\"'".contains(ch))
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// A workflow file name with no directory (issue #1162).
pub(super) fn valid_workflow_file_name(value: &str) -> bool {
    !value.contains('/') && valid_workflow_target(value)
}

pub(super) fn request_target_binding(
    request: &str,
    key: &str,
    validate: fn(&str) -> bool,
) -> RequestTargetBinding {
    let encoded = super::work_item_steps::fill(key, &[]);
    let patterns: Vec<String> =
        serde_json::from_str(&encoded).expect("valid target binding policy");
    let excluded_key = key.replace("-binding-patterns", "-excluded-patterns");
    let excluded_encoded = super::work_item_steps::fill(&excluded_key, &[]);
    let excluded_patterns: Vec<String> =
        serde_json::from_str(&excluded_encoded).expect("valid excluded target policy");
    let mut values = std::collections::BTreeSet::new();
    let mut excluded = std::collections::BTreeSet::new();
    let mut refused = false;
    let mut fenced = false;
    let mut offset = 0;
    let quoted = crate::normal_markov::quoted_segment_spans(request);
    for raw in request.split_inclusive('\n') {
        let line_offset = offset;
        offset += raw.len();
        let mut line = raw.trim();
        if target_policy_values("owned-clause-fence-prefixes")
            .iter()
            .any(|prefix| line.starts_with(prefix.as_str()))
        {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if ["- ", "* ", "+ "]
            .iter()
            .any(|prefix| line.starts_with(prefix))
        {
            line = line[2..].trim();
        } else if let Some((lead, tail)) = line.split_once(' ')
            && lead
                .trim_end_matches(['.', ')'])
                .chars()
                .all(|ch| ch.is_ascii_digit())
            && lead.ends_with(['.', ')'])
        {
            line = tail.trim();
        }
        let owned_at = line_offset + raw.find(line).unwrap_or(0);
        if quoted
            .iter()
            .any(|segment| segment.start <= owned_at && owned_at < segment.end)
        {
            continue;
        }
        for (is_excluded, pattern) in patterns
            .iter()
            .map(|value| (false, value))
            .chain(excluded_patterns.iter().map(|value| (true, value)))
        {
            let regex = regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .expect("valid target binding pattern");
            let Some(matched) = regex.captures(line) else {
                continue;
            };
            let mut value = matched[1].trim();
            value = value.strip_suffix(['.', '。', '।']).unwrap_or(value);
            value = value.strip_suffix(',').unwrap_or(value);
            let literal = crate::normal_markov::quoted_segment_spans(value)
                .into_iter()
                .find(|segment| segment.start == 0 && segment.end == value.len())
                .map(|segment| segment.text);
            if let Some(text) = literal.as_deref() {
                value = text;
            }
            if validate(value) {
                if is_excluded {
                    excluded.insert(value.to_owned());
                } else {
                    values.insert(value.to_owned());
                }
            } else {
                refused = true;
            }
        }
    }
    if refused
        || values.len() > 1
        || (values.is_empty() && !excluded.is_empty())
        || values.iter().any(|value| excluded.contains(value))
    {
        RequestTargetBinding::Refused
    } else {
        values
            .into_iter()
            .next()
            .map_or(RequestTargetBinding::Absent, RequestTargetBinding::Bound)
    }
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

pub(super) use crate::portable_string_contracts::quote_shell_word as shell_quote;

/// Commit declared artifacts and a verified task-owned scaffold deletion.
/// Compiler outputs and unrelated staged edits remain outside this commit.
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind quoted operands in the shared commit template"
)]
pub(super) fn recipe_commit_command(recipe: &ExecutionRecipe, target: &CommitTarget) -> String {
    if target.binding_refused {
        return super::work_item_steps::fill("target_binding_refused_command", &[]);
    }
    let files = std::iter::once(recipe.path.as_str())
        .chain(
            recipe
                .supporting_files
                .iter()
                .map(|file| file.path.as_str()),
        )
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ");
    let command = super::work_item_steps::fill(
        "recipe_commit_command",
        &[
            ("{files}", "\"$@\""),
            ("{subject}", &shell_quote(&recipe_subject(recipe))),
            ("{body}", &shell_quote(&resolves_body(&target.reference))),
            ("{branch}", &shell_quote(target.push_ref())),
        ],
    );
    super::work_item_steps::fill(
        "task-scaffold-commit-command",
        &[
            ("{files}", &files),
            ("{reference}", &shell_quote(&target.reference)),
            ("{retire}", &superseded_workflow_step(recipe)),
            ("{commit}", &command),
        ],
    )
}

/// The step that retires this generator's earlier workflow for the same
/// program, or nothing when the recipe carries no workflow (issue #1162).
///
/// A resumed Hive Mind session regenerated the workflow at
/// `.github/workflows/Main.yml` beside the `run.yml` an earlier session had
/// committed, so the pull request ran the program twice. The retired file must
/// sit beside the recipe's workflow and carry the template's exact header and
/// name lines for this program; anything a person wrote or edited stays.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn superseded_workflow_step(recipe: &ExecutionRecipe) -> String {
    let template = super::work_item_steps::fill("workflow_template", &[("{path}", &recipe.path)]);
    let header = template.lines().next().unwrap_or_default();
    let Some(name) = template
        .lines()
        .find(|line| line.strip_prefix("name: ").is_some()) else {
        return String::new();
    };
    let Some(workflow) = recipe
        .supporting_files
        .iter()
        .find(|file| file.source.lines().any(|line| line == name))
    else {
        return String::new();
    };
    let (dir, file) = workflow
        .path
        .rsplit_once('/')
        .unwrap_or((".", workflow.path.as_str()));
    super::work_item_steps::fill(
        "superseded-workflow-retire-command",
        &[
            ("{dir}", &shell_quote(dir)),
            ("{workflow}", &shell_quote(&format!("{dir}/{file}"))),
            ("{header}", &shell_quote(header)),
            ("{name}", &shell_quote(name)),
        ],
    )
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
    // The commit cue must be the request's own words: `Append "Commit all
    // changes" to notes.txt.` quotes it as text to write (PR #1188 gap 2).
    let normalized = crate::engine::normalize_prompt(
        &crate::solver_handlers::text_outside_quoted_segments(task),
    );
    if !seed::lexicon().mentions_role(seed::ROLE_GIT_COMMIT_REQUEST, &normalized) {
        return None;
    }
    // Only a request that is *about* committing is answered here. One that
    // also names the work to do -- "Implement X … and commit the changes", a
    // work item with its issue URL -- is the work, with the commit as its
    // last step, and belongs to the routes that do the work.
    // Commit phrases can contain a generic authoring verb. Mask their complete
    // spans while preserving an independent authoring instruction.
    let spans: Vec<_> = seed::lexicon()
        .words_for_role(seed::ROLE_GIT_COMMIT_REQUEST)
        .iter()
        .flat_map(|cue| {
            normalized
                .match_indices(cue)
                .filter_map(|(start, matched)| {
                    let end = start + matched.len();
                    let before = normalized[..start].chars().next_back();
                    let after = normalized[end..].chars().next();
                    (before.is_none_or(|ch| !ch.is_alphanumeric())
                        && after.is_none_or(|ch| !ch.is_alphanumeric()))
                    .then_some((start, end))
                })
        })
        .collect();
    let authoring_text: String = normalized
        .char_indices()
        .map(|(index, ch)| {
            if spans
                .iter()
                .any(|(start, end)| index >= *start && index < *end)
            {
                ' '
            } else {
                ch
            }
        })
        .collect();
    if super::general_planner::repository_work_reference(task).is_some()
        || super::general_planner::mentions_software_authoring(&authoring_text)
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
    let subject = stated_subject(task).unwrap_or_else(|| subject_for_listing(task));
    let command = commit_command(&subject, None, "HEAD");
    Some(plan_one(run, json!({ "command": command }).to_string()))
}

/// The message the request quotes for the commit: the first quoted text after
/// a seeded `git_commit_message_lead` ("with the message 'initial notes'"),
/// so a stated message is never replaced by a generated one (PR #1188
/// dogfooding).
fn stated_subject(task: &str) -> Option<String> {
    let lowered = task.to_lowercase();
    let from = seed::lexicon()
        .words_for_role("git_commit_message_lead")
        .iter()
        .filter_map(|word| {
            let word = word.to_lowercase();
            lowered.find(&word).map(|at| at + word.len())
        })
        .min()?;
    crate::normal_markov::quoted_segment_spans(task)
        .into_iter()
        .find(|segment| segment.start >= from && !segment.text.trim().is_empty())
        .map(|segment| segment.text)
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

/// Feedback read, description edit and readiness for a pull request target.
/// `requirements` is the reference the request was for: when it is an issue
/// the description closes that issue and the title is the issue's title
/// (issue #1162: the delivered pull requests kept the orchestrator's
/// placeholder title and description, and the body named the pull request
/// itself rather than the issue it resolves).
pub(super) fn pr_completion_commands(
    recipe: &ExecutionRecipe,
    target: &CommitTarget,
    requirements: &str,
) -> Vec<String> {
    if !is_pull_request_target(target) {
        return Vec::new();
    }
    let target_url = shell_quote(&target.reference);
    let issue = (!is_pull_request_reference(requirements)).then_some(requirements);
    let commands = recipe
        .commands
        .iter()
        .map(|command| format!("- `{command}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let body = super::work_item_steps::fill(
        "pr_body_template",
        &[
            ("{path}", &recipe.path),
            (concat!("{", "commands}"), &commands),
            ("{reference}", issue.unwrap_or(&target.reference)),
        ],
    );
    let quoted_body = shell_quote(&body);
    let edit = match issue {
        Some(issue) => super::work_item_steps::fill(
            "pull-request-edit-titled-command",
            &[
                (concat!("{", "target}"), &target_url),
                (
                    concat!("{", "issue}"),
                    &shell_quote(issue),
                ),
                (concat!("{", "body}"), &quoted_body),
            ],
        ),
        None => super::work_item_steps::fill(
            "pr_edit_command",
            &[
                (concat!("{", "target}"), &target_url),
                (concat!("{", "body}"), &quoted_body),
            ],
        ),
    };
    vec![
        super::work_item_steps::fill(
            "pr_comments_command",
            &[
                (concat!("{", "target}"), &target_url),
                (
                    concat!("{", "filter}"),
                    &shell_quote(&super::restart_feedback::pull_request_feedback_filter()),
                ),
            ],
        ),
        edit,
        super::work_item_steps::fill(
            "pr_ready_command",
            &[(concat!("{", "target}"), &target_url)],
        ),
    ]
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/git_commit_target_binding.rs"]
mod target_binding_controls;

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/git_commit_extension_identity.rs"]
mod extension_identity_tests;

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/issue_1162_delivery_commands.rs"]
mod issue_1162_delivery_commands;
