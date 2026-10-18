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
    let need = crate::prerequisite::classify_failure(command, exit_code, output, command)?;
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
            return recovery;
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

    AgenticPlan::Final(format!(
        "I can run `{command}` when the client advertises a shell tool such as `bash`, `shell`, or `run_command`."
    ))
}
