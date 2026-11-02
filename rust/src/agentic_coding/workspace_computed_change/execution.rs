//! Execution of source-derived computed workspace edits.
use super::ComputedChange;
use crate::agentic_coding::code_artifact::source_from_read_result;
use crate::agentic_coding::code_task::render_seeded_outcome;
use crate::agentic_coding::final_result::FinalResult;
use crate::agentic_coding::intent_router::edit_arguments;
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_for, write_arguments,
};
use crate::agentic_coding::workspace_change::{
    VerifiedChange, plan_digest_verification, plan_edit_failure, plan_edit_preimage_gap,
    plan_edit_read_verification, read_arguments, result_for_edit, result_for_path,
};
use crate::agentic_coding::workspace_line_operation::drops_most_of_file;
use crate::protocol::ChatMessage;

pub(in crate::agentic_coding) fn plan_computed_change_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    change: &ComputedChange,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let target = change.target.as_str();
    let Some(read) = result_for_path(current_turn, Capability::Read, target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(target)));
    };
    // A read that came back as the client's file block is the file, whatever
    // its text says: a ledger that quotes error lines is not a missing file.
    let missing = crate::agentic_coding::code_artifact::source_from_agent_read_result(&read)
        .is_none()
        && crate::agentic_coding::tool_result::failure_message(&read, false, true).is_some();
    let progress = crate::agentic_coding::progress::Progress::scan(current_turn);
    let preimage = progress.source_read_before_latest_mutation_for(target, Capability::Edit);
    if preimage.is_none() && progress.attempted_mutation_for(target, Capability::Edit) {
        return Some(plan_edit_preimage_gap(target, result));
    }
    let source = preimage
        .and_then(|observed| observed.source.clone())
        .unwrap_or_else(|| {
            if missing {
                String::new()
            } else {
                source_from_read_result(&read)
            }
        });

    if let Some(answer) = crate::agentic_coding::edit_scope::scoped_decline(task, target, &source) {
        return Some(AgenticPlan::Final(answer));
    }
    let Some(updated) = change
        .compute(&source, missing)
        .filter(|updated| *updated != source)
    else {
        // Text the change names that the file no longer holds is the answer (G87).
        let absent = if missing {
            None
        } else {
            change.absent_text_answer(task, &source)
        };
        return Some(AgenticPlan::Final(match absent {
            Some(answer) => answer,
            None => render_seeded_outcome("coding_workspace_verification_failed", task, target)?,
        }));
    };
    // A change that would drop most of the file is refused unless its request
    // states that extent (PR #1188 T29).
    if !change.bounded() && drops_most_of_file(&source, &updated) {
        return Some(AgenticPlan::Final(render_seeded_outcome(
            "coding_workspace_fragment_refused",
            task,
            target,
        )?));
    }
    let (intent, reported) = change.reported(&source);
    let slots: Vec<(&str, &str)> = reported
        .iter()
        .map(|(slot, value)| (*slot, value.as_str()))
        .collect();
    let values = change.reported_values(&source);
    let list_slots: Vec<(&str, &[String])> = values
        .as_deref()
        .map(|values| vec![(concat!("{", "old", "}"), values)])
        .unwrap_or_default();
    let verified = VerifiedChange {
        target,
        expected: &updated,
        intent,
        slots: &slots,
        list_slots: &list_slots,
    };
    if let Some((old, new)) = change.edit(&source, &updated)
        && let Some(tool) = tool_for(tool_names, Capability::Edit)
    {
        if result_for_edit(current_turn, target, &old, &new).is_none() {
            return Some(plan_one(tool, edit_arguments(target, &old, &new)));
        }
        if let Some(failed) = plan_edit_failure(
            task,
            current_turn,
            tool,
            &edit_arguments(target, &old, &new),
            result,
        ) {
            return Some(failed);
        }
        if let Some(observed) = plan_edit_read_verification(
            task,
            current_turn,
            tool_names,
            &verified,
            &edit_arguments(target, &old, &new),
            result,
        ) {
            return Some(observed);
        }
        return plan_digest_verification(task, current_turn, tool_names, &verified, result);
    }
    if result_for_path(current_turn, Capability::Write, target, Some(&updated)).is_none() {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(tool, write_arguments(target, &updated)));
    }
    plan_digest_verification(task, current_turn, tool_names, &verified, result)
}
