use formal_ai::agentic_coding::general_planner::{compose_general_change_plan, PLAN_PATH};
use formal_ai::agentic_coding::{plan_chat_step, run_agentic_task, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};

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
fn agentic_general_planner_rejects_non_referential_payload() {
    // A "save it to FILE" / "write this to FILE" request names no literal
    // content: the pronoun points back at content a keyword recipe must still
    // compose, so the generic write probe must decline and let that recipe win
    // (issue #663). It would otherwise fabricate a file whose body is "it".
    for request in [
        "save it to handler-precedence-learning-report.lino",
        "please write this to notes/output.txt",
    ] {
        assert!(
            compose_general_change_plan(request).is_none(),
            "non-referential payload must not compose a literal write: {request}",
        );
    }
    // A payload that merely *begins* with such a word is still literal content.
    let plan = compose_general_change_plan("write to notes/quote.txt saying to be or not to be")
        .expect("literal content beginning with a function word is still a write");
    assert_eq!(plan.content, "to be or not to be");
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
fn general_task_preserves_exact_multiline_lino_payload() {
    let payload =
        "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"reverse_sort\"";
    let task = format!("Create file data/seed/learned-program-rules.lino containing\n{payload}");

    let plan = compose_general_change_plan(&task).expect("general plan");
    assert_eq!(plan.content, payload);

    let outcome = run_agentic_task(&task).expect("agentic execution");
    let write: serde_json::Value =
        serde_json::from_str(&outcome.steps[1].arguments).expect("write arguments");
    assert_eq!(write["path"], "data/seed/learned-program-rules.lino");
    assert_eq!(write["content"], payload);
}

#[test]
fn command_stdout_requests_run_the_command_instead_of_writing_the_reference_phrase() {
    let task = "Run 'printf learned-output' and write its exact stdout to reports/learned.txt";
    let plan = compose_general_change_plan(task).expect("command-output plan");

    assert_eq!(plan.target, "reports/learned.txt");
    assert!(
        plan.content.is_empty(),
        "the referential phrase `its exact stdout` is not literal file content"
    );
    assert_eq!(
        plan.steps
            .iter()
            .map(|step| format!("{:?}", step.capability))
            .collect::<Vec<_>>(),
        ["Write", "Run", "Run"]
    );
    assert_eq!(
        plan.steps[1].command.as_deref(),
        Some("printf learned-output > 'reports/learned.txt'")
    );
}

#[test]
fn command_stdout_plan_executes_generate_then_verify_through_cli_tools() {
    let task = "Execute the auto-learning task. Run 'printf learned-output' and write its exact \
                stdout to reports/learned.txt";
    let tools = ["write", "bash"];
    let mut messages = vec![ChatMessage::user(task)];
    let mut commands = Vec::new();

    for (index, result) in [
        "wrote the plan",
        "created reports/learned.txt",
        "learned-output",
    ]
    .into_iter()
    .enumerate()
    {
        let AgenticPlan::ToolCalls(calls) =
            plan_chat_step(&messages, &tools).expect("next planned tool call")
        else {
            panic!("step {index} must be a tool call");
        };
        let call = &calls[0];
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments");
        if call.tool == "bash" {
            commands.push(arguments["command"].as_str().expect("command").to_owned());
        }
        let id = format!("command-output-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }

    assert_eq!(
        commands,
        [
            "printf learned-output > 'reports/learned.txt'",
            "cat reports/learned.txt",
        ]
    );
    assert!(matches!(
        plan_chat_step(&messages, &tools),
        Some(AgenticPlan::Final(answer)) if answer.contains("Completed the general change request")
    ));
}

#[test]
fn explicit_issue_named_file_write_is_not_misrouted_to_issue_reporting() {
    let payload =
        "learned_rules\n  id \"issue_656_agent_learning\"\n  rule \"unseen_verified_modifier\"";
    let task = format!("Create file data/seed/issue-656-agent-learned.lino containing\n{payload}");

    let outcome = run_agentic_task(&task).expect("agentic execution");
    let tools: Vec<&str> = outcome
        .steps
        .iter()
        .map(|step| step.tool.as_str())
        .collect();
    assert_eq!(tools, ["write_file", "write_file", "run_command"]);
    let write: serde_json::Value =
        serde_json::from_str(&outcome.steps[1].arguments).expect("write arguments");
    assert_eq!(write["path"], "data/seed/issue-656-agent-learned.lino");
    assert_eq!(write["content"], payload);
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

/// A **read** request must never compose a write plan (found by the issue-#671
/// matrix on the `opencode` leg).
///
/// `formal-ai with opencode "read the file alpha.txt and print its contents"`
/// planned `write(alpha.txt)` and overwrote the fixture with a stray quote
/// character before "verifying" it with `cat alpha.txt`. The marker-led branch
/// of the write recogniser accepted `file alpha.txt` (a positional target cue)
/// plus the trailing content lead `contents`, with no write action cue anywhere
/// in the request — so a request whose only verbs were `read` and `print`
/// destroyed the file it was asked to read. Codex escaped only because its
/// toolset advertises no plain write tool, which is exactly the "inferred from
/// the shared adapters" reasoning issue #671 exists to reject.
#[test]
fn agentic_general_planner_never_claims_a_read_request() {
    for request in [
        "read the file alpha.txt and print its contents",
        "\"read the file alpha.txt and print its contents\"",
        "show me the contents of the file beta.md",
        "print the contents of subdir/nested.log",
    ] {
        assert!(
            compose_general_change_plan(request).is_none(),
            "a read request must not compose a write plan: {request}",
        );
    }
}

/// Content recovered from a request has to be *content*. The `opencode` leg's
/// overwrite payload was a single `"` — the tail left over after the trailing
/// content-lead marker in a quoted prompt. A payload with nothing to say is a
/// mis-parse, not a write.
#[test]
fn agentic_general_planner_rejects_punctuation_only_payload() {
    for request in [
        "write file notes/empty.txt containing \"",
        "create file notes/dots.txt with text ...",
    ] {
        assert!(
            compose_general_change_plan(request).is_none(),
            "a payload with no word characters must not compose a write: {request}",
        );
    }
}
