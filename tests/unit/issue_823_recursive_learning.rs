//! Regression coverage for PR #823 review feedback: recursion must execute, and
//! reported traces must feed the live, human-gated learning path.

use formal_ai::recursive_execution::{
    solve_recursively, RecursiveExecution, RecursiveRun, RecursiveTask, TaskAttempt, TaskExecutor,
};
use formal_ai::{
    learn_from_reported_conversation, learning_trace_from_symbolic_answer, ConversationTurn,
    SolverConfig, UniversalSolver,
};
use serde_json::json;

#[derive(Default)]
struct FixtureExecutor {
    parent_attempts: usize,
    extension_applied: bool,
}

impl TaskExecutor for FixtureExecutor {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        match task.id.as_str() {
            "whole" => {
                self.parent_attempts += 1;
                if self.parent_attempts == 1 {
                    TaskAttempt::failed("whole-task test failed")
                } else {
                    TaskAttempt::passed("children composed and whole-task test passed")
                }
            }
            "known-leaf" => TaskAttempt::passed("focused test passed"),
            "missing-leaf" if self.extension_applied => {
                TaskAttempt::passed("focused test passed after extension")
            }
            "missing-leaf" => TaskAttempt::failed("no method can solve this leaf"),
            other => TaskAttempt::failed(format!("unexpected task {other}")),
        }
    }

    fn extend_for(&mut self, task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        if task.id == "missing-leaf" {
            self.extension_applied = true;
            true
        } else {
            false
        }
    }
}

#[test]
fn failed_parent_shrinks_to_leaves_extends_the_missing_method_and_climbs_back_up() {
    let task = RecursiveTask::branch(
        "whole",
        "implement the whole reviewed change",
        vec![
            RecursiveTask::leaf("known-leaf", "run an already-solvable focused task"),
            RecursiveTask::leaf("missing-leaf", "solve an elementary unsupported task"),
        ],
    );
    let mut executor = FixtureExecutor::default();

    let run = solve_recursively(&task, &mut executor);

    assert_eq!(run.status, RecursiveExecution::Passed);
    assert_eq!(run.attempts.len(), 2, "the parent must be re-attempted");
    assert_eq!(run.children.len(), 2);
    assert!(run.children.iter().all(RecursiveRun::is_passed));
    let repaired_leaf = &run.children[1];
    assert!(repaired_leaf.extension_applied);
    assert_eq!(repaired_leaf.attempts.len(), 2);
    assert!(executor.extension_applied);
}

#[test]
fn a_reported_full_context_trace_enters_rule_synthesis_but_stays_review_gated() {
    let context = json!({
        "messages": [{"role": "user", "content": "List the files but sort the results in reverse order"}],
        "server_logs": [{
            "learning_trace": {
                "events": [
                    {"kind": "selected_rule", "payload": "initial unknown reason no_seed_route next try_rule_synthesis"},
                    {"kind": "rule_synthesis_candidate", "payload": "rule_synthesis_candidate\n  id reverse_sort_list_files\n  source constructed_from_operation_vocabulary\n  base_task list_files\n  modifier reverse_sort\n  resolved_task list_files_reverse_sort"},
                    {"kind": "rule_verification", "payload": "rule_verification\n  candidate reverse_sort_list_files\n  fixture list_files_output_order\n  status passed"}
                ]
            }
        }]
    });

    let staged = learn_from_reported_conversation(&context).expect("learnable report");

    assert_eq!(
        staged.trace.prompt,
        "List the files but sort the results in reverse order"
    );
    assert_eq!(staged.learning.proposals.len(), 1);
    assert_eq!(
        staged.learning.proposals[0].rule_id,
        "reverse_sort_list_files"
    );
    assert!(staged.awaiting_human_review);
    assert!(
        staged.promoted_ledger.is_none(),
        "an upload is not approval"
    );
}

#[test]
fn a_real_symbolic_candidate_round_trips_through_report_ingestion() {
    let setup = UniversalSolver::default();
    let first_prompt =
        "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
    let first = setup.solve(first_prompt);
    let first_history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let path_prompt = "Сделай так, чтобы программа принимала путь как аргумент";
    let path_answer = setup.solve_with_history(path_prompt, &first_history);
    let history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(path_prompt),
        ConversationTurn::assistant(path_answer.answer),
    ];
    let prompt = "Сделай сортировку результатов в обратном порядке";
    let candidate = setup.solve_with_history(prompt, &history);

    assert!(candidate
        .links_notation
        .contains("rule_synthesis_candidate"));
    let learning_trace = learning_trace_from_symbolic_answer(prompt, &candidate)
        .expect("verified candidates must expose structured learning metadata");
    let report = json!({
        "messages": [{"role": "user", "content": prompt}],
        "server_logs": [{"response_body": json!({"learning_trace": learning_trace}).to_string()}]
    });
    let staged = learn_from_reported_conversation(&report).expect("round-trip must be learnable");

    assert_eq!(
        staged.learning.proposals.len(),
        1,
        "trace: {learning_trace:#}\nstaged: {staged:#?}"
    );
    assert_eq!(
        staged.learning.proposals[0].rule_id,
        "reverse_sort_list_files_arg"
    );
}

#[test]
fn the_live_solver_recalls_an_approved_lesson_instead_of_rederiving_it() {
    let setup = UniversalSolver::default();
    let first_prompt = "Write a Rust program that lists files in the current directory";
    let first = setup.solve(first_prompt);
    let history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer),
    ];
    let solver = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        ..SolverConfig::default()
    });

    let response = solver.solve_with_history(
        "List the files but sort the results in reverse order",
        &history,
    );

    assert_eq!(response.intent, "write_program");
    assert!(response.answer.contains("learning_ledger_recall"));
    assert!(response.answer.contains("reverse_sort_list_files"));
    assert!(
        !response.answer.contains("rule_synthesis_candidate"),
        "approved recall must bypass re-derivation"
    );
}
