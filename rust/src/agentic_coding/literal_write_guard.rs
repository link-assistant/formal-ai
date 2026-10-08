//! The literal-file backstop (PR #1188 T96).
//!
//! The guard behind G13, G17 and T29: the general change plan wrote its target
//! without reading it, so every new misreading of an edit request as a
//! whole-file write destroyed the file. Unless the request's first write verb
//! is a seeded whole-file write (`file_whole_write_action`: create, write,
//! save, new file, ... in every registered language), the target is read
//! first, and an existing non-empty file is replaced only when the request
//! consents to it (`file_overwrite_consent`); otherwise the plan declines with
//! a seeded refusal naming the file. The per-shape guards
//! (`names_an_addition`, the routed-write removal check) stay; this is the
//! backstop. Twin of `js/agentic/literal_write_guard.mjs`.

use super::general_planner::{GeneralChangePlan, GeneralPlanMode};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::write_request::{first_action_cue_end, first_action_cue_start, tokens};
use crate::protocol::ChatMessage;
use crate::seed;

const ROLE_WHOLE_WRITE: &str = "file_whole_write_action";
const ROLE_OVERWRITE_CONSENT: &str = "file_overwrite_consent";
const REFUSAL: &str = "general_change_existing_file_kept";

/// Whether the request's first write verb names a file's whole new content,
/// or the request consents to replacing it.
pub(super) fn writes_whole_file(request: &str) -> bool {
    let lexicon = seed::lexicon();
    if lexicon.mentions_role(
        ROLE_OVERWRITE_CONSENT,
        &crate::engine::normalize_prompt(request),
    ) {
        return true;
    }
    let toks = tokens(request);
    let (Some(start), Some(end)) = (first_action_cue_start(&toks), first_action_cue_end(&toks))
    else {
        return false;
    };
    request.get(start..end).is_some_and(|verb| {
        lexicon.mentions_role(ROLE_WHOLE_WRITE, &crate::engine::normalize_prompt(verb))
    })
}

/// The read of the target, or the refusal that keeps an existing file, ahead
/// of a literal-file plan's writes; `None` when the plan may proceed.
pub(super) fn guarded_step(
    plan: &GeneralChangePlan,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    if plan.mode != GeneralPlanMode::LiteralFile || writes_whole_file(&plan.goal) {
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
    let language = crate::language::detect(&plan.goal).slug();
    seed::render_response(REFUSAL, language, &values)
        .or_else(|| seed::render_response(REFUSAL, "en", &values))
        .map(AgenticPlan::Final)
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
