//! The literal-file backstop (PR #1188 T96).
//!
//! The guard behind G13, G17 and T29: the general change plan wrote its target
//! without reading it, so every new misreading of an edit request as a
//! whole-file write destroyed the file. Unless the request's first write verb
//! is a seeded whole-file write (`file_whole_write_action`: create, write,
//! save, new file, ... in every registered language), the target is read
//! first, and an existing non-empty file is replaced only when the request
//! consents to it (`file_overwrite_consent`); otherwise the plan declines with
//! a seeded refusal naming the file. A request that declares the file it
//! writes (a seeded `file_declared_noun` right before the path, no destination
//! ahead of it, the content stated after it: `add file note.txt containing
//! hello`) creates that file, whatever its verb. The per-shape guards
//! (`names_an_addition`, the routed-write removal check) stay; this is the
//! backstop. Twin of `js/agentic/literal_write_guard.mjs`.

use super::general_planner::{GeneralChangePlan, GeneralPlanMode};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::write_request::{
    Token, bare_surfaces, clean_cue_token, clean_path_token, first_action_cue_end,
    first_action_cue_start, first_content_lead_end, looks_like_file_path, tokens,
};
use crate::protocol::ChatMessage;
use crate::seed;

const ROLE_WHOLE_WRITE: &str = "file_whole_write_action";
const ROLE_OVERWRITE_CONSENT: &str = "file_overwrite_consent";
const ROLE_DECLARED_NOUN: &str = "file_declared_noun";
const REFUSAL: &str = "general_change_existing_file_kept";
const UNNAMED: &str = "general_change_target_unnamed";

/// Whether the request's first write verb names a file's whole new content,
/// or the request consents to replacing it.
pub(super) fn writes_whole_file(request: &str) -> bool {
    let instruction = instruction_outside_quotes(request);
    let lexicon = seed::lexicon();
    if lexicon.mentions_role(
        ROLE_OVERWRITE_CONSENT,
        &crate::engine::normalize_prompt(&instruction),
    ) {
        return true;
    }
    if declares_file(
        &instruction,
        &super::positional_edit::unquoted_path_tokens(request),
    ) {
        return true;
    }
    let toks = tokens(&instruction);
    let (Some(start), Some(end)) = (first_action_cue_start(&toks), first_action_cue_end(&toks))
    else {
        return false;
    };
    instruction.get(start..end).is_some_and(|verb| {
        lexicon.mentions_role(ROLE_WHOLE_WRITE, &crate::engine::normalize_prompt(verb))
    })
}

fn instruction_outside_quotes(request: &str) -> String {
    let mut instruction = request.to_owned();
    for span in crate::normal_markov::quoted_segment_spans(request)
        .into_iter()
        .rev()
    {
        instruction.replace_range(span.start..span.end, &" ".repeat(span.end - span.start));
    }
    instruction
}

/// The requested edit action itself must own a whole-file operation.
pub(super) fn whole_file_operation_at(
    request: &str,
    action_start: usize,
    target: &str,
    target_end: usize,
) -> bool {
    let instruction = instruction_outside_quotes(request);
    let action_tokens = tokens(&instruction);
    if first_action_cue_start(&action_tokens).is_some_and(|first| first < action_start) {
        return false;
    }
    let Some(action) = action_tokens
        .iter()
        .find(|token| token.start == action_start)
    else {
        return false;
    };
    if declares_file(
        &instruction,
        &super::positional_edit::unquoted_path_tokens(request),
    ) {
        return true;
    }
    if seed::lexicon().mentions_role(
        ROLE_WHOLE_WRITE,
        &crate::engine::normalize_prompt(action.text),
    ) {
        return true;
    }
    let suffix: Vec<_> = super::positional_edit::unquoted_path_tokens(request)
        .into_iter()
        .filter(|token| token.start >= action_start)
        .collect();
    bare_surfaces(ROLE_OVERWRITE_CONSENT).iter().any(|cue| {
        let prefix = tokens(cue);
        if prefix.is_empty()
            || prefix.len() > suffix.len()
            || !prefix
                .iter()
                .zip(&suffix)
                .all(|(token, actual)| clean_cue_token(token.text) == clean_cue_token(actual.text))
        {
            return false;
        }
        let mut destination = prefix.len();
        let word = suffix
            .get(destination)
            .map(|token| clean_cue_token(token.text))
            .unwrap_or_default();
        if bare_surfaces("request_function_word").contains(&word)
            && !bare_surfaces("file_edit_new_lead_cue").contains(&word)
        {
            destination += 1;
        }
        if suffix.get(destination).is_some_and(|token| {
            bare_surfaces(ROLE_DECLARED_NOUN).contains(&clean_cue_token(token.text))
        }) {
            destination += 1;
        }
        suffix
            .get(destination)
            .is_some_and(|token| token.end == target_end && clean_path_token(token.text) == target)
    })
}

/// Whether the request declares the file it writes.
///
/// A path right after a seeded `file_declared_noun`, no destination or
/// location cue ahead of that noun (`add 'x' to the file a.txt` edits it), and
/// a content lead after the path (`new file: notes.txt, contents: hello`).
fn declares_file(request: &str, toks: &[Token<'_>]) -> bool {
    let nouns = bare_surfaces(ROLE_DECLARED_NOUN);
    let mut cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    cues.extend(bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE));
    toks.iter().enumerate().skip(1).any(|(index, token)| {
        looks_like_file_path(clean_path_token(token.text))
            && nouns.contains(&clean_cue_token(toks[index - 1].text))
            && !toks[..index - 1]
                .iter()
                .any(|before| cues.contains(&clean_cue_token(before.text)))
            && request
                .get(token.end..)
                .is_some_and(|after| first_content_lead_end(&after.to_lowercase()).is_some())
    })
}

/// The read of the target, or the refusal that keeps an existing file, ahead
/// of a literal-file plan's writes; `None` when the plan may proceed.
pub(super) fn guarded_step(
    plan: &GeneralChangePlan,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    if plan.mode != GeneralPlanMode::LiteralFile {
        return None;
    }
    let language = crate::language::detect(&plan.goal).slug();
    // A write never targets a path the request names only inside quoted text
    // (PR #1188 G86).
    if !names_as_destination(&plan.goal, &plan.target) {
        let values = [("path", plan.target.as_str())];
        return seed::render_response(UNNAMED, language, &values)
            .or_else(|| seed::render_response(UNNAMED, "en", &values))
            .map(AgenticPlan::Final);
    }
    if writes_whole_file(&plan.goal) {
        return None;
    }
    let read = tool_for(tool_names, Capability::Read)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let Some(source) = super::module_function::read_source(current_turn, &plan.target) else {
        return Some(plan_one(
            read,
            super::workspace_change::read_arguments(&plan.target),
        ));
    };
    if source.trim().is_empty() || source == plan.content {
        return None;
    }
    let values = [("path", plan.target.as_str())];
    seed::render_response(REFUSAL, language, &values)
        .or_else(|| seed::render_response(REFUSAL, "en", &values))
        .map(AgenticPlan::Final)
}

/// Whether `request` names `target` outside its quoted text.
///
/// Or, naming no path outside it, quotes `target` whole (mirrors
/// `namesAsDestination`).
fn names_as_destination(request: &str, target: &str) -> bool {
    let segments = crate::normal_markov::quoted_segment_spans(request);
    let unquoted: Vec<&str> = tokens(request)
        .iter()
        .filter(|token| {
            !segments
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
        })
        .map(|token| clean_path_token(token.text))
        .filter(|path| looks_like_file_path(path))
        .collect();
    unquoted.contains(&target)
        || (unquoted.is_empty() && segments.iter().any(|segment| segment.text.trim() == target))
}

/// Whether a failed call is the guard's read of a target that does not exist
/// yet, which the plan goes on to create.
pub(super) fn is_guard_read(
    plan: &GeneralChangePlan,
    failed_path: Option<&str>,
    capability: Capability,
) -> bool {
    plan.mode == GeneralPlanMode::LiteralFile
        && capability == Capability::Read
        && failed_path == Some(plan.target.as_str())
        && !writes_whole_file(&plan.goal)
}
