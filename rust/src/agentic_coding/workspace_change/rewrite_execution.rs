//! Execute source-owned rewrites from a qualified preimage and independently verify their effects.
use super::{
    GroundedRewrite, VerifiedChange, anchored_lines, as_written, changed_lines_edit,
    plan_digest_verification, plan_edit_failure, plan_edit_preimage_gap,
    plan_edit_read_verification, plan_write_digest_verification, read_arguments,
    repeated_identifier_rewrite_command, result_for_command, result_for_edit, result_for_path,
    rewritten_source,
};
use crate::agentic_coding::code_artifact::source_from_read_result;
use crate::agentic_coding::code_task::render_seeded_outcome;
use crate::agentic_coding::final_result::FinalResult;
use crate::agentic_coding::intent_router::edit_arguments;
use crate::agentic_coding::planner::{
    AgenticPlan, Capability, plan_one, tool_for, write_arguments,
};
use crate::protocol::ChatMessage;
use crate::workspace_change_learning::{RewriteScope, word_scoped_matches};
use serde_json::json;

pub(super) fn plan_rewrite_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    rewrite: &GroundedRewrite,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let Some(read) = result_for_path(current_turn, Capability::Read, &rewrite.target, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&rewrite.target)));
    };
    let progress = crate::agentic_coding::progress::Progress::scan(current_turn);
    let preimage =
        progress.source_read_before_latest_mutation_for(&rewrite.target, Capability::Edit);
    if preimage.is_none() && progress.attempted_mutation_for(&rewrite.target, Capability::Edit) {
        return Some(plan_edit_preimage_gap(&rewrite.target, result));
    }
    let source = preimage
        .and_then(|observed| observed.source.clone())
        .unwrap_or_else(|| source_from_read_result(&read));
    let written = as_written(&source, rewrite);
    let rewrite = written.as_ref().unwrap_or(rewrite);
    let Some(updated) = rewritten_source(&source, rewrite) else {
        // A replace asked again finds its old text gone: that is the answer (G87).
        let absent = crate::agentic_coding::replace_list::absent_text_answer(
            task,
            &rewrite.target,
            &source,
            &rewrite.pattern,
            Some(&rewrite.replacement),
        );
        return Some(AgenticPlan::Final(match absent {
            Some(answer) => answer,
            None => render_seeded_outcome(
                "coding_workspace_verification_failed",
                task,
                &rewrite.target,
            )?,
        }));
    };

    let occurrences = match rewrite.scope {
        RewriteScope::Substring => source.match_indices(&rewrite.pattern).count(),
        RewriteScope::Word => word_scoped_matches(&source, &rewrite.pattern).len(),
    };
    if occurrences > 1
        && let Some(answer) =
            crate::agentic_coding::edit_scope::scoped_decline(task, &rewrite.target, &source)
    {
        return Some(AgenticPlan::Final(answer));
    }
    if occurrences == 1 {
        // The bare pattern when it is unique in the file; otherwise the
        // smallest unique run of changed lines -- the edit tool refuses an
        // `oldString` it finds twice, and `smal` also sits inside `small`.
        let edit = if rewrite.unique {
            anchored_lines(&source, rewrite).map(|(_, old, new)| (old, new))
        } else if source.matches(rewrite.pattern.as_str()).count() == 1 {
            Some((rewrite.pattern.clone(), rewrite.replacement.clone()))
        } else {
            changed_lines_edit(&source, &updated)
        };
        if let Some(tool) = tool_for(tool_names, Capability::Edit)
            && let Some((old, new)) = edit
        {
            if result_for_edit(current_turn, &rewrite.target, &old, &new).is_none() {
                return Some(plan_one(tool, edit_arguments(&rewrite.target, &old, &new)));
            }
            if let Some(failed) = plan_edit_failure(
                task,
                current_turn,
                tool,
                &edit_arguments(&rewrite.target, &old, &new),
                result,
            ) {
                return Some(failed);
            }
            let verification = VerifiedChange {
                target: &rewrite.target,
                expected: &updated,
                intent: rewrite.stated_intent(),
                slots: &rewrite.stated_slots(),
                list_slots: &[],
            };
            if let Some(observed) = plan_edit_read_verification(
                task,
                current_turn,
                tool_names,
                &verification,
                &edit_arguments(&rewrite.target, &old, &new),
                result,
            ) {
                return Some(observed);
            }
            return plan_digest_verification(
                task,
                current_turn,
                tool_names,
                &VerifiedChange {
                    target: &rewrite.target,
                    expected: &updated,
                    intent: rewrite.stated_intent(),
                    slots: &rewrite.stated_slots(),
                    list_slots: &[],
                },
                result,
            );
        }
    } else if tool_for(tool_names, Capability::Edit).is_some()
        && let Some(command) = repeated_identifier_rewrite_command(rewrite)
        && let Some(tool) = tool_for(tool_names, Capability::Run)
    {
        if result_for_command(current_turn, &command).is_none() {
            return Some(plan_one(tool, json!({"command": command}).to_string()));
        }
        return plan_digest_verification(
            task,
            current_turn,
            tool_names,
            &VerifiedChange {
                target: &rewrite.target,
                expected: &updated,
                intent: rewrite.stated_intent(),
                slots: &rewrite.stated_slots(),
                list_slots: &[],
            },
            result,
        );
    }

    if result_for_path(
        current_turn,
        Capability::Write,
        &rewrite.target,
        Some(&updated),
    )
    .is_none()
    {
        let tool = tool_for(tool_names, Capability::Write)?;
        return Some(plan_one(tool, write_arguments(&rewrite.target, &updated)));
    }
    plan_write_digest_verification(
        task,
        current_turn,
        tool_names,
        &VerifiedChange {
            target: &rewrite.target,
            expected: &updated,
            intent: rewrite.stated_intent(),
            slots: &rewrite.stated_slots(),
            list_slots: &[],
        },
        result,
    )
}
