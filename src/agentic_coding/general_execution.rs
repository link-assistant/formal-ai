//! Execution state machine for a composed general change plan.

use serde_json::json;

use super::general_planner::{
    compose_general_change_plan, GeneralChangePlan, GeneralPlanMode, PLAN_PATH,
};
use super::planner::{
    fetch_arguments, plan_one, tool_for, write_arguments, AgenticPlan, Capability,
};
use super::progress::Progress;
use super::tool_result;
use crate::protocol::ChatMessage;

const EXPECTED_PLACEHOLDER: &str = concat!("{", "expected", "}");
const OBSERVED_PLACEHOLDER: &str = concat!("{", "observed", "}");
const PLAN_PLACEHOLDER: &str = concat!("{", "plan", "}");
const PLAN_PATH_PLACEHOLDER: &str = concat!("{", "plan_path", "}");

pub(super) fn plan_general_change_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan: &GeneralChangePlan,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    // Reading the work item is an attempt to find out whether anything *can* be
    // executed, not a step the request named. A read that comes back missing or
    // empty therefore answers that question — the capability is unavailable —
    // and the run continues to record the reference. Reporting the fetch as the
    // run's failure would replace the honest terminal state of issue #904 with
    // a transport message about a URL the user never asked to see.
    let work_item_unreadable = plan.mode == GeneralPlanMode::RepositoryWorkItem
        && progress
            .latest_failure()
            .is_some_and(|failure| failure.capability == Capability::Fetch);
    if let Some(failure) = progress.latest_failure().filter(|_| !work_item_unreadable) {
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
            // Recovery is exhausted, but the failed call is not the last word:
            // run the check the plan itself named so the report can carry the
            // status the workspace answered with rather than a transport
            // message alone (issue #916, rung R916-01).
            if plan.mode != GeneralPlanMode::RepositoryWorkItem
                && !plan.verification_command.trim().is_empty()
                && progress.count(Capability::Run) == 0
            {
                if let Some(run_tool) = tool_for(tool_names, Capability::Run) {
                    return plan_one(
                        run_tool,
                        json!({ "command": plan.verification_command }).to_string(),
                    );
                }
            }
            return AgenticPlan::Final(tool_result::render_failure(
                path.as_deref().unwrap_or("write"),
                &failure.detail,
                &plan.goal,
            ));
        }
        // A failed run is reported as the command that failed and the status it
        // exited with, not as an anonymous "run" step (issues #905 and #908).
        if failure.capability == Capability::Run {
            if let Some(report) = tool_result::failed_verification(
                &progress.run_outputs,
                &plan.verification_command,
                &plan.goal,
            ) {
                return AgenticPlan::Final(report);
            }
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
    // A repository work item names an issue, not an artifact. Recording the
    // reference and stopping is what issue #904 reported: nothing the request
    // asked for was ever produced. The issue itself is where the artifact is
    // named, so read it before concluding that nothing can be executed — the
    // capability is only genuinely unavailable once the fetch has been tried.
    if plan.mode == GeneralPlanMode::RepositoryWorkItem {
        if let Some(fetched) = repository_work_item_objective(plan, &progress) {
            if let Some(step) = plan_work_item_execution(&fetched, messages, tool_names) {
                return step;
            }
        } else if let Some(tool) = tool_for(tool_names, Capability::Fetch) {
            if !progress
                .attempted_fetches
                .iter()
                .any(|url| url == &plan.target)
            {
                return plan_one(tool, fetch_arguments(&plan.target));
            }
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
            GeneralPlanMode::LiteralFile if runs == 0 => {
                return plan_one(
                    tool,
                    json!({ "command": plan.verification_command }).to_string(),
                );
            }
            // A repository work item names no verification command: the only
            // file this run writes is the plan record, and reading it back
            // would verify nothing but the write itself (issue #904).
            GeneralPlanMode::LiteralFile | GeneralPlanMode::RepositoryWorkItem => {}
        }
    }
    finish_general_change(plan, &progress)
}

/// Plan the next step from what the fetched work item actually asks for.
///
/// The real corpus decides the shape here. The issues Hive Mind dispatches say
/// *"implement a Hello World program in Scala"* — a described artifact in a
/// named language, not literal bytes — so the same coding catalog that answers
/// that request when a user types it directly answers it here, through the
/// execution recipe its [`SymbolicAnswer`](crate::solver::SymbolicAnswer)
/// carries. The literal-file composer follows, for a work item that does spell
/// out a path and its contents.
///
/// Returning [`None`] means the work item named nothing this sandbox can
/// produce — an unsupported language, or prose with no artifact in it at all —
/// which keeps `planned_not_executed` truthful rather than inventing an
/// artifact the issue never asked for.
fn plan_work_item_execution(
    objective: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let solver = crate::solver::UniversalSolver::new(crate::solver::SolverConfig {
        agent_mode: true,
        ..crate::solver::SolverConfig::default()
    });
    let answer = solver.solve(objective);
    if let Some(step) =
        super::command_reroute::plan_symbolic_command_reroute(messages, tool_names, &answer)
    {
        return Some(step);
    }
    let executable = compose_general_change_plan(objective)
        .filter(|executable| executable.mode != GeneralPlanMode::RepositoryWorkItem)?;
    Some(plan_general_change_step(messages, tool_names, &executable))
}

/// The text of the issue this work item names, once the client has fetched it.
///
/// Only the page fetched from the plan's own target counts. A repository work
/// item carries one URL, and answering it with whatever page happened to be
/// fetched for some other reason this turn would plan against the wrong issue.
fn repository_work_item_objective(plan: &GeneralChangePlan, progress: &Progress) -> Option<String> {
    progress
        .fetched_pages
        .iter()
        .find(|(url, _)| url == &plan.target)
        .map(|(_, text)| text.clone())
        .filter(|text| !text.trim().is_empty())
}

fn finish_general_change(plan: &GeneralChangePlan, progress: &Progress) -> AgenticPlan {
    if plan.mode == GeneralPlanMode::RepositoryWorkItem {
        return AgenticPlan::Final(plan.planned_not_executed_answer());
    }
    // The workspace gets the last word over a completion claim: a verification
    // command that exited non-zero replaces the claim with its own report.
    if let Some(report) = tool_result::failed_verification(
        &progress.run_outputs,
        &plan.verification_command,
        &plan.goal,
    ) {
        return AgenticPlan::Final(report);
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
    AgenticPlan::Final(general_plan_completed(plan))
}

/// The completion claim is the sentence the issue quotes as the false report,
/// so it is seeded like every other outcome of this state machine rather than
/// typed into Rust: the words live in `data/seed/`, and the reply follows the
/// language the request was written in.
fn general_plan_completed(plan: &GeneralChangePlan) -> String {
    let language = crate::language::detect(&plan.goal).slug();
    let mut answer =
        crate::seed::localized_response("general_plan_completed", language).unwrap_or_default();
    answer = answer.replace("{target}", &plan.target);
    answer = answer.replace("{command}", &plan.verification_command);
    answer = answer.replace(PLAN_PATH_PLACEHOLDER, PLAN_PATH);
    answer.replace(
        PLAN_PLACEHOLDER,
        &crate::issue_report::fenced_block(
            crate::issue_report::LINO_FENCE_LANGUAGE,
            &plan.links_notation(),
        ),
    )
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
