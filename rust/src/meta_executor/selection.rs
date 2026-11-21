use super::*;

pub(super) fn select(
    node: &TaskNode,
    test: &TaskTest,
    child_outcomes: &[TaskOutcome],
    candidates: &mut [Candidate],
    budget_exhausted: bool,
    state: &mut RunState,
) -> TaskOutcome {
    let mut ranked = Vec::with_capacity(candidates.len());
    for candidate in candidates.iter_mut() {
        let verdict = run_task_test(test, Some(&candidate.draft), child_outcomes);
        candidate.passed = verdict.passed;
        candidate.code_units = count_chars(&candidate.draft.content);
        emit(
            &mut state.events,
            "meta_exec:draft",
            &[
                ("task", node.id.clone()),
                ("method", candidate.method.to_owned()),
                ("steps", candidate.steps.to_string()),
                ("code_units", candidate.code_units.to_string()),
                ("executed", verdict.executed.join(",")),
                ("passed", verdict.passed.to_string()),
                ("failed", verdict.failed.unwrap_or_else(|| "-".to_owned())),
            ],
        );
        ranked.push(SolutionCandidate {
            id: candidate.method.to_owned(),
            solves_entire_range: verdict.passed,
            cost: ActionCost {
                steps: candidate.steps,
                code_units: candidate.code_units,
                compute_ms: 0,
                memory_kb: 0,
            },
        });
    }
    if budget_exhausted {
        emit(
            &mut state.events,
            "meta_exec:draft",
            &[
                ("task", node.id.clone()),
                ("method", "derivation".to_owned()),
                ("skipped", "budget".to_owned()),
            ],
        );
    }
    let order = rank_by_least_action(&ranked);
    let fallback = candidates
        .iter()
        .find(|candidate| candidate.method != "composition" && candidate.method != "cache")
        .cloned();
    let Some(best) = order.first() else {
        let reason = if candidates.is_empty() {
            if budget_exhausted {
                "budget"
            } else {
                "no_draft"
            }
        } else if !node.children.is_empty() && !children_satisfy(test, child_outcomes) {
            "children"
        } else {
            "no_passing_draft"
        };
        emit(
            &mut state.events,
            "meta_exec:unresolved",
            &[("task", node.id.clone()), ("reason", reason.to_owned())],
        );
        return TaskOutcome {
            task: node.id.clone(),
            status: "unresolved",
            reason: Some(reason),
            candidate: None,
            fallback,
        };
    };
    let selected = candidates
        .iter()
        .find(|candidate| candidate.method == best.id && candidate.passed)
        .cloned()
        .expect("a ranked candidate is a passing candidate");
    emit(
        &mut state.events,
        "meta_exec:select",
        &[
            ("task", node.id.clone()),
            ("method", selected.method.to_owned()),
            ("reason", "least_action".to_owned()),
            ("steps", selected.steps.to_string()),
            ("code_units", selected.code_units.to_string()),
            ("passing", format!("{}/{}", order.len(), candidates.len())),
        ],
    );
    state.memo.insert(node.text.clone(), selected.draft.clone());
    TaskOutcome {
        task: node.id.clone(),
        status: "resolved",
        reason: None,
        candidate: Some(selected),
        fallback,
    }
}
