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

use super::planner::{plan_one, tool_for, AgenticPlan, Capability};
use super::progress::Progress;
use super::tool_result;
use crate::protocol::ChatMessage;
use crate::seed::{self, ShellIntentVocabulary};

const SOURCE_PLACEHOLDER: &str = concat!("{", "source", "}");
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
/// longest declared command that prefixes it wins, and the remainder must be
/// exactly the two operands a source/destination effect is written against.
pub(super) fn expand(command: &str) -> Option<VerifiedAction> {
    expand_with(command, &seed::shell_intent_vocabulary())
}

fn expand_with(command: &str, vocab: &ShellIntentVocabulary) -> Option<VerifiedAction> {
    let (effect, operands) = vocab
        .intents
        .iter()
        .filter(|intent| intent.effect.is_declared())
        .filter_map(|intent| {
            let rest = command
                .strip_prefix(intent.command.as_str())
                .filter(|rest| rest.starts_with(' '))?;
            Some((&intent.effect, rest.split_whitespace().collect::<Vec<_>>()))
        })
        .max_by_key(|(_, operands)| operands.len())?;
    let [source, destination] = operands.as_slice() else {
        return None;
    };
    let fill = |template: &String| {
        template
            .replace(SOURCE_PLACEHOLDER, source)
            .replace(DESTINATION_PARENT_PLACEHOLDER, &parent_of(destination))
            .replace(DESTINATION_PLACEHOLDER, destination)
    };
    let mut steps: Vec<String> = effect.before.iter().map(&fill).collect();
    steps.extend(effect.prepare.iter().map(&fill));
    let action = steps.len();
    steps.push(command.to_owned());
    steps.extend(effect.after.iter().map(&fill));
    Some(VerifiedAction { steps, action })
}

/// The directory component of `path`, or `.` when it names no directory.
///
/// Written as text rather than through [`std::path::Path`] on purpose: the
/// operand is a shell word that may still be rooted at `~`, and `Path` would
/// happily hand back a `~` that the shell then declines to expand once it is no
/// longer the first character of the word.
fn parent_of(path: &str) -> String {
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
            return Some(AgenticPlan::Final(blocked_report(
                &recipe,
                recipe.steps().get(index).map_or(command, String::as_str),
                observed,
                prompt,
            )));
        }
    }
    if let Some(step) = recipe.steps().get(taken) {
        return Some(plan_one(tool, json!({ "command": step }).to_string()));
    }
    Some(AgenticPlan::Final(completed_report(&recipe, prompt)))
}

/// The report for a recipe that stopped: which check stopped it, with what
/// status, and the fact that the action did not run.
///
/// A blocked move is not a failed move, and saying so is the point. Issue #824
/// is the record of what over-refusal costs and issue #916 rung `R916-01` is the
/// record of what a false completion claim costs; a recipe that names the check
/// it stopped on avoids both.
fn blocked_report(recipe: &VerifiedAction, check: &str, observed: &str, prompt: &str) -> String {
    let language = tool_result::response_language(prompt);
    let Some(mut answer) = seed::localized_response("mutating_action_blocked", language) else {
        return tool_result::render(check, observed, prompt);
    };
    answer = answer.replace(ACTION_PLACEHOLDER, recipe.action());
    answer = answer.replace(CHECK_PLACEHOLDER, check);
    answer = answer.replace(
        EXIT_CODE_PLACEHOLDER,
        &tool_result::reported_exit_code(observed).map_or_else(String::new, |code| code.to_string()),
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

