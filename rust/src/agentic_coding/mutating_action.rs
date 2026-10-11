//! Verified mutating filesystem actions (issues #824 and #944).
//!
//! Issue #824 reported a request to *move* a directory being refused outright.
//! The routing half of that was answered by the seed `mv` intent: the request
//! now lowers to a concrete `mv SOURCE DESTINATION`. What issue #944 asks for is
//! the other half — that the action be **verified** rather than merely issued.
//!
//! A read-only command answers by what it prints, so running it and reporting
//! its output is the whole job. A mutating command answers by what the workspace
//! *holds afterwards*, and a zero exit status is not that: `mv a b` exits zero
//! whether or not `b` was something the user wanted overwritten, and exits
//! non-zero for a missing parent directory that the request plainly implied
//! should exist. So the action is planned as an ordered recipe —
//!
//! 1. the preconditions the intent declares (the source is there, the
//!    destination is free),
//! 2. the preparation the intent declares (`mkdir -p` on the destination's
//!    parent, which is what makes a deep target path work at all),
//! 3. the action itself,
//! 4. the postconditions the intent declares (the destination is there, and for
//!    a move, the source is not),
//!
//! — with each step observed before the next is planned. A step that exits
//! non-zero ends the recipe and is reported as itself, so a blocked move says
//! which check blocked it and that nothing changed, instead of claiming a
//! completion the workspace would contradict.
//!
//! None of those predicates are written here. They are declared per intent in
//! `data/seed/shell-intents.lino` as [`crate::seed::ShellIntentEffect`], so a
//! maintainer teaches a new mutating verb its pre/post conditions by editing
//! seed data — the same rule every other trigger vocabulary on this project
//! follows, and the reason this module works for `cp` without naming `cp`.

use serde_json::json;

use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use super::tool_result;
use crate::protocol::ChatMessage;
use crate::seed::{self, ShellIntentVocabulary};

const PATH_PLACEHOLDER: &str = concat!("{", "path", "}");
const SOURCE_PLACEHOLDER: &str = concat!("{", "source", "}");
const SOURCES_PLACEHOLDER: &str = concat!("{", "sources", "}");
const SETUP_PLACEHOLDER: &str = concat!("{", "setup", "}");
const DESTINATION_PLACEHOLDER: &str = concat!("{", "destination", "}");
const DESTINATION_PARENT_PLACEHOLDER: &str = concat!("{", "destination_parent", "}");
const ACTION_PLACEHOLDER: &str = concat!("{", "action", "}");
const CHECK_PLACEHOLDER: &str = concat!("{", "check", "}");
const CHECKS_PLACEHOLDER: &str = concat!("{", "checks", "}");
const EXIT_CODE_PLACEHOLDER: &str = concat!("{", "exit_code", "}");

/// The directory component used when the destination names no directory at all.
const CURRENT_DIRECTORY: &str = ".";

/// One mutating action expanded into the ordered steps that carry it out.
pub(super) struct VerifiedAction {
    /// Every step in order: preconditions, preparation, the action, postconditions.
    steps: Vec<String>,
    /// The index of the action itself within [`Self::steps`].
    action: usize,
}

impl VerifiedAction {
    /// The steps in the order they are run.
    pub(super) fn steps(&self) -> &[String] {
        &self.steps
    }

    /// The mutating command itself, the one step that is not a check.
    pub(super) fn action(&self) -> &str {
        self.steps[self.action].as_str()
    }

    /// The checks that ran after the action, which is what the completion claim
    /// rests on.
    pub(super) fn postconditions(&self) -> &[String] {
        &self.steps[self.action + 1..]
    }
}

/// Expand `command` into its verified recipe, or `None` when no seed intent
/// declares an effect for it.
///
/// The command was assembled by [`super::shell_command`] as the intent's own
/// command followed by its operands, so it is taken apart the same way: the
/// longest declared command that prefixes it wins. Declared options are consumed
/// before positional paths are bound to scalar or collection effect templates.
pub(super) fn expand(command: &str) -> Option<VerifiedAction> {
    expand_with(command, &seed::shell_intent_vocabulary())
}

fn expand_with(command: &str, vocab: &ShellIntentVocabulary) -> Option<VerifiedAction> {
    let intent = vocab
        .intents
        .iter()
        .filter(|intent| intent.effect.is_declared())
        .filter(|intent| {
            command
                .strip_prefix(intent.command.as_str())
                .is_some_and(|rest| rest.starts_with(' '))
        })
        .max_by_key(|intent| intent.command.len())?;
    let effect = &intent.effect;
    let (operands, flags) = effect_operands(&command[intent.command.len()..], effect)?;
    if operands.is_empty() {
        return None;
    }
    let before = if flags.iter().any(|flag| effect.reuse_options.contains(flag))
        && !effect.before_reuse.is_empty()
    {
        &effect.before_reuse
    } else {
        &effect.before
    };
    let has_path = before
        .iter()
        .chain(&effect.prepare)
        .chain(&effect.after)
        .any(|template| template.contains(PATH_PLACEHOLDER));
    let mut groups: Vec<Vec<(&str, String)>> = Vec::new();
    let mut collection = None;
    if has_path {
        groups.extend(
            operands
                .iter()
                .map(|path| vec![(PATH_PLACEHOLDER, (*path).to_owned())]),
        );
    } else {
        if operands.len() < 2 || (operands.len() > 2 && !effect.directory_targets) {
            return None;
        }
        let destination = *operands.last()?;
        let sources = &operands[..operands.len() - 1];
        if !effect.directory_targets && sources.iter().any(|source| has_shell_expansion(source)) {
            return None;
        }
        if effect.directory_targets
            && (sources.len() > 1
                || sources.iter().any(|source| has_shell_expansion(source))
                || destination.trim_matches(['\'', '"']).ends_with('/'))
        {
            collection = Some((sources.join(" "), destination));
            groups.push(vec![
                (SOURCE_PLACEHOLDER, String::from("\"$formal_ai_source\"")),
                (
                    DESTINATION_PARENT_PLACEHOLDER,
                    String::from("\"$formal_ai_parent\""),
                ),
                (
                    DESTINATION_PLACEHOLDER,
                    String::from("\"$formal_ai_destination\""),
                ),
            ]);
        } else {
            groups.push(vec![
                (SOURCE_PLACEHOLDER, sources[0].to_owned()),
                (DESTINATION_PARENT_PLACEHOLDER, parent_of(destination)),
                (DESTINATION_PLACEHOLDER, destination.to_owned()),
            ]);
        }
    }
    let fill = |template: &String| -> Vec<String> {
        groups
            .iter()
            .map(|bindings| {
                let check = bind_template(template, bindings);
                if let Some((sources, destination)) = &collection {
                    collection_check(sources, destination, &check)
                } else {
                    check
                }
            })
            .collect()
    };
    let mut steps: Vec<String> = before
        .iter()
        .chain(&effect.prepare)
        .flat_map(fill)
        .collect();
    let action = steps.len();
    steps.push(command.to_owned());
    steps.extend(effect.after.iter().flat_map(fill));
    Some(VerifiedAction { steps, action })
}

fn collection_check(sources: &str, destination: &str, check: &str) -> String {
    let setup = super::work_item_steps::fill(
        "filesystem-collection-setup",
        &[(DESTINATION_PLACEHOLDER, destination)],
    );
    super::work_item_steps::fill(
        "filesystem-collection-check",
        &[
            (SOURCES_PLACEHOLDER, sources),
            (SETUP_PLACEHOLDER, &setup),
            (CHECK_PLACEHOLDER, check),
        ],
    )
}

fn bind_template(template: &str, bindings: &[(&str, String)]) -> String {
    bindings
        .iter()
        .fold(template.to_owned(), |text, (key, value)| {
            text.replace(key, value)
        })
}

/// Separate declared options from shell words while preserving operand quoting.
fn effect_operands<'a>(
    text: &'a str,
    effect: &seed::ShellIntentEffect,
) -> Option<(Vec<&'a str>, Vec<String>)> {
    let words = shell_words(text)?;
    let mut operands = Vec::new();
    let mut flags = Vec::new();
    let mut options = true;
    let mut index = 0;
    while index < words.len() {
        let raw = words[index];
        let word = raw.trim_matches(['\'', '"']);
        if options && word == "--" {
            options = false;
        } else if options && word.starts_with('-') && word != "-" {
            if let Some(option) = effect.value_options.iter().find(|option| {
                word == option.as_str()
                    || word.strip_prefix(option.as_str()).is_some_and(|suffix| {
                        suffix.starts_with('=') || (!option.starts_with("--") && !suffix.is_empty())
                    })
            }) {
                if word == option.as_str() {
                    index += 1;
                    if index >= words.len() {
                        return None;
                    }
                }
                flags.push(option.clone());
            } else if effect.flags.iter().any(|flag| flag == word) {
                flags.push(word.to_owned());
            } else if !word.starts_with("--") {
                let bundled: Vec<String> = word[1..]
                    .chars()
                    .map(|letter| format!("-{letter}"))
                    .collect();
                if !bundled.iter().all(|flag| effect.flags.contains(flag)) {
                    return None;
                }
                flags.extend(bundled);
            } else {
                return None;
            }
        } else {
            operands.push(raw);
        }
        index += 1;
    }
    Some((operands, flags))
}

fn shell_words(text: &str) -> Option<Vec<&str>> {
    let mut words = Vec::new();
    let mut start = None;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in text.char_indices() {
        if start.is_none() && character.is_whitespace() {
            continue;
        }
        if start.is_none() {
            start = Some(index);
        }
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }
        if character.is_whitespace() {
            words.push(&text[start.take()?..index]);
        } else if ";|&<>()`".contains(character) {
            return None;
        }
    }
    if quote.is_some() || escaped {
        return None;
    }
    if let Some(start) = start {
        words.push(&text[start..]);
    }
    Some(words)
}

fn has_shell_expansion(word: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for character in word.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
        } else if "*?[$".contains(character) {
            return true;
        }
    }
    false
}

/// The directory component of `path`, or `.` when it names no directory.
///
/// Written as text rather than through [`std::path::Path`] on purpose: the
/// operand is a shell word that may still be rooted at `~`, and `Path` would
/// happily hand back a `~` that the shell then declines to expand once it is no
/// longer the first character of the word.
fn parent_of(path: &str) -> String {
    if path.contains(['"', '\'', '\\']) {
        return format!("\"$(dirname -- {path})\"");
    }
    match path.rsplit_once('/') {
        Some(("", _)) => String::from("/"),
        Some((parent, _)) => parent.to_owned(),
        None => String::from(CURRENT_DIRECTORY),
    }
}

/// Plan the next step of the verified recipe for `command`.
///
/// Returns `None` when the command declares no effect (every read-only intent),
/// or when the client advertised no shell tool — in which case the caller's
/// single-shot path still produces the honest "I can run this when you give me a
/// shell" answer rather than this module inventing a second one.
pub(super) fn plan_step(
    command: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    prompt: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let recipe = expand(command)?;
    let tool = tool_for(tool_names, Capability::Run)?;
    let progress = Progress::scan(messages);
    let taken = progress.run_outputs.len();

    // The workspace gets the last word before anything else is planned: a step
    // that exited non-zero ends the recipe where it stopped.
    if let Some(index) = taken.checked_sub(1) {
        let observed = &progress.run_outputs[index];
        if tool_result::step_outcome(observed) == tool_result::StepOutcome::Failed {
            return Some(record(
                AgenticPlan::Final(blocked_report(
                    &recipe,
                    recipe.steps().get(index).map_or(command, String::as_str),
                    observed,
                    prompt,
                    index >= recipe.action,
                )),
                FinalDisposition::Failure,
                "mutating_action_failed",
                result,
            ));
        }
    }
    if let Some(step) = recipe.steps().get(taken) {
        return Some(plan_one(tool, json!({ "command": step }).to_string()));
    }
    Some(record(
        AgenticPlan::Final(completed_report(&recipe, prompt)),
        FinalDisposition::Finding,
        "mutating_action_verified",
        result,
    ))
}

/// The report for a recipe that stopped: which check stopped it, with what
/// status, and the fact that the action did not run.
///
/// A blocked move is not a failed move, and saying so is the point. Issue #824
/// is the record of what over-refusal costs and issue #916 rung `R916-01` is the
/// record of what a false completion claim costs; a recipe that names the check
/// it stopped on avoids both.
fn blocked_report(
    recipe: &VerifiedAction,
    check: &str,
    observed: &str,
    prompt: &str,
    attempted: bool,
) -> String {
    let language = tool_result::response_language(prompt);
    let intent = if attempted {
        "mutating-action-observed-failure"
    } else {
        "mutating_action_blocked"
    };
    let Some(mut answer) = seed::localized_response(intent, language) else {
        return tool_result::render(check, observed, prompt);
    };
    answer = answer.replace(ACTION_PLACEHOLDER, recipe.action());
    answer = answer.replace(CHECK_PLACEHOLDER, check);
    answer = answer.replace(
        EXIT_CODE_PLACEHOLDER,
        &tool_result::reported_exit_code(observed)
            .map_or_else(String::new, |code| code.to_string()),
    );
    answer
}

/// The report for a recipe that finished: the action that ran, and the checks
/// that were observed to hold afterwards.
///
/// The claim rests on those checks and names them, because a claim the reader
/// cannot re-run is the narration issue #916 rung `R916-01` was written against.
fn completed_report(recipe: &VerifiedAction, prompt: &str) -> String {
    let language = tool_result::response_language(prompt);
    let checks = recipe
        .postconditions()
        .iter()
        .map(|check| format!("`{check}`"))
        .collect::<Vec<_>>()
        .join(", ");
    seed::localized_response("mutating_action_completed", language).map_or_else(
        || recipe.action().to_owned(),
        |answer| {
            answer
                .replace(ACTION_PLACEHOLDER, recipe.action())
                .replace(CHECKS_PLACEHOLDER, &checks)
        },
    )
}

/// The ordered steps `command` is carried out as.
///
/// `None` when no seed intent declares an effect for it — every read-only
/// intent, and every mutating one whose operands are not the source/destination
/// pair the effect is written against.
///
/// Read-only intents keep their single-shot path, which is why this is the same
/// question the planner asks: a caller that wants to know whether a command is
/// carried out as a recipe asks for the recipe.
#[must_use]
pub fn verified_recipe(command: &str) -> Option<Vec<String>> {
    expand(command).map(|recipe| recipe.steps)
}
