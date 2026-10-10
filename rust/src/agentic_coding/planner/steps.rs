//! Repeated-step stops and the explicit shell transaction shared by the planner.

use super::{AgenticPlan, Capability, Progress, plan_shell_step, tool_for, tool_result};
use crate::agentic_coding::final_result::FinalResult;
use crate::agentic_coding::{mutating_action, shell_command, shell_file_fallback};
use crate::protocol::ChatMessage;

/// A tool call this turn already completed is not a plan again (issue #1154).
///
/// Independent of any argument parsing: whatever keys a client names its
/// arguments by, a stateless planner that re-derives a call whose result is
/// already in the transcript ignored that result. The Codex Rust run planned
/// the identical `gh issue view` 78 times over 6.2M input tokens because the
/// read's accounting could not see the `cmd` key; this stop would have ended
/// the loop on the second attempt whatever the parser did. The repeat is
/// answered by reporting the stuck step and its last result.
pub(super) fn stop_repeated_call(plan: AgenticPlan, messages: &[ChatMessage]) -> AgenticPlan {
    let AgenticPlan::ToolCalls(calls) = &plan else {
        return plan;
    };
    let progress = Progress::scan(messages);
    let Some((repeated, attempt)) = calls
        .iter()
        .find_map(|call| progress.repeated_call(call).map(|attempt| (call, attempt)))
    else {
        return plan;
    };
    AgenticPlan::Final(crate::agentic_coding::work_item_steps::fill(
        "stuck_step_report",
        &[
            (
                "{step}",
                format!("{} {}", repeated.tool, repeated.arguments).as_str(),
            ),
            ("{result}", attempt.detail.trim()),
        ],
    ))
}

/// A tool call that has already failed twice this turn with the same report
/// is not a plan; the third attempt is replaced by the report of the failure.
///
/// Issue #1133: the Kotlin run planned `mcp__playwright__browser_click` 547
/// times, each answered by the same selector error, until the context window
/// overflowed. No route knows it is looping -- each re-derives the same next
/// step from the same transcript -- so the stop is applied to whatever any
/// route planned.
pub(super) fn stop_repeated_failure(plan: AgenticPlan, messages: &[ChatMessage]) -> AgenticPlan {
    const REPEATED_FAILURES_THAT_STOP: usize = 2;
    let AgenticPlan::ToolCalls(calls) = &plan else {
        return plan;
    };
    let progress = Progress::scan(messages);
    let Some(repeated) = calls
        .iter()
        .find(|call| progress.identical_failures_of(&call.tool) >= REPEATED_FAILURES_THAT_STOP)
    else {
        return plan;
    };
    let Some(failure) = progress.latest_failure_of_tool(&repeated.tool) else {
        return plan;
    };
    let prompt = crate::protocol::latest_user_request(messages).unwrap_or_default();
    AgenticPlan::Final(tool_result::render_failure(
        &repeated.tool,
        &failure.detail,
        &prompt,
    ))
}

pub(super) fn explicit_shell_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    tool_for(tool_names, Capability::Run)?;
    let command = shell_command::explicit_passthrough_command(task)?;
    shell_file_fallback::plan_step(task, messages, tool_names, &command)
        .or_else(|| mutating_action::plan_step(&command, messages, tool_names, task, result))
        .or_else(|| Some(plan_shell_step(messages, tool_names, &command, result)))
}

/// The seeded question for an addition that quotes no text (PR #1188 G69).
///
/// `None` for any other unplanned local edit, which is declined.
pub(super) fn unquoted_addition_question(task: &str) -> Option<AgenticPlan> {
    crate::agentic_coding::positional_edit::unquoted_addition_path(task)
        .and_then(|path| {
            crate::agentic_coding::code_task::render_seeded_change(
                "file_addition_unquoted",
                task,
                &path,
                &[],
            )
        })
        .map(AgenticPlan::Final)
}

pub(super) fn plan_verified_source_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    crate::agentic_coding::code_task::plan_verified_generated_source_step(
        task, messages, tool_names, result,
    )
    .or_else(|| {
        crate::agentic_coding::workspace_change::plan_workspace_change_step(
            task, messages, tool_names, result,
        )
    })
}

/// Preserve verified source priority and owned-goal refusal before later routes.
pub(super) fn plan_source_or_owned_goal_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<Option<AgenticPlan>> {
    let source = crate::agentic_coding::code_task::verified_source_description(task)
        .map(|_| plan_verified_source_step(task, messages, tool_names, result));
    source.or_else(|| {
        let owned_goal =
            if crate::agentic_coding::evidence_record::has_typed_evidence_delivery(task)
                || crate::meta_translate::owned_source_tree_request(task).is_some()
            {
                None
            } else {
                crate::agentic_coding::general_planner::plan_owned_goal_step(
                    task,
                    messages,
                    tool_names,
                    super::plan_chat_step_resolved,
                    result,
                )
            };
        owned_goal
            .or_else(|| {
                crate::agentic_coding::quote_nesting::request_fault_answer(
                    task,
                    tool_for(tool_names, Capability::MultiEdit).is_some(),
                )
            })
            .or_else(|| crate::computer_use::plan_agentic_step(messages, tool_names))
            .map(Some)
    })
}
