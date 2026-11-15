//! Repeated-step stops and the explicit shell transaction shared by the planner.

use super::{AgenticPlan, Capability, Progress, plan_shell_step, tool_for, tool_result};
use crate::agentic_coding::final_result::FinalResult;
use crate::agentic_coding::{mutating_action, shell_command, shell_file_fallback};
use crate::protocol::ChatMessage;

/// Keep durable coding progress while explicit source reads require fresh evidence.
fn repetition_progress(messages: &[ChatMessage]) -> Progress {
    // Only later user requests can create a shorter source-read window.
    if messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .is_none_or(|index| index == 0)
    {
        return Progress::scan(messages);
    }
    let current = super::super::file_read::file_read_evidence(messages);
    if current.len() == messages.len() {
        return Progress::scan(messages);
    }
    let request = crate::protocol::latest_user_request(messages).unwrap_or_default();
    let current = if super::super::file_read::file_read_task_for(&request).is_some() {
        current
    } else {
        messages
    };
    Progress::scan(current)
}

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
    let progress = repetition_progress(messages);
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
    let progress = repetition_progress(messages);
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

pub(super) enum SourceOrOwnedGoalStep {
    Unclaimed,
    Claimed(Option<AgenticPlan>),
}

/// Resolve request ownership before incidental routes or prerequisite gaps.
pub(super) fn plan_request_preflight(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> SourceOrOwnedGoalStep {
    if let Some(contract) =
        crate::agentic_coding::file_command_contract::file_command_contract(task)
        && contract["operation"] != "listing"
        && !(tool_for(tool_names, Capability::Read).is_some()
            && crate::agentic_coding::file_read::file_read_task_for(task).is_some())
        && crate::agentic_coding::workspace_search::content_search_for(task).is_none()
        && !crate::agentic_coding::local_search::owns_path_discovery(task)
        && (contract["operation"] != "search" || tool_for(tool_names, Capability::Grep).is_none())
    {
        let plan = if let Some(command) = contract["command"].as_str() {
            let progress = Progress::scan(messages);
            if contract["operation"] == "count" {
                if let Some(raw) = progress.latest_run_output_for(command) {
                    let status = tool_result::reported_exit_code(raw);
                    let output =
                        tool_result::normalized_payload(raw).map(|value| value.trim().to_owned());
                    let valid = output.as_deref().and_then(|value| {
                        value
                            .parse::<usize>()
                            .ok()
                            .filter(|count| *count <= 4096 && count.to_string() == value)
                    });
                    if status == Some(0) && valid.is_some() {
                        return SourceOrOwnedGoalStep::Claimed(Some(
                            crate::agentic_coding::final_result::record(
                                AgenticPlan::Final(output.expect("ObservedFileCount")),
                                crate::agentic_coding::final_result::FinalDisposition::Finding,
                                "file-count-observed",
                                result,
                            ),
                        ));
                    }
                    if matches!(status, None | Some(0)) {
                        let reason = if status.is_none() {
                            "FileCountStatusUnavailable"
                        } else {
                            "FileCountOutputUnsupported"
                        };
                        return SourceOrOwnedGoalStep::Claimed(Some(crate::agentic_coding::final_result::record(AgenticPlan::Final(serde_json::json!({"schema":"file-command-contract/v1","operation":"count","complete":false,"verified":false,"reason":reason}).to_string()),crate::agentic_coding::final_result::FinalDisposition::Gap,"file-count-observed",result)));
                    }
                }
            }
            plan_shell_step(messages, tool_names, command, result)
        } else {
            crate::agentic_coding::final_result::record(
                AgenticPlan::Final(contract.to_string()),
                crate::agentic_coding::final_result::FinalDisposition::Gap,
                "file-command-contract",
                result,
            )
        };
        return SourceOrOwnedGoalStep::Claimed(Some(plan));
    }
    // Declared issue ownership and continuation/prerequisite guards share the
    // request boundary; surrounding examples cannot become independent effects.
    if let Some(plan) = crate::agentic_coding::general_planner::compose_general_change_plan(task)
        && (plan.mode == crate::agentic_coding::general_planner::GeneralPlanMode::CommandOutput
            || plan.mode
                == crate::agentic_coding::general_planner::GeneralPlanMode::RepositoryWorkItem
                && task.trim_start().starts_with(&plan.target))
    {
        return SourceOrOwnedGoalStep::Claimed(Some(
            crate::agentic_coding::general_execution::plan_general_change_step(
                messages, tool_names, &plan, result,
            ),
        ));
    }
    if crate::rule_interpreter::handler_matches("conversation_control", task)
        || super::is_continuation_cue(task)
    {
        return SourceOrOwnedGoalStep::Claimed(None);
    }
    if let Some(plan) = crate::agentic_coding::general_planner::pending_read_gap(task, result) {
        return SourceOrOwnedGoalStep::Claimed(Some(plan));
    }
    // Quoted edit payloads are data, not new documentation or repair instructions.
    if crate::agentic_coding::general_planner::compose_edit_request(task).is_some() {
        return SourceOrOwnedGoalStep::Unclaimed;
    }
    if let Some(step) =
        crate::agentic_coding::work_item_boundary_candidate::plan_work_item_boundary_step(
            task,
            messages,
            tool_names,
            None,
            |clause| {
                crate::agentic_coding::work_item_evidence::work_item_evidence_paths(clause)
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            },
            result,
        )
    {
        return SourceOrOwnedGoalStep::Claimed(Some(step));
    }
    if crate::agentic_coding::source_maintenance::owned_source_maintenance_request(task).is_none()
        && !crate::agentic_coding::function_expectation::has_function_expectation(task)
        && let Some(step) = crate::agentic_coding::source_repair::plan_owned_source_repair(
            task, messages, tool_names, None, result,
        )
    {
        return SourceOrOwnedGoalStep::Claimed(Some(step));
    }
    if let Some(step) =
        crate::agentic_coding::repository_documentation::plan_repository_documentation(
            task, messages, tool_names, None, result,
        )
    {
        return SourceOrOwnedGoalStep::Claimed(Some(step));
    }
    if let Some(step) = crate::agentic_coding::source_documentation::plan_source_documentation_step(
        task, messages, tool_names, None, result,
    ) {
        return SourceOrOwnedGoalStep::Claimed(Some(step));
    }
    SourceOrOwnedGoalStep::Unclaimed
}

/// Preserve verified source priority and owned-goal refusal before later routes.
pub(super) fn plan_source_or_owned_goal_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> SourceOrOwnedGoalStep {
    if crate::agentic_coding::code_task::verified_source_description(task).is_some() {
        return SourceOrOwnedGoalStep::Claimed(plan_verified_source_step(
            task, messages, tool_names, result,
        ));
    }
    {
        let owned_goal =
            if crate::agentic_coding::evidence_record::has_typed_evidence_delivery(task)
                || crate::meta_translate::owned_source_tree_request(task).is_some()
                || crate::agentic_coding::statement_audit::owned_statement_audit_output(task)
                    .is_some()
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
            .map_or(SourceOrOwnedGoalStep::Unclaimed, |plan| {
                SourceOrOwnedGoalStep::Claimed(Some(plan))
            })
    }
}
