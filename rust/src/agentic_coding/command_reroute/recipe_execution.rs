use super::*;

pub(super) fn recipe_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    symbolic_answer: &SymbolicAnswer,
    objective: &str,
    recovery_only: bool,
) -> Option<AgenticPlan> {
    let request = crate::protocol::latest_user_request(messages).unwrap_or_default();
    let mut requested = requested_recipe(&request, symbolic_answer.execution_recipe.as_ref()?);
    if super::super::ci_workflow::requested_in(objective) {
        super::super::ci_workflow::attach(&mut requested, objective);
    }
    let recipe = &requested;
    let write_tool = tool_for(tool_names, Capability::Write).or_else(|| {
        tool_names
            .iter()
            .copied()
            .find(|name| is_workspace_creation_tool(name))
    })?;
    let run_tool = tool_for(tool_names, Capability::Run)?;
    // A request that names the pull request or issue the work is for is asking
    // for the result to land there, so the recipe ends by committing and
    // pushing it (issue #1133).
    let mut commit = crate::protocol::latest_user_request(messages)
        .as_deref()
        .and_then(git_commit::target_of);
    let requirements_reference = commit.as_ref().map(|target| target.reference.clone());
    match super::super::recipe_resume::recipe_delivery_reference(
        &request,
        requirements_reference.as_deref(),
    ) {
        super::super::recipe_resume::RecipeDeliveryReference::Absent => {}
        super::super::recipe_resume::RecipeDeliveryReference::Bound(reference) => {
            if let Some(target) = &mut commit {
                target.reference = reference;
            }
        }
        super::super::recipe_resume::RecipeDeliveryReference::Refused(reason) => {
            return Some(AgenticPlan::Final(reason));
        }
    }
    let resume = commit
        .as_ref()
        .is_some_and(git_commit::is_pull_request_target);
    let workflow = commit
        .as_ref()
        .filter(|target| git_commit::is_pull_request_target(target))
        .filter(|_| super::super::ci_workflow::requested_in(objective))
        .and_then(|_| recipe_workflow(recipe, objective));
    let commit_step = |target: &CommitTarget| git_commit::recipe_commit_command(recipe, target);
    // Only the recipe's own commands count as recipe steps. A shell result
    // from before the recipe -- the `gh issue view` that read the work item --
    // is neither a step done nor a step failed (issue #1133).
    let mut expected_commands: Vec<String> = recipe
        .commands
        .iter()
        .cloned()
        .chain(
            commit
                .as_ref()
                .filter(|_| workflow.is_none())
                .map(commit_step),
        )
        .collect();
    if let Some(target) = &commit
        && workflow.is_none()
    {
        expected_commands.extend(git_commit::pr_completion_commands(
            recipe,
            target,
            requirements_reference
                .as_deref()
                .unwrap_or(target.reference.as_str()),
        ));
    }
    let progress =
        RecipeProgress::after_latest_user(messages, write_tool, recipe, &expected_commands, resume);
    if progress.source_invalidated {
        return Some(super::super::recipe_resume::recipe_source_refusal());
    }
    if recovery_only
        && progress
            .failure
            .as_ref()
            .is_none_or(|failure| !failure.from_run)
    {
        return None;
    }
    if let Some(target) = &commit {
        let followup = git_commit::pr_completion_commands(
            recipe,
            target,
            requirements_reference
                .as_deref()
                .unwrap_or(target.reference.as_str()),
        );
        let observed = super::super::progress::Progress::scan(messages);
        if let Some(comments) = followup.first()
            && let Some(output) = observed.latest_successful_run_output_for(comments)
            && super::super::restart_feedback::feedback_needs_changes(output)
        {
            return Some(AgenticPlan::Final(super::super::work_item_steps::fill(
                "pr_feedback_review_report",
                &[(concat!("{", "output}"), output)],
            )));
        }
    }

    let next_file = || {
        if progress.files_written == 0 {
            Some((&recipe.path, &recipe.source))
        } else {
            recipe
                .supporting_files
                .get(progress.files_written - 1)
                .map(|file| (&file.path, &file.source))
        }
    };
    if let Some(failure) = &progress.failure {
        let step = if failure.from_run {
            expected_commands
                .get(progress.commands_done)
                .map_or(write_tool, String::as_str)
        } else {
            write_tool
        };
        let failed_path = next_file().map_or(recipe.path.as_str(), |(path, _)| path);
        let mut repair_note = String::new();
        if failure.from_run {
            let observation = super::super::progress::Progress::scan(messages);
            let bound = observation.qualified_command_attempt(step);
            let owned = std::iter::once((&recipe.path, &recipe.source))
                .chain(
                    recipe
                        .supporting_files
                        .iter()
                        .map(|file| (&file.path, &file.source)),
                )
                .all(|(path, source)| recipe_file_matches(&observation, path, source));
            let observed =
                super::super::tool_result::failure_message(&failure.reported, true, false);
            let reported = observed
                .as_ref()
                .map_or(failure.reported.as_str(), String::as_str);
            let identity = if owned
                && bound.is_some_and(|frame| {
                    frame.binding == super::super::qualified_tool_observation::Binding::Exact
                }) {
                crate::coding::program_contract::program_recovery_command(
                    objective, recipe, step, reported,
                )
            } else {
                None
            };
            if recovery_only && identity.is_none() {
                return None;
            }
            if identity.is_some()
                && failure.exit_code == Some(127)
                && let (Some(target), Some(workflow)) = (&commit, &workflow)
            {
                let followup = git_commit::pr_completion_commands(
                    recipe,
                    target,
                    requirements_reference
                        .as_deref()
                        .unwrap_or(target.reference.as_str()),
                );
                let start = progress.failure_index.map_or(0, |index| index + 1);
                return Some(super::super::recipe_resume::plan_recipe_ci_fallback(
                    &messages[start..],
                    tool_names,
                    super::super::recipe_resume::RecipeCompletion {
                        recipe,
                        reference: &target.reference,
                        workflow,
                        followup: &followup,
                        local_verified: false,
                        commit_reference: requirements_reference
                            .as_deref()
                            .unwrap_or(target.reference.as_str()),
                    },
                ));
            }
            let recovery = identity.as_deref().map_or_else(
                || {
                    super::super::prerequisite_recovery::plan_recovery(
                        messages,
                        tool_names,
                        step,
                        failure.exit_code,
                        &failure.reported,
                    )
                },
                |original| {
                    super::super::prerequisite_recovery::plan_recovery_with_identity(
                        messages,
                        tool_names,
                        step,
                        failure.exit_code,
                        &failure.reported,
                        original,
                    )
                },
            );
            if let Some(plan) = recovery {
                return Some(plan);
            }
            let failed =
                super::super::repair_loop::FailedStep::new(&recipe.language, &failure.reported)
                    .with_exit_code(failure.exit_code)
                    .with_failed_command(Some(step.to_owned()))
                    .with_artifact_path(failed_path);
            let rung = progress.repair_rung.saturating_sub(1);
            let max_rungs = super::super::repair_loop::MAX_REPAIR_RUNGS;
            match super::super::repair_loop::repair_step(
                messages, tool_names, &failed, rung, max_rungs,
            ) {
                super::super::repair_loop::RepairOutcome::Stop(reason) => {
                    repair_note = super::super::repair_loop::stop_note(
                        messages, &failed, reason, rung, max_rungs,
                    );
                }
                outcome => return outcome.plan(),
            }
        }
        let mut report = failure.report(messages, failed_path, step);
        if !repair_note.is_empty() {
            report.push_str("\n\n");
            report.push_str(&repair_note);
        }
        return Some(AgenticPlan::Final(report));
    }
    if let Some((path, source)) = next_file() {
        if resume && !progress.source_checked.contains(path) {
            return Some(one_call(
                run_tool,
                json!({
                    "command": super::super::recipe_resume::recipe_source_command(path)
                })
                .to_string(),
            ));
        }
        if resume
            && let Some(before) = progress
                .source_preimages
                .get(path)
                .filter(|before| !before.absent)
        {
            let Some(command) =
                super::super::recipe_resume::recipe_source_update_command(path, source, before)
            else {
                return Some(super::super::recipe_resume::recipe_source_refusal());
            };
            return Some(one_call(run_tool, json!({"command":command}).to_string()));
        }
        return Some(one_call(write_tool, write_arguments(path, source)));
    }
    if let Some(command) = expected_commands.get(progress.commands_done) {
        return Some(one_call(
            run_tool,
            json!({ "command": command }).to_string(),
        ));
    }

    if let (Some(target), Some(workflow)) = (&commit, &workflow) {
        let followup = git_commit::pr_completion_commands(
            recipe,
            target,
            requirements_reference
                .as_deref()
                .unwrap_or(target.reference.as_str()),
        );
        let start = progress.last_command_index.map_or(0, |index| index + 1);
        return Some(super::super::recipe_resume::plan_recipe_ci_fallback(
            &messages[start..],
            tool_names,
            super::super::recipe_resume::RecipeCompletion {
                recipe,
                reference: &target.reference,
                workflow,
                followup: &followup,
                local_verified: true,
                commit_reference: requirements_reference
                    .as_deref()
                    .unwrap_or(target.reference.as_str()),
            },
        ));
    }
    let answer = recipe.final_answer(&progress.command_outputs, commit.as_ref());
    let verified = progress.command_outputs.len().min(recipe.commands.len());
    let acceptance = super::super::task_acceptance::acceptance_report(
        objective,
        recipe,
        &progress.command_outputs[..verified],
    );
    Some(AgenticPlan::Final(if acceptance.is_empty() {
        answer
    } else {
        format!("{answer}\n{acceptance}\n")
    }))
}
