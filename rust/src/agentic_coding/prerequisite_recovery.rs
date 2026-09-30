//! Lower trusted typed setup recipes into client-owned tools (#1159).
//! Fetch, installation, probe and retry stay visible in the transcript.
use super::planner::{AgenticPlan, Capability, fetch_arguments, plan_one, tool_for};
use super::progress::Progress;
use crate::protocol::ChatMessage;
use serde_json::json;

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
    if std::env::var("FORMAL_AI_INSTALL_GRANT").ok().as_deref() != Some("workspace") {
        return Some(AgenticPlan::Final(format!(
            "Missing prerequisite `{}`. Workspace installation requires FORMAL_AI_INSTALL_GRANT=workspace. Failed command: `{command}`\n{output}",
            need.program,
        )));
    }
    let publisher = crate::prerequisite::publisher::seed_publishers()
        .into_iter()
        .find(|publisher| publisher.program == need.program)?;
    let url = publisher.documentation_url();
    let progress = Progress::scan(messages);
    let document = progress.fetched_pages.iter().find(|(page, _)| page == &url);
    let Some((_, document)) = document else {
        if progress.attempted_fetches.contains(&url) {
            return None;
        }
        return Some(plan_one(
            tool_for(tools, Capability::Fetch)?,
            fetch_arguments(&url),
        ));
    };
    let Some(procedure) =
        crate::prerequisite::publisher::parse_setup_document(&need, &publisher, &url, document)
    else {
        return Some(AgenticPlan::Final(format!(
            "Publisher `{url}` provided no typed, verifiable setup recipe for `{}`; installation was not planned.",
            need.program,
        )));
    };
    let quote = super::git_commit::shell_quote;
    let mut commands = Vec::new();
    for step in &procedure.steps {
        let path = &step.writes_under;
        // No absolute or parent path can leave the client-owned workspace.
        if path.is_absolute()
            || !path.starts_with(".formal-ai/toolchains")
            || path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
            || matches!(
                step.program.as_str(),
                "sudo" | "sh" | "bash" | "zsh" | "cmd" | "powershell"
            )
        {
            return Some(AgenticPlan::Final(
                "The publisher setup recipe exceeds the workspace installation grant.".to_owned(),
            ));
        }
        let invocation = std::iter::once(step.program.as_str())
            .chain(step.arguments.iter().map(String::as_str))
            .map(quote)
            .collect::<Vec<_>>()
            .join(" ");
        commands.push(invocation);
    }
    let probe = procedure.postcondition.as_ref()?;
    let probe_command = std::iter::once(probe.program.as_str())
        .chain(probe.argv.iter().map(String::as_str))
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ");
    commands.push(probe_command);
    let run = tool_for(tools, Capability::Run)?;
    let bins = procedure
        .steps
        .iter()
        .map(|step| format!("{}/bin", step.writes_under.display()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(":");
    let prefix = format!("PATH={}:\"$PATH\"; export PATH", quote(&bins));
    let recovery = format!(
        "{prefix} && {} &&\n# __formal_ai_prerequisite_retry\n{command}",
        commands.join(" && ")
    );
    if progress.run_count_for(&recovery) > 0 {
        return None;
    }
    Some(plan_one(run, json!({"command": recovery}).to_string()))
}
