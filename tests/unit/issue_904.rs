//! Issue #904 — a repository work item must not be reduced to writing a plan
//! file and reading it back, and its goal must not be the caller's preamble.
//!
//! Agent mode answered a repository coding task with a two-step plan whose only
//! steps wrote `.formal-ai/general-change-plan.lino` and `cat`-ed it back. The
//! verification command therefore observed the file the run itself had just
//! written, so it passed by construction and the run terminated in a *success*
//! state with the working tree untouched. The same plan's `goal` field held the
//! caller's entire system-prompt preamble ("You are an AI issue solver …")
//! rather than the objective the caller stated after it.
//!
//! The three requirements of the issue, one test each:
//!
//! 1. the goal is taken from the objective, not from the harness preamble;
//! 2. verification never observes only the plan record the run wrote itself;
//! 3. a plan with no step touching the requested artefact ends in a
//!    "planned, not executed" state instead of a success state.

use formal_ai::agentic_coding::general_planner::{
    compose_general_change_plan, objective_text, GeneralPlanMode, PlanTerminalState, PLAN_PATH,
};
use formal_ai::agentic_coding::{run_agentic_task, AgenticPlan};
use formal_ai::protocol::ChatMessage;

/// The prompt shape from the issue: a harness system-prompt preamble, then the
/// caller's objective introduced by an explicit lead.
const HARNESS_PROMPT: &str = "\
You are an AI issue solver using @link-assistant/agent.
General guidelines.
   - When you execute commands and the output becomes large, save the logs to files.
   - When you test assumptions, keep experiment scripts in ./experiments.
Issue to solve: https://github.com/link-assistant/formal-ai/issues/904
Implement the fix in the repository and verify it with tests.";

#[test]
fn plan_goal_is_the_objective_not_the_caller_preamble() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");

    assert_eq!(plan.mode, GeneralPlanMode::RepositoryWorkItem);
    assert!(
        !plan.goal.contains("You are an AI issue solver"),
        "the goal must not hold the caller's system-prompt preamble: {}",
        plan.goal
    );
    assert!(
        plan.goal
            .contains("https://github.com/link-assistant/formal-ai/issues/904"),
        "the goal must state the objective the caller wrote after the lead: {}",
        plan.goal
    );
    assert!(
        !plan.links_notation().contains("You are an AI issue solver"),
        "the serialized plan must not carry the preamble either",
    );
}

#[test]
fn objective_lead_marks_the_boundary_between_preamble_and_objective() {
    assert_eq!(
        objective_text(HARNESS_PROMPT),
        "https://github.com/link-assistant/formal-ai/issues/904\n\
         Implement the fix in the repository and verify it with tests.",
    );
    // Without a documented lead there is nothing to separate, so the whole
    // request stays the objective.
    let plain = "Create file notes/plain.txt containing hello";
    assert_eq!(objective_text(plain), plain);
}

#[test]
fn repository_work_item_plan_never_verifies_only_its_own_plan_record() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");

    assert!(
        plan.verification_command.is_empty(),
        "a command that reads back the plan the run just wrote proves nothing: {}",
        plan.verification_command
    );
    assert!(
        !plan.steps.iter().any(|step| step
            .command
            .as_deref()
            .is_some_and(|command| command.contains(PLAN_PATH))),
        "no step may verify the run by observing the plan record it wrote itself",
    );
}

#[test]
fn repository_work_item_plan_ends_in_planned_not_executed() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");

    assert_eq!(plan.terminal_state, PlanTerminalState::PlannedNotExecuted);
    assert!(
        plan.links_notation().contains("planned_not_executed"),
        "the serialized plan must record the terminal state it reached",
    );
}

#[test]
fn literal_file_plans_still_execute_and_verify_the_requested_artifact() {
    let plan = compose_general_change_plan("Create file notes/general-demo.txt containing hello")
        .expect("literal file plan");

    assert_eq!(plan.terminal_state, PlanTerminalState::Executed);
    assert_eq!(plan.verification_command, "cat notes/general-demo.txt");
}

/// Whole-task test: the run described by the issue, end to end.
#[test]
fn repository_work_item_run_reports_planned_not_executed_instead_of_success() {
    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HARNESS_PROMPT)],
        &["write_file", "run_command"],
    )
    .expect("the harness prompt must have an agentic plan") else {
        panic!("the first step must persist the plan record")
    };
    assert_eq!(calls[0].tool, "write_file");

    let outcome = run_agentic_task(HARNESS_PROMPT).expect("Agent CLI replay");
    assert!(!outcome.hit_turn_cap);
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["write_file"],
        "the run must not add a step that only reads its own plan back",
    );
    assert!(
        !outcome.final_answer.contains("Recorded and verified"),
        "a run that changed no requested artefact must not read as a success: {}",
        outcome.final_answer
    );
    assert!(
        outcome.final_answer.contains("Planned"),
        "the run must end in a planned, not executed state: {}",
        outcome.final_answer
    );
    assert!(
        !outcome.final_answer.contains("You are an AI issue solver"),
        "the answer must restate the objective, not the caller's preamble",
    );
}
