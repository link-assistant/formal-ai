use formal_ai::agentic_coding::general_planner::{PLAN_PATH, compose_general_change_plan};
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step, run_agentic_task};
use formal_ai::protocol::{ChatMessage, ToolCall};

use crate::observed_plan_tools;

const EN_TASK: &str = "Create file notes/general-demo.txt containing planner fallback works";
const EN_TASK_ALT: &str = "Write file artifacts/unseen-case.md with text capability composed plan";
const RU_TASK: &str = "Создай файл output/пример.txt с текстом общий план работает";
const ISSUE_698_WORK_ITEM: &str = "\
You are an AI issue solver using OpenAI Codex.
Issue to solve: https://github.com/link-assistant/formal-ai/issues/698
Continue pull request https://github.com/link-assistant/formal-ai/pull/816.
Implement a real external benchmark harness with pinned permissive upstream suites, honest \
passed/failed totals, a scheduled committed ledger, and a monotonic per-suite ratchet.
Record benchmark_unavailable instead of proxy scores, grade SWE-bench with its official tests, \
and derive review-gated learning only from observed failures.
Read the repository, reproduce the requirement with tests, implement the change, and verify it \
through the Formal AI Agent CLI.";

#[test]
fn agentic_general_planner_composes_capability_steps_and_verification() {
    let plan = compose_general_change_plan(EN_TASK).expect("general plan");
    assert!(plan.steps.len() >= 3);
    assert!(
        plan.steps
            .iter()
            .all(|step| !step.expected_evidence.is_empty())
    );
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
        assert_ne!(plan.goal, "");
        assert!(plan.links_notation().contains("expected_evidence"));
    }
}

#[test]
fn agentic_general_planner_rejects_unsafe_or_ambiguous_requests() {
    assert!(compose_general_change_plan("Create file ../escape.txt containing no").is_none());
    assert!(compose_general_change_plan("Please improve the repository").is_none());
}

#[test]
fn agentic_general_planner_does_not_treat_dot_relative_directories_as_files() {
    let issue_policy = "When you create debug or example scripts while fixing an issue, \
                        keep them in ./examples and/or ./experiments. Continue with the \
                        following investigation details and solve the issue completely.";

    assert!(
        compose_general_change_plan(issue_policy).is_none(),
        "a dot-relative policy directory must not become a literal file-write target",
    );
}

#[test]
fn compound_github_work_item_routes_to_agentic_planning_before_project_lookup() {
    let messages = vec![ChatMessage::user(ISSUE_698_WORK_ITEM)];
    let tools = ["web_search", "web_fetch", "write_file", "run_command"];
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &tools).expect("compound work item must have an agentic plan")
    else {
        panic!("the first work-item step must read the work item")
    };

    assert_eq!(calls.len(), 1);
    // A work item names an issue, and an issue URL names no artifact, so the
    // run reads the work item before deciding what it can execute (issue #904,
    // follow-up). A protocol-hosted fetch tool runs the read server-side
    // (issue #1133), so it comes first when the client has one; the structured
    // `gh` read is the path for shell-only clients. The plan record is still
    // written; it is no longer the first and only thing the run does.
    assert_eq!(calls[0].tool, "web_fetch");
    assert!(calls[0].arguments.contains("issues/698"), "{calls:?}");

    // A client with a shell but no fetch tool reads the work item through
    // `gh issue view` before anything is recorded (issue #1133).
    let write_only = plan_chat_step(&messages, &["write_file", "run_command"]);
    let Some(AgenticPlan::ToolCalls(write_only)) = write_only else {
        panic!("a write-only client still reads the work item")
    };
    assert_eq!(write_only[0].tool, "run_command");
    assert!(
        write_only[0].arguments.contains("gh issue view"),
        "{write_only:?}"
    );
    // An empty read is a failed read (issue #1155): the REST fallbacks are
    // walked next, and once every route has answered nothing the run closes
    // with the reads it tried instead of spending its write on a plan record
    // for an issue it never saw.
    let mut after_read = messages;
    let mut planned = vec![write_only[0].clone()];
    let report = loop {
        let step = planned.len();
        let call = &planned[step - 1];
        after_read.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            format!("c{step}"),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        after_read.push(ChatMessage::tool_result(format!("c{step}"), &call.tool, ""));
        match plan_chat_step(&after_read, &["write_file", "run_command"]) {
            Some(AgenticPlan::ToolCalls(calls)) if step < 6 => planned.push(calls[0].clone()),
            Some(AgenticPlan::Final(report)) => break report,
            other => panic!("the exhausted read closes with a report, got {other:?}"),
        }
    };
    assert!(
        planned
            .iter()
            .any(|call| call.tool == "run_command" && call.arguments.contains("curl -fsSL")),
        "the credential-free REST read is a fallback: {planned:?}"
    );
    assert!(
        planned
            .iter()
            .all(|call| call.tool == "run_command" && !call.arguments.contains(PLAN_PATH)),
        "no plan record is written for an unread work item: {planned:?}"
    );
    assert!(report.contains("could not be read"), "{report}");

    let outcome = run_agentic_task(ISSUE_698_WORK_ITEM).expect("Agent CLI work-item replay");
    assert!(!outcome.hit_turn_cap);
    // Two steps: the run reads the work item through its hosted fetch and
    // records it. Issue #904 removed the `cat .formal-ai/general-change-plan.lino`
    // step, which verified nothing but the write the same run had just
    // performed, and that step has not come back; the hosted-fetch read also
    // replaced the `gh` plan the driver's default-deny sandbox refused.
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["web_fetch", "read_file", "write_file", "read_file"]
    );
    assert!(outcome.steps[1].arguments.contains(PLAN_PATH));
    assert!(outcome.final_answer.contains("Planned, not executed"));
    assert!(!outcome.final_answer.contains("project lookup"));
}

#[test]
fn repository_work_item_completion_is_seeded_for_every_supported_language() {
    struct LocalizedCase {
        language: &'static str,
        expected_fragment: &'static str,
    }

    for case in [
        LocalizedCase {
            language: "en",
            expected_fragment: "Planned, not executed",
        },
        LocalizedCase {
            language: "ru",
            expected_fragment: "Запланировано, но не выполнено",
        },
        LocalizedCase {
            language: "hi",
            expected_fragment: "योजना बनी, निष्पादित नहीं",
        },
        LocalizedCase {
            language: "zh",
            expected_fragment: "已规划，未执行",
        },
    ] {
        let response =
            formal_ai::seed::response_for("general_plan_repository_planned", case.language)
                .unwrap_or_else(|| panic!("missing repository outcome for {}", case.language));
        assert!(
            response.contains(case.expected_fragment),
            "repository completion is not localized for {}: {response}",
            case.language
        );
    }
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
    assert_eq!(
        tools,
        [
            "read_file",
            "write_file",
            "read_file",
            "write_file",
            "run_command"
        ]
    );
    assert!(outcome.steps[0].arguments.contains(PLAN_PATH));
    assert!(
        outcome.steps[3]
            .arguments
            .contains("notes/general-demo.txt")
    );
    assert!(outcome.steps[4].result.contains("planner fallback works"));
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
        serde_json::from_str(&outcome.steps[3].arguments).expect("write arguments");
    assert_eq!(write["path"], "data/seed/learned-program-rules.lino");
    assert_eq!(write["content"], payload);
}

#[test]
fn bare_with_preserves_an_exact_backticked_multiline_payload() {
    let payload = "coding_discovery_recipe\n  coding_discovery_step_verify";
    let task = format!("Create file data/meta/coding-discovery-recipe.lino with `{payload}`");

    let plan = compose_general_change_plan(&task).expect("literal file plan");
    assert_eq!(plan.target, "data/meta/coding-discovery-recipe.lino");
    assert_eq!(plan.content, payload);

    let messages = vec![ChatMessage::user(&task)];
    let tools = ["write", "grep", "read", "bash"];
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &tools).expect("literal file plan must own the request")
    else {
        panic!("the literal file plan must persist its plan before execution")
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "bash");
    assert!(
        calls[0].arguments.contains(PLAN_PATH),
        "the evidence-record route must not search for the payload before it is created: {}",
        calls[0].arguments,
    );
}

#[test]
fn literal_file_marker_owns_payload_that_contains_an_edit_phrase() {
    let payload = "prefix rename X to Y suffix";
    let task =
        format!("Create file issue_708_memory_program.rs with exactly this content:\n{payload}");

    let plan = compose_general_change_plan(&task).expect("literal file plan");
    assert_eq!(plan.content, payload);
}

#[test]
fn literal_file_marker_routes_before_edit_shaped_payload() {
    let payload = "prefix rename X to Y suffix";
    let tools = ["write", "read", "edit", "bash"];

    for task in [
        format!("Create file issue-708-literal-payload.txt with exactly this content:\n{payload}"),
        format!("Создай файл issue-708-literal-payload.txt с точно таким содержанием:\n{payload}"),
        format!("फ़ाइल issue-708-literal-payload.txt ठीक इसी सामग्री के साथ बनाओ:\n{payload}"),
        format!("创建 文件 issue-708-literal-payload.txt 内容与以下完全相同：\n{payload}"),
    ] {
        let messages = vec![ChatMessage::user(&task)];
        let AgenticPlan::ToolCalls(calls) =
            plan_chat_step(&messages, &tools).expect("literal file plan must own the request")
        else {
            panic!("literal file plan must persist its plan before execution: {task}")
        };
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool, "bash", "{task}");
        assert!(
            calls[0].arguments.contains(PLAN_PATH),
            "edit-shaped payload must not route to a read of the new target: {}",
            calls[0].arguments,
        );
    }
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
        Some(
            "formal_ai_capture_target='reports/learned.txt'; formal_ai_capture_parent=${formal_ai_capture_target%/*}; if [ \"$formal_ai_capture_parent\" != \"$formal_ai_capture_target\" ]; then mkdir -p -- \"$formal_ai_capture_parent\" || exit $?; fi; ( printf learned-output ) > \"$formal_ai_capture_target\""
        )
    );
}

#[test]
fn command_stdout_plan_executes_generate_then_verify_through_cli_tools() {
    let task = "Execute the auto-learning task. Run 'printf learned-output' and write its exact \
                stdout to reports/learned.txt";
    let plan = compose_general_change_plan(task).expect("original stdout plan");
    let tools = ["read", "write", "run_command"];
    let mut messages = vec![ChatMessage::user(task)];
    let mut commands = Vec::new();
    let mut workspace = observed_plan_tools::ToolWorkspace::new(task);
    for index in 0..8 {
        let step = plan_chat_step(&messages, &tools).expect("planned step");
        let AgenticPlan::ToolCalls(calls) = step else {
            break;
        };
        let call = &calls[0];
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments");
        let id = format!("command-output-{index}");
        let command = arguments["command"].as_str();
        let auxiliary = call.tool == "run_command"
            && command != plan.steps[1].command.as_deref()
            && command != Some(plan.verification_command.as_str());
        let observation = if auxiliary {
            observed_plan_tools::observe_append(&mut workspace, &id, call, task)
        } else {
            if let Some(command) = command {
                commands.push(command.to_owned());
            }
            workspace.execute(&id, call)
        };
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(observation);
    }

    assert_eq!(
        workspace
            .read("reports/learned.txt")
            .expect("physical stdout target"),
        "learned-output"
    );
    assert_eq!(
        commands,
        [
            "formal_ai_capture_target='reports/learned.txt'; formal_ai_capture_parent=${formal_ai_capture_target%/*}; if [ \"$formal_ai_capture_parent\" != \"$formal_ai_capture_target\" ]; then mkdir -p -- \"$formal_ai_capture_parent\" || exit $?; fi; ( printf learned-output ) > \"$formal_ai_capture_target\"",
            "cat 'reports/learned.txt'",
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
    assert_eq!(
        tools,
        [
            "read_file",
            "write_file",
            "read_file",
            "write_file",
            "run_command"
        ]
    );
    let initial: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("initial read");
    assert_eq!(initial["path"], PLAN_PATH);
    let event: serde_json::Value =
        serde_json::from_str(&outcome.steps[1].arguments).expect("stored event");
    assert_eq!(event["path"], PLAN_PATH);
    let tree = formal_ai::seed::parse_lino(event["content"].as_str().expect("whole event"));
    assert_eq!(tree.children[0].find_child_value("goal"), task);
    let observed: serde_json::Value =
        serde_json::from_str(&outcome.steps[2].arguments).expect("event readback");
    assert_eq!(observed["path"], PLAN_PATH);
    assert!(outcome.steps[2].result.starts_with("<file>\n"));
    assert!(
        outcome.steps[2]
            .result
            .contains(tree.children[0].find_child_value("id"))
    );
    let receipt: serde_json::Value =
        serde_json::from_str(&outcome.steps[4].result).expect("actual target readback");
    assert_eq!(receipt["schema"], "command-execution-receipt/v1");
    assert_eq!(
        receipt["command"],
        "cat data/seed/issue-656-agent-learned.lino"
    );
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["truncated"], false);
    assert_eq!(receipt["timed_out"], false);
    assert_eq!(receipt["stdout"], payload);
    let write: serde_json::Value =
        serde_json::from_str(&outcome.steps[3].arguments).expect("write arguments");
    assert_eq!(write["path"], "data/seed/issue-656-agent-learned.lino");
    assert_eq!(write["content"], payload);
}

#[test]
fn general_change_plan_fixture_pins_the_shape() {
    let fixture = include_str!("../../../../data/meta/general-change-plan.lino");
    for field in [
        "goal",
        "ordered_steps",
        "capability",
        "expected_evidence",
        "verification_command",
        "append_before_execute",
        "terminal_state",
        "no_self_verification",
        "planned_not_executed",
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

/// A multi-line payload is written whole, whichever line the marker ends on.
///
/// The content lead alone on its line already introduced the whole block below
/// it. The same request with the payload starting on the marker's own line means
/// exactly the same thing, but the sentence bound cut it at the first line: a
/// 1478-byte document was written as its 58-byte title, and `alpha\nbeta\ngamma`
/// was written as `alpha`. Nothing in the request said the rest would be
/// dropped, and the write reported success, so the self-authoring loop produced
/// one-line stubs of the documents it was handed and looked broken for a reason
/// that was never in its own logs.
///
/// Both spellings are pinned to the identical result, so neither can drift back
/// into truncating the other. The one-line payload keeps the sentence bound --
/// "write the following: hello to `x.txt`" must still stop at the clause.
#[test]
fn a_multi_line_payload_survives_a_marker_that_shares_its_line() {
    let body = "alpha\nbeta\ngamma";
    let inline = compose_general_change_plan(&format!(
        "Create a file out.md with exactly this content: {body}"
    ))
    .expect("an inline multi-line payload must compose a plan");
    let own_line = compose_general_change_plan(&format!(
        "Create a file out.md with exactly this content:\n{body}"
    ))
    .expect("a marker-led block must compose a plan");

    assert_eq!(
        inline.content, body,
        "the payload must survive whole when the marker shares its first line"
    );
    assert_eq!(
        inline.content, own_line.content,
        "the two spellings of one request must deliver the same bytes"
    );

    // A payload that never leaves its line is unaffected: there is no
    // continuation to widen to, so it still stops where it always did.
    let bounded = compose_general_change_plan(EN_TASK)
        .expect("a single-line payload must still compose a plan");
    assert_eq!(
        bounded.content, "planner fallback works",
        "a payload that stays on one line is bounded exactly as before"
    );
}

#[test]
fn command_capture_parent_file_is_a_real_failure_without_false_completion() {
    let prompt = "Run 'printf learned-output' and write its exact stdout to reports/learned.txt";
    let plan = compose_general_change_plan(prompt).expect("declared output plan");
    let mut workspace = observed_plan_tools::ToolWorkspace::new(prompt);
    workspace
        .write_initial("reports", "existing file")
        .expect("real obstructing parent");
    let tools = ["read", "write", "run_command"];
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..8 {
        match plan_chat_step(&messages, &tools).expect("planned output task") {
            AgenticPlan::ToolCalls(calls) => {
                let call = &calls[0];
                let id = format!("blocked-parent-{turn}");
                let observation = workspace.execute(&id, call);
                let failed = observation.is_error;
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    &id,
                    &call.tool,
                    call.arguments.clone(),
                )]));
                messages.push(observation);
                if failed {
                    assert!(workspace.read(&plan.target).is_err());
                    assert!(
                        matches!(plan_chat_step(&messages, &tools), Some(AgenticPlan::Final(answer)) if !answer.contains("Completed the general change request"))
                    );
                    return;
                }
            }
            AgenticPlan::Final(_) => {
                panic!("the blocked parent must be observed as a failed tool call")
            }
        }
    }
    panic!("bounded real capture did not observe the parent error");
}
