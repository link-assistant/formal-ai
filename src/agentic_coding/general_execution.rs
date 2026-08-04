//! Execution state machine for a composed general change plan.

use serde_json::json;

use super::general_planner::{GeneralChangePlan, GeneralPlanMode, PLAN_PATH};
use super::planner::{plan_one, tool_for, write_arguments, AgenticPlan, Capability};
use super::progress::Progress;
use super::tool_result;
use crate::protocol::ChatMessage;

const GENERAL_PLAN_BODY_PLACEHOLDER: &str = concat!("{", "plan", "}");
const EXPECTED_PLACEHOLDER: &str = concat!("{", "expected", "}");
const OBSERVED_PLACEHOLDER: &str = concat!("{", "observed", "}");

pub(super) fn plan_general_change_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan: &GeneralChangePlan,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    if let Some(failure) = progress.latest_failure() {
        if failure.capability == Capability::Write {
            let path = failure.arguments.as_deref().and_then(tool_argument_path);
            if let (Some(path), Some(read_tool)) =
                (path.as_deref(), tool_for(tool_names, Capability::Read))
            {
                if progress.failed_write_count_for(path) == 1 {
                    return plan_one(read_tool, read_arguments(path));
                }
            }
            // The plan event is auxiliary. A write-only client cannot perform
            // the preferred read-before-retry recovery, but its failure must
            // not swallow the user's primary literal-file write.
            if path.as_deref() == Some(PLAN_PATH)
                && plan.mode == GeneralPlanMode::LiteralFile
                && tool_for(tool_names, Capability::Read).is_none()
                && progress.failed_write_count_for(PLAN_PATH) == 1
            {
                if let Some(write_tool) = tool_for(tool_names, Capability::Write) {
                    return plan_one(write_tool, write_arguments(&plan.target, &plan.content));
                }
            }
            return AgenticPlan::Final(tool_result::render_failure(
                path.as_deref().unwrap_or("write"),
                &failure.detail,
                &plan.goal,
            ));
        }
        // A client may require an attempted read before creating a missing
        // file. Whether that read found bytes or reported absence, retry the
        // original write once; its per-target failure budget remains bounded.
        let recovering_missing_write = failure.capability == Capability::Read
            && progress.previous_attempt().is_some_and(|previous| {
                previous.capability == Capability::Write && !previous.succeeded
            });
        if !recovering_missing_write {
            return AgenticPlan::Final(tool_result::render_failure(
                capability_label(failure.capability),
                &failure.detail,
                &plan.goal,
            ));
        }
    }
    let recovering_write = progress.last() == Some(Capability::Read)
        && progress.previous_attempt().is_some_and(|previous| {
            previous.capability == Capability::Write && !previous.succeeded
        });
    if recovering_write {
        if let (Some(tool), Some(arguments)) = (
            tool_for(tool_names, Capability::Write),
            progress
                .previous_attempt()
                .and_then(|attempt| attempt.arguments.as_deref()),
        ) {
            return plan_one(tool, arguments.to_owned());
        }
    }
    if let Some(tool) = tool_for(tool_names, Capability::Write) {
        let plan_attempted = progress.attempted_write_for(PLAN_PATH);
        let plan_written = progress.successful_write_for(PLAN_PATH);
        if !plan_attempted {
            return plan_one(tool, write_arguments(PLAN_PATH, &plan.links_notation()));
        }
        if plan.mode == GeneralPlanMode::LiteralFile
            && !progress.successful_write_for(&plan.target)
            && (plan_written || plan_attempted)
        {
            return plan_one(tool, write_arguments(&plan.target, &plan.content));
        }
    }
    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        let runs = progress.successful_count(Capability::Run);
        match plan.mode {
            GeneralPlanMode::CommandOutput => {
                if let Some(generation) = plan.steps.get(1).and_then(|step| step.command.as_deref())
                {
                    if runs == 0 {
                        return plan_one(tool, json!({ "command": generation }).to_string());
                    }
                    if runs == 1 {
                        return plan_one(
                            tool,
                            json!({ "command": plan.verification_command }).to_string(),
                        );
                    }
                }
            }
            GeneralPlanMode::LiteralFile | GeneralPlanMode::RepositoryWorkItem if runs == 0 => {
                return plan_one(
                    tool,
                    json!({ "command": plan.verification_command }).to_string(),
                );
            }
            GeneralPlanMode::LiteralFile | GeneralPlanMode::RepositoryWorkItem => {}
        }
    }
    finish_general_change(plan, &progress)
}

fn finish_general_change(plan: &GeneralChangePlan, progress: &Progress) -> AgenticPlan {
    if plan.mode == GeneralPlanMode::RepositoryWorkItem {
        if progress.successful_count(Capability::Run) == 0 {
            return AgenticPlan::Final(general_plan_unverified(plan));
        }
        let expected = plan.links_notation();
        let observed = progress
            .latest_successful_output(Capability::Run)
            .and_then(tool_result::observed_payload);
        if observed.as_deref().map(str::trim) != Some(expected.trim()) {
            return AgenticPlan::Final(general_plan_mismatch(
                plan,
                observed.as_deref().unwrap_or_default(),
            ));
        }
        let response_language = crate::language::detect(&plan.goal).slug();
        let mut answer =
            crate::seed::localized_response("general_plan_repository_complete", response_language)
                .unwrap_or_default();
        answer = answer.replace("{target}", &plan.target);
        answer = answer.replace("{plan_path}", PLAN_PATH);
        answer = answer.replace(
            GENERAL_PLAN_BODY_PLACEHOLDER,
            plan.links_notation().trim_end(),
        );
        return AgenticPlan::Final(answer);
    }
    if progress.successful_count(Capability::Run) == 0 {
        return AgenticPlan::Final(general_plan_unverified(plan));
    }
    if plan.mode == GeneralPlanMode::LiteralFile {
        let observed = progress
            .latest_successful_output(Capability::Run)
            .and_then(tool_result::observed_payload);
        if observed.as_deref().map(str::trim) != Some(plan.content.trim()) {
            return AgenticPlan::Final(general_plan_mismatch(
                plan,
                observed.as_deref().unwrap_or_default(),
            ));
        }
    }
    AgenticPlan::Final(format!(
        "Completed the general change request for {} and verified it with `{}`.\n\nPlan event ({}):\n\n{}",
        plan.target,
        plan.verification_command,
        PLAN_PATH,
        plan.links_notation().trim_end(),
    ))
}

fn tool_argument_path(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    ["path", "filePath", "file_path"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}

fn read_arguments(path: &str) -> String {
    json!({
        "path": path,
        "filePath": path,
        "file_path": path,
    })
    .to_string()
}

const fn capability_label(capability: Capability) -> &'static str {
    match capability {
        Capability::Search => "search",
        Capability::Fetch => "fetch",
        Capability::Read => "read",
        Capability::Write => "write",
        Capability::Edit => "edit",
        Capability::Run => "run",
        Capability::Grep => "grep",
        Capability::Glob => "glob",
        Capability::ListDir => "list directory",
        Capability::Todo => "todo",
        Capability::Subagent => "subagent",
        Capability::ReadMany => "read many",
        Capability::MultiEdit => "multi-edit",
        Capability::AskUser => "ask user",
    }
}

fn general_plan_mismatch(plan: &GeneralChangePlan, observed: &str) -> String {
    let language = crate::language::detect(&plan.goal).slug();
    let mut answer =
        crate::seed::localized_response("general_plan_verification_mismatch", language)
            .unwrap_or_default();
    answer = answer.replace("{target}", &plan.target);
    let expected = if plan.mode == GeneralPlanMode::RepositoryWorkItem {
        plan.links_notation()
    } else {
        plan.content.clone()
    };
    answer = answer.replace(EXPECTED_PLACEHOLDER, expected.trim());
    answer.replace(OBSERVED_PLACEHOLDER, observed.trim())
}

fn general_plan_unverified(plan: &GeneralChangePlan) -> String {
    let language = crate::language::detect(&plan.goal).slug();
    let mut answer =
        crate::seed::localized_response("general_plan_unverified", language).unwrap_or_default();
    answer = answer.replace("{target}", &plan.target);
    answer.replace("{command}", &plan.verification_command)
}
