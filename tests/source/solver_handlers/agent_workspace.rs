//! Agent-mode projection for bounded workspace tasks.

use std::fmt::Write as _;

use crate::agent::{parse_agent_plan, run_agent_plan, AgentAction, AgentRunStatus};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

use super::finalize_simple;

pub fn try_agent_workspace_task(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if parse_agent_plan(prompt).is_empty() {
        return None;
    }

    log.append("agent_mode:opted_in", prompt.to_owned());
    log.append("agent_mode:active", prompt.to_owned());
    log.append(
        "execution_environment",
        "isolated temp workspace with cleared command environment".to_owned(),
    );

    let config = crate::agent::AgentWorkspaceConfig::default();
    let run = match run_agent_plan(prompt, &config) {
        Ok(run) => run,
        Err(error) => {
            log.append("trace:execution_failure", error.to_string());
            log.append("execution_status", "agent:failed".to_owned());
            let body = format!(
                "Execution status: failed in isolated sandbox.\nWorkspace isolation: could not \
                 create the temp workspace.\nError: {error}"
            );
            return Some(finalize_simple(
                prompt,
                log,
                "agent_workspace_task_failed",
                "response:agent_workspace_task_failed",
                &body,
                0.2,
            ));
        }
    };

    log.append("agent_workspace", run.workspace.display().to_string());
    for action in &run.actions {
        log.append(action.event_kind(), action.evidence_payload());
    }
    let status_label = match run.status {
        AgentRunStatus::Completed => "agent:completed",
        AgentRunStatus::Failed => "agent:failed",
    };
    log.append("execution_status", status_label.to_owned());
    if run.status == AgentRunStatus::Failed {
        log.append(
            "trace:execution_failure",
            "agent workspace action failed".to_owned(),
        );
    }

    let body = render_agent_run(&run);
    let (intent, response, confidence) = match run.status {
        AgentRunStatus::Completed => ("agent_workspace_task", "response:agent_workspace_task", 0.9),
        AgentRunStatus::Failed => (
            "agent_workspace_task_failed",
            "response:agent_workspace_task_failed",
            0.4,
        ),
    };
    Some(finalize_simple(
        prompt, log, intent, response, &body, confidence,
    ))
}

fn render_agent_run(run: &crate::agent::AgentRun) -> String {
    let status = match run.status {
        AgentRunStatus::Completed => "completed",
        AgentRunStatus::Failed => "failed",
    };
    let mut body = format!(
        "Execution status: {status} in isolated sandbox.\nWorkspace isolation: {} \
         (paths are confined to this workspace; command environment is cleared).\n\nAction log:",
        run.workspace.display()
    );
    for action in &run.actions {
        let _ = write!(body, "\n- {}", action_summary(action));
    }
    for command in &run.command_results {
        let output = if command.stdout.is_empty() {
            "(no output)"
        } else {
            command.stdout.trim_end()
        };
        let _ = write!(
            body,
            "\n\nCommand: `{}`\nExit: {:?}\nOutput:\n```text\n{}\n```",
            command.command, command.status_code, output
        );
        if !command.stderr.trim().is_empty() {
            let _ = write!(
                body,
                "\nStderr:\n```text\n{}\n```",
                command.stderr.trim_end()
            );
        }
        if command.timed_out {
            body.push_str("\nTimed out: true");
        }
    }
    body
}

fn action_summary(action: &AgentAction) -> String {
    let (completed, failed) = match action.kind {
        crate::agent::AgentActionKind::CreateFile => ("created", "create"),
        crate::agent::AgentActionKind::ModifyFile => ("modified", "modify"),
        crate::agent::AgentActionKind::DeleteFile => ("deleted", "delete"),
        crate::agent::AgentActionKind::RunCommand => ("ran command", "run command"),
    };
    if action.status == crate::agent::AgentActionStatus::Completed {
        format!("{completed} {}", action.target)
    } else {
        format!("failed to {failed} {}: {}", action.target, action.detail)
    }
}
