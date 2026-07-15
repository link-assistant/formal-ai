use formal_ai::agentic_coding::general_planner::{compose_general_change_plan, PLAN_PATH};
use formal_ai::agentic_coding::{plan_chat_step, run_agentic_task, AgenticPlan};
use formal_ai::protocol::ChatMessage;

const EN_TASK: &str = "Create file notes/general-demo.txt containing planner fallback works";
const EN_TASK_ALT: &str = "Write file artifacts/unseen-case.md with text capability composed plan";
const RU_TASK: &str = "Создай файл output/пример.txt с текстом общий план работает";

#[test]
fn agentic_general_planner_composes_capability_steps_and_verification() {
    let plan = compose_general_change_plan(EN_TASK).expect("general plan");
    assert!(plan.steps.len() >= 3);
    assert!(plan
        .steps
        .iter()
        .all(|step| !step.expected_evidence.is_empty()));
    assert_eq!(plan.verification_command, "cat notes/general-demo.txt");
}

#[test]
fn agentic_general_planner_is_deterministic() {
    assert_eq!(
        compose_general_change_plan(EN_TASK),
        compose_general_change_plan(EN_TASK)
    );
}

#[test]
fn agentic_general_planner_accepts_three_unpinned_phrasings_in_two_languages() {
    for request in [EN_TASK, EN_TASK_ALT, RU_TASK] {
        let plan = compose_general_change_plan(request).expect(request);
        assert!(!plan.goal.is_empty());
        assert!(plan.links_notation().contains("expected_evidence"));
    }
}

#[test]
fn agentic_general_planner_rejects_unsafe_or_ambiguous_requests() {
    assert!(compose_general_change_plan("Create file ../escape.txt containing no").is_none());
    assert!(compose_general_change_plan("Please improve the repository").is_none());
}

#[test]
fn general_plan_is_emitted_before_execution() {
    let messages = vec![ChatMessage::user(EN_TASK)];
    let tools = ["write_file", "run_command"];
    let AgenticPlan::ToolCalls(calls) = plan_chat_step(&messages, &tools).expect("plan") else {
        panic!("tool call expected")
    };
    assert!(calls[0].arguments.contains(PLAN_PATH));
    assert!(calls[0].arguments.contains("general_change_plan"));
}

#[test]
fn general_task_runs_end_to_end() {
    let outcome = run_agentic_task(EN_TASK).expect("workspace");
    assert!(!outcome.hit_turn_cap);
    let tools: Vec<&str> = outcome
        .steps
        .iter()
        .map(|step| step.tool.as_str())
        .collect();
    assert_eq!(tools, ["write_file", "write_file", "run_command"]);
    assert!(outcome.steps[0].arguments.contains(PLAN_PATH));
    assert!(outcome.steps[1]
        .arguments
        .contains("notes/general-demo.txt"));
    assert!(outcome.steps[2].result.contains("planner fallback works"));
}

#[test]
fn general_change_plan_fixture_pins_the_shape() {
    let fixture = include_str!("../../data/meta/general-change-plan.lino");
    for field in [
        "goal",
        "ordered_steps",
        "capability",
        "expected_evidence",
        "verification_command",
        "append_before_execute",
    ] {
        assert!(fixture.contains(field), "missing {field}");
    }
}
