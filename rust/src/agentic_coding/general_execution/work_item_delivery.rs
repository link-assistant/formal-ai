use super::*;

/// The step that reads the work item, through whichever client tool reaches it.
pub(super) fn plan_work_item_read(
    messages: &[ChatMessage],
    tool_names: &[&str],
    target: &str,
    progress: &Progress,
) -> Option<AgenticPlan> {
    let fetch = tool_for(tool_names, Capability::Fetch);
    if !progress.attempted_work_item_read_of(target)
        && let Some(run) = shell_command_tool(tool_names)
    {
        return Some(plan_one(
            run,
            json!({ "command": issue_view_command(target) }).to_string(),
        ));
    }
    // A read that failed — or one a client echoed without its status, which
    // the sentinel now exposes (issue #1155) — must not end the retrieval
    // while another route to the same text is untried. The fallbacks walk in
    // the issue's order: the client's fetch tool on the issue URL, the
    // credential-free REST read (`curl` needs no `gh` and no token for a
    // public repository — exactly the case an unauthenticated `gh` used to
    // end a session for), the authenticated REST read, and last the prepared
    // pull request's own page, whose title and body restate the issue.
    if !progress.attempted_fetch_of(target)
        && let Some(tool) = fetch
    {
        return Some(plan_one(tool, work_item_fetch_arguments(target)));
    }
    if let Some(run) = shell_command_tool(tool_names) {
        for fallback in [
            issue_rest_read_command(target),
            issue_api_read_command(target),
        ]
        .into_iter()
        .flatten()
        {
            if !progress.has_run(&fallback) {
                return Some(plan_one(run, json!({ "command": fallback }).to_string()));
            }
        }
    }
    if let Some(prepared_pull) = prepared_pull_reference(messages)
        && prepared_pull != target
        && !progress.attempted_fetch_of(&prepared_pull)
        && let Some(tool) = fetch
    {
        return Some(plan_one(tool, work_item_fetch_arguments(&prepared_pull)));
    }
    None
}

pub(super) fn finish_general_change(
    plan: &GeneralChangePlan,
    progress: &Progress,
    resolved_from_work_item: bool,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    if plan.mode == GeneralPlanMode::RepositoryWorkItem {
        return record(
            AgenticPlan::Final(plan.planned_not_executed_answer()),
            FinalDisposition::Gap,
            "repository_work_not_executed",
            result,
        );
    }
    // The workspace gets the last word over a completion claim: a verification
    // command that exited non-zero replaces the claim with its own report.
    if let Some(report) = progress
        .latest_run_output_for(&plan.verification_command)
        .filter(|output| !literal_verification_matches(plan, output))
        .and_then(|output| {
            tool_result::failed_verification(
                std::slice::from_ref(output),
                &plan.verification_command,
                &plan.goal,
            )
        })
    {
        return AgenticPlan::Final(report);
    }
    if progress.successful_run_count_for(&plan.verification_command) == 0 {
        return AgenticPlan::Final(general_plan_unverified(plan));
    }
    if plan.mode == GeneralPlanMode::LiteralFile {
        let observed = progress
            .latest_successful_run_output_for(&plan.verification_command)
            .and_then(tool_result::observed_payload);
        let exact_literal = progress
            .latest_successful_run_output_for(&plan.verification_command)
            .is_some_and(|output| literal_verification_matches(plan, output));
        if !exact_literal && observed.as_deref().map(str::trim) != Some(plan.content.trim()) {
            return AgenticPlan::Final(general_plan_mismatch(
                plan,
                observed.as_deref().unwrap_or_default(),
            ));
        }
    }
    if resolved_from_work_item && plan.mode == GeneralPlanMode::LiteralFile {
        return record(
            AgenticPlan::Final(work_item_completion(plan, progress)),
            FinalDisposition::Finding,
            "work_item_completion_verified",
            result,
        );
    }
    record(
        AgenticPlan::Final(general_plan_completed(plan)),
        FinalDisposition::Finding,
        "general_plan_completed_verified",
        result,
    )
}
