//! Client-visible workspace recovery through one canonical bounded installer.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::progress::{Progress, write_matches};
use super::tool_result;
use crate::protocol::ChatMessage;
use serde_json::json;

const BOOTSTRAP_PATH: &str = ".formal-ai/bootstrap/prerequisite-installer.py";
const BOOTSTRAP: &str = include_str!("../../embedded/data/meta/prerequisite-installer.py");

fn context(command: &str, program: &str) -> Option<String> {
    let grant = crate::prerequisite::configured_workspace_grant()?;
    if !grant
        .get("programs")?
        .as_array()?
        .iter()
        .any(|value| value.as_str() == Some(program))
    {
        return None;
    }
    if grant
        .get("bootstrap")
        .and_then(|value| value.get("program"))
        .and_then(serde_json::Value::as_str)
        != Some("python3")
        || grant
            .get("bootstrap")
            .and_then(|value| value.get("arguments"))
            != Some(&json!([BOOTSTRAP_PATH]))
        || grant
            .get("allowNetwork")
            .and_then(serde_json::Value::as_bool)
            != Some(true)
    {
        return None;
    }
    let publisher = crate::prerequisite::publisher::seed_publishers()
        .into_iter()
        .find(|publisher| publisher.program == program)?;
    let dependencies = crate::prerequisite::probe::seed_probes()
        .into_iter()
        .find(|probe| probe.program == program)
        .map_or_else(Vec::new, |probe| probe.requires);
    let documentation = publisher.documentation_url();
    let request = json!({"program":program,"grant":grant,"dependencies":dependencies,"publisher":{"program":publisher.program,"source_id":publisher.source_id,"host":publisher.host,"documentation":documentation}});
    let quote = super::git_commit::shell_quote;
    let active = [".formal-ai/toolchains/active-", program, ".bin"].concat();
    Some(["python3 ",&quote(BOOTSTRAP_PATH)," ",&quote(&request.to_string())," && read -r __formal_ai_prerequisite_bin < ",&quote(&active)," && export PATH=\"$__formal_ai_prerequisite_bin:$PATH\" &&\n# __formal_ai_prerequisite_retry\n",command].concat())
}

pub(super) fn recovery_output(messages: &[ChatMessage], command: &str) -> Option<String> {
    let program = command.split_whitespace().next()?;
    let recovery = context(command, program)?;
    Progress::scan(messages)
        .latest_run_output_for(&recovery)
        .cloned()
}

pub(super) fn plan_recovery(
    messages: &[ChatMessage],
    tools: &[&str],
    command: &str,
    exit_code: Option<i32>,
    output: &str,
) -> Option<AgenticPlan> {
    plan_recovery_with_identity(messages, tools, command, exit_code, output, command)
}

pub(super) fn plan_recovery_with_identity(
    messages: &[ChatMessage],
    tools: &[&str],
    command: &str,
    exit_code: Option<i32>,
    output: &str,
    source_command: &str,
) -> Option<AgenticPlan> {
    let need =
        crate::prerequisite::classify_failure(source_command, exit_code, output, source_command)?;
    if !matches!(
        need.observed,
        crate::prerequisite::probe::ProbeVerdict::Missing { .. }
    ) {
        return None;
    }
    let Some(recovery) = context(command, &need.program) else {
        return Some(AgenticPlan::Final(super::work_item_steps::fill(
            "prerequisite_grant_required_report",
            &[
                (concat!("{", "program}"), need.program.as_str()),
                (concat!("{", "command}"), command),
                (concat!("{", "output}"), output),
            ],
        )));
    };
    let progress = Progress::scan(messages);
    if let Some(observed) = progress.latest_run_output_for(&recovery) {
        return (super::tool_result::step_outcome(observed)
            == super::tool_result::StepOutcome::Failed)
            .then(|| AgenticPlan::Final(observed.clone()));
    }
    let (Some(write), Some(run)) = (
        tool_for(tools, Capability::Write),
        tool_for(tools, Capability::Run),
    ) else {
        return Some(AgenticPlan::Final(super::work_item_steps::fill(
            "prerequisite_grant_exceeded_report",
            &[],
        )));
    };
    let written = progress
        .latest_successful_write_index(BOOTSTRAP_PATH)
        .and_then(|index| progress.research_attempts().get(index))
        .and_then(|attempt| attempt.arguments.as_deref())
        .is_some_and(|arguments| write_matches(arguments, BOOTSTRAP_PATH, BOOTSTRAP));

    Some(if written {
        plan_one(run, json!({"command":recovery}).to_string())
    } else {
        plan_one(write, write_arguments(BOOTSTRAP_PATH, BOOTSTRAP))
    })
}

/// Observe an exact shell transaction, recover a missing prerequisite, and retain its final disposition.
pub(super) fn plan_shell_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    command: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    let recovered = recovery_output(messages, command);
    if let Some(raw) = recovered
        .as_ref()
        .or_else(|| progress.latest_run_output_for(command))
    {
        if tool_result::step_outcome(raw) == tool_result::StepOutcome::Failed
            && let Some(recovery) = plan_recovery(
                messages,
                tool_names,
                command,
                tool_result::reported_exit_code(raw)
                    .and_then(|exit_code| i32::try_from(exit_code).ok()),
                raw,
            )
        {
            return match recovery {
                AgenticPlan::Final(reason) => {
                    let failure = tool_result::render(
                        command,
                        raw,
                        crate::protocol::latest_user_request(messages)
                            .as_deref()
                            .unwrap_or_default(),
                    );
                    record(
                        AgenticPlan::Final(super::wording::agentic_message(
                            "shell-recovery-final-failure",
                            &[("failure", &failure), ("recovery", &reason)],
                        )),
                        FinalDisposition::Failure,
                        "shell_result_observed",
                        result,
                    )
                }
                plan @ AgenticPlan::ToolCalls(_) => plan,
            };
        }
        let disposition = if tool_result::step_outcome(raw) == tool_result::StepOutcome::Failed {
            FinalDisposition::Failure
        } else {
            FinalDisposition::Finding
        };
        return record(
            AgenticPlan::Final(tool_result::render(
                command,
                raw,
                crate::protocol::latest_user_request(messages)
                    .as_deref()
                    .unwrap_or_default(),
            )),
            disposition,
            "shell_result_observed",
            result,
        );
    }

    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        return plan_one(tool, json!({ "command": command }).to_string());
    }

    AgenticPlan::Final(super::wording::agentic_message(
        "shell_tool_missing",
        &[("command", command)],
    ))
}

/// Bind a current-turn failed command or its exact authored recovery transaction.
/// Observations choose a route; existing caller grants and recovery own all authority.
pub(super) fn observed_recovery_command(messages: &[ChatMessage]) -> Option<String> {
    use super::qualified_tool_observation::Binding;
    let progress = Progress::scan(messages);
    let attempts = progress.research_attempts();
    let latest_index = attempts
        .iter()
        .rposition(|attempt| attempt.capability == Capability::Run)?;
    let latest = &attempts[latest_index];
    let latest_arguments = latest.arguments.as_deref()?;
    let latest_command = tool_result::command_argument(latest_arguments)?;
    for attempt in attempts[..=latest_index].iter().rev() {
        if attempt.capability != Capability::Run {
            continue;
        }
        let Some(arguments) = attempt.arguments.as_deref() else {
            continue;
        };
        let Some(command) = tool_result::command_argument(arguments) else {
            continue;
        };
        let Some(receipt) = progress.latest_run_output_for(&command) else {
            continue;
        };
        if tool_result::step_outcome(receipt) != tool_result::StepOutcome::Failed {
            continue;
        }
        let Ok(arguments_value) = serde_json::from_str(arguments) else {
            continue;
        };
        let Some(bound) = progress.latest_attempt_for(Capability::Run, &arguments_value) else {
            continue;
        };
        if bound.binding != Binding::Exact {
            continue;
        }
        let Some(need) = crate::prerequisite::classify_failure(
            &command,
            tool_result::reported_exit_code(receipt).and_then(|value| i32::try_from(value).ok()),
            receipt,
            &command,
        ) else {
            continue;
        };
        if !matches!(
            need.observed,
            crate::prerequisite::probe::ProbeVerdict::Missing { .. }
        ) {
            continue;
        }
        let Some(recovery) = context(&command, &need.program) else {
            continue;
        };
        if latest_command != command && latest_command != recovery {
            continue;
        }
        let latest_value = serde_json::from_str(latest_arguments).ok()?;
        if progress
            .latest_attempt_for(Capability::Run, &latest_value)?
            .binding
            != Binding::Exact
        {
            return None;
        }
        for following in &attempts[latest_index + 1..] {
            if following.capability != Capability::Write || !following.succeeded {
                return None;
            }
            let written_arguments = following.arguments.as_deref()?;
            if !write_matches(written_arguments, BOOTSTRAP_PATH, BOOTSTRAP) {
                return None;
            }
            let written_value = serde_json::from_str(written_arguments).ok()?;
            let written = progress.latest_attempt_for(Capability::Write, &written_value)?;
            if written.binding != Binding::Exact || !written.succeeded {
                return None;
            }
        }
        return Some(command);
    }
    None
}
