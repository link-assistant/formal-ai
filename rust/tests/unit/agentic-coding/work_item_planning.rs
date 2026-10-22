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
    GeneralPlanMode, PLAN_PATH, PlanTerminalState, compose_general_change_plan, objective_text,
};
use formal_ai::agentic_coding::{AgenticPlan, run_agentic_task};
use formal_ai::protocol::ChatMessage;

use crate::tool_workspace;

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

/// The "planned, not executed" answer is user-visible text, so every registered
/// language must state it — an English-only honesty message would let the same
/// run read as a success in the other languages.
#[test]
fn every_registered_language_states_planned_not_executed() {
    // english, russian, hindi, chinese, spanish — the registered matrix.
    let leads = [
        ("en", "Planned, not executed"),
        ("ru", "Запланировано, но не выполнено"),
        ("hi", "योजना बनी, निष्पादित नहीं"),
        ("zh", "已规划，未执行"),
        ("es", "Planificado, no ejecutado"),
    ];

    for language in formal_ai::language::registered_languages() {
        let slug = language.slug();
        let text = formal_ai::seed::localized_response("general_plan_repository_planned", slug)
            .unwrap_or_else(|| panic!("{slug} planned-not-executed response"));
        let (_, lead) = leads
            .iter()
            .find(|(candidate, _)| *candidate == slug)
            .unwrap_or_else(|| panic!("{slug} is registered but has no expected lead"));

        assert!(text.starts_with(lead), "{slug} lead: {text}");
        assert!(
            !text.contains("{plan_path}"),
            "{slug} must not imply an auxiliary journal: {text}"
        );
        for placeholder in ["{target}", "{plan}"] {
            assert!(
                text.contains(placeholder),
                "{slug} answer must name {placeholder}: {text}"
            );
        }
    }
}

/// Whole-task test: the run described by the issue, end to end.
#[test]
fn repository_work_item_run_reports_planned_not_executed_instead_of_success() {
    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HARNESS_PROMPT)],
        &["write_file", "run_command"],
    )
    .expect("the harness prompt must have an agentic plan") else {
        panic!("the first step must read the work item")
    };
    // A shell is a way to read the issue (`gh issue view …`), so a client with
    // no fetch tool reads it there before anything is recorded (issue #1133);
    // the record follows once the read has been tried.
    assert_eq!(calls[0].tool, "run_command");
    assert!(calls[0].arguments.contains("gh issue view"), "{calls:?}");

    let outcome = run_agentic_task(HARNESS_PROMPT).expect("Agent CLI replay");
    assert!(!outcome.hit_turn_cap);
    // The exact original request is still missing from the offline corpus.
    // A known failed provider read must not be recorded as a retrieved goal.
    assert!(outcome.steps.iter().all(|step| step.tool != "write_file"));
    assert!(outcome.steps.iter().any(|step| step.tool == "web_fetch"));
    assert!(
        !outcome.final_answer.contains("Recorded and verified"),
        "a run that changed no requested artefact must not read as a success: {}",
        outcome.final_answer
    );
    assert!(
        outcome
            .final_answer
            .contains("https://github.com/link-assistant/formal-ai/issues/904"),
        "the unavailable-source report must identify the original issue: {}",
        outcome.final_answer
    );
    assert!(
        !outcome.final_answer.contains("You are an AI issue solver"),
        "the answer must restate the objective, not the caller's preamble",
    );
}

// ---------------------------------------------------------------------------
// The follow-up: honest is not the same as done
// ---------------------------------------------------------------------------
//
// The terminal state above is truthful, and it stayed truthful — but three
// production matrices later, no repository work had ever been executed
// ([comment](https://github.com/link-assistant/formal-ai/issues/904#issuecomment-5302919859)):
// Agent + Scala, Claude + Kotlin and Codex + Rust each ended `planned_not_executed`
// with an empty pull request after five automatic restarts.
//
// The reason is visible in the plan: a work item names an *issue*, and an issue
// URL names no artifact. Recording the reference and stopping was the only honest
// end available, because the run never read the one document that says what to
// build. So the fix is not to loosen the terminal state — it is to read the work
// item first, and keep `planned_not_executed` for the case the comment's point 3
// reserves it for: the required capability is genuinely unavailable.

/// The work item the fetch step retrieves, in the shape a client returns it.
const FETCHED_WORK_ITEM: &str = "\
Add a greeting file

Create file greeting.txt containing Hello from the work item";

/// The bounded objective Hive Mind 2.12.5 actually sends, byte-for-byte from
/// `buildFormalAiRepositoryPrompt` in
/// [hive-mind#2159](https://github.com/link-assistant/hive-mind/pull/2159).
///
/// The #2158 fix stripped Hive Mind's workflow policy out of this request, so
/// there is no preamble left to separate and no delimiter to lean on — which is
/// exactly why the remaining defect is Formal AI's. This shape is what
/// production sends, so it is what the follow-up has to be measured against.
const HIVE_MIND_BOUNDED_OBJECTIVE: &str = "\
Resolve the GitHub issue at https://github.com/example/example/issues/1 in this repository.
Keep the solution on branch issue-1-example.
Update the pull request at https://github.com/example/example/pull/2.

Implement and verify the solution before reporting completion.
Proceed.
";

/// The production request routes to a repository work item and reads it, with
/// no objective delimiter involved.
#[test]
fn the_bounded_hive_mind_objective_reads_the_issue_it_names() {
    let plan = compose_general_change_plan(HIVE_MIND_BOUNDED_OBJECTIVE)
        .expect("the bounded Hive Mind objective must compose a repository work-item plan");

    assert_eq!(plan.mode, GeneralPlanMode::RepositoryWorkItem);
    assert_eq!(
        plan.target, "https://github.com/example/example/issues/1",
        "the work item is the issue, not the branch or the pull request",
    );

    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HIVE_MIND_BOUNDED_OBJECTIVE)],
        &["fetch_url", "write_file", "run_command"],
    )
    .expect("the production request must have an agentic plan") else {
        panic!("the production request must read the issue it names")
    };
    assert_eq!(calls[0].tool, "run_command");
    assert!(
        calls[0].arguments.contains(&plan.target),
        "the structured read must address the issue Hive Mind named: {}",
        calls[0].arguments,
    );
}

/// Requirement 1 of the follow-up: reading the work item comes before concluding
/// that nothing can be executed.
#[test]
fn a_repository_work_item_is_read_before_anything_is_concluded() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");

    assert_eq!(
        plan.steps.first().map(|step| step.capability),
        Some(formal_ai::agentic_coding::planner::Capability::Fetch),
        "the first step must read the work item the request named",
    );
    assert!(
        plan.steps[0].action.contains(&plan.target),
        "the read step must name the work item it reads: {}",
        plan.steps[0].action,
    );

    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HARNESS_PROMPT)],
        &["fetch_url", "write_file", "run_command"],
    )
    .expect("the harness prompt must have an agentic plan") else {
        panic!("the first step must read the work item")
    };
    assert_eq!(calls[0].tool, "run_command");
    assert!(
        calls[0].arguments.contains(&plan.target),
        "the structured read must address the work item, not some other page: {}",
        calls[0].arguments,
    );
}

/// The three work items the production matrix was dispatched against, abridged
/// from the reproduction repositories linked in
/// [hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158).
///
/// This is what an issue really looks like: a described artifact in a named
/// language, not literal bytes. Reading it is worth nothing if the text cannot
/// then be routed, so the corpus decides which route runs first — the
/// seed-backed source route, then the literal-file composer.
fn hello_world_work_item(language: &str) -> String {
    format!(
        "Implement Hello World in {language}\n\n\
         ## Task\n\
         Please implement a \"Hello World\" program in {language}.\n\n\
         ## Expected Output\n\
         When the program runs, it should output:\n\n\
         Hello, World!"
    )
}

/// Requirement 2 of the follow-up, against the real corpus: a described
/// artifact in a *supported* language reaches a route that authors it instead
/// of stopping at the plan record.
///
/// Rust is the Codex leg of the production matrix — the one whose language the
/// catalog already covers, and the leg that got furthest before the separate
/// #907 `sudo` hijack stopped it.
#[test]
fn a_read_work_item_describing_a_program_reaches_an_authoring_route() {
    let messages = work_item_transcript(&hello_world_work_item("Rust"));
    let plan = formal_ai::agentic_coding::plan_chat_step(
        &messages,
        &["fetch_url", "write_file", "run_command"],
    )
    .expect("a read work item must have a plan");

    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("a described program must be authored, not narrated")
    };
    assert_eq!(
        calls[0].tool, "write_file",
        "the run must write the program the work item described",
    );
    assert!(
        !calls[0].arguments.contains(PLAN_PATH),
        "the plan record is not the artifact the work item asked for: {}",
        calls[0].arguments,
    );
}

/// The honest half of the same corpus: reading a work item cannot invent a
/// language the catalog does not have, so one naming an absent language still
/// ends `planned_not_executed` — truthfully, and for the reason
/// [hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158)
/// point 3 reserves that state for.
///
/// The production matrix's own two languages, Scala and Kotlin, were absent
/// when this was written and are covered now, so the rule is checked against
/// languages that really are missing. Adding either of these is what should
/// make this test fail — the failure is the signal to move it on, not to
/// weaken it.
#[test]
fn an_uncovered_language_is_not_invented_into_an_artifact() {
    for language in ["Haskell", "Elixir"] {
        let messages = work_item_transcript(&hello_world_work_item(language));
        let plan = formal_ai::agentic_coding::plan_chat_step(
            &messages,
            &["fetch_url", "write_file", "run_command"],
        );

        if let Some(AgenticPlan::ToolCalls(calls)) = plan {
            for call in &calls {
                assert!(
                    call.arguments.contains(PLAN_PATH),
                    "{language} is not in the coding catalog, so the only file this run may \
                     write is its own plan record: {}",
                    call.arguments,
                );
            }
        }
    }
}

/// Requirement 2 of the follow-up: once the work item has been read, the run
/// executes what it names — the artifact, not another plan record.
#[test]
fn a_read_work_item_produces_the_artifact_it_names() {
    let messages = work_item_transcript(FETCHED_WORK_ITEM);
    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &messages,
        &["fetch_url", "write_file", "run_command"],
    )
    .expect("a read work item must have an executable plan") else {
        panic!("the run must act on the artifact the work item named")
    };

    assert_eq!(calls[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("write arguments");
    let path = ["path", "file_path", "filePath"]
        .iter()
        .find_map(|key| arguments.get(*key).and_then(serde_json::Value::as_str))
        .expect("the write must name a path");
    assert!(
        path == PLAN_PATH || path == "greeting.txt",
        "the run must address the work item's own artifact, got {path}",
    );
}

/// Requirement 3 of the follow-up, and the guard on it: `planned_not_executed`
/// survives exactly where the comment says it should — the capability needed to
/// read the work item is not available — and nowhere else.
#[test]
fn planned_not_executed_remains_only_when_the_work_item_cannot_be_read() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("work-item plan");
    let AgenticPlan::Final(answer) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HARNESS_PROMPT)],
        &["write_file"],
    )
    .expect("a client without Read or Run can still state its boundary") else {
        panic!("an unobserved auxiliary Write must not claim persisted history")
    };
    // No work-item source or journal observation exists for this write-only client.
    let mut expected_plan = plan.clone();
    expected_plan.goal.clone_from(&plan.target);
    assert_eq!(answer, expected_plan.planned_not_executed_answer());
    assert_eq!(
        expected_plan.terminal_state,
        PlanTerminalState::PlannedNotExecuted
    );
    assert!(answer.contains("Planned, not executed"));
    assert!(answer.contains("The requested work remains unverified"));
    assert!(answer.contains(&plan.target));
    assert!(!answer.contains("You are an AI issue solver"));
    assert!(!answer.contains("Created and verified"));
    assert!(!answer.contains("Completed the general change request"));
    assert!(!answer.contains("Recorded and verified"));
    let outcome = run_agentic_task(HARNESS_PROMPT).expect("Agent CLI replay");
    assert!(outcome.steps.iter().all(|step| step.tool != "write_file"));
    assert!(!outcome.final_answer.contains("Recorded and verified"));
}

#[test]
fn a_genuinely_fetched_non_artifact_work_item_preserves_verified_history_and_stays_planned() {
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("work-item plan");
    let previous = compose_general_change_plan("Create file prior.txt containing earlier history")
        .expect("prior event")
        .links_notation();
    let mut workspace = tool_workspace::ToolWorkspace::new(HARNESS_PROMPT);
    workspace
        .write_initial(PLAN_PATH, &previous)
        .expect("physical prior history");
    let mut messages = work_item_transcript("Investigate why the nightly job is slow.");
    let mut final_answer = None;
    let mut calls = Vec::new();
    for turn in 0..12 {
        match formal_ai::agentic_coding::plan_chat_step(
            &messages,
            &["fetch_url", "read_file", "write_file"],
        )
        .expect("source-bound planned replay")
        {
            AgenticPlan::Final(answer) => {
                final_answer = Some(answer);
                break;
            }
            AgenticPlan::ToolCalls(planned) => {
                for (index, call) in planned.iter().enumerate() {
                    assert!(matches!(
                        formal_ai::agentic_coding::planner::tool_capability(&call.tool),
                        Some(
                            formal_ai::agentic_coding::planner::Capability::Read
                                | formal_ai::agentic_coding::planner::Capability::Write
                        )
                    ));
                    let args: serde_json::Value =
                        serde_json::from_str(&call.arguments).expect("arguments");
                    assert_eq!(args["path"].as_str(), Some(PLAN_PATH));
                    let id = format!("planned-history-{turn}-{index}");
                    messages.push(ChatMessage::assistant_tool_calls(vec![
                        formal_ai::protocol::ToolCall::function(
                            &id,
                            &call.tool,
                            call.arguments.clone(),
                        ),
                    ]));
                    messages.push(workspace.execute(&id, call));
                    calls.push(call.tool.clone());
                }
            }
        }
    }
    let answer = final_answer.expect("bounded replay settles");
    let mut expected_plan = plan.clone();
    expected_plan.goal = "Investigate why the nightly job is slow.".to_owned();
    assert_eq!(answer, expected_plan.planned_not_executed_answer());
    assert_eq!(
        expected_plan.terminal_state,
        PlanTerminalState::PlannedNotExecuted
    );
    assert!(answer.contains(&expected_plan.goal));
    assert!(answer.contains(&plan.target));
    assert!(!answer.contains("Created and verified"));
    assert!(!answer.contains("Completed the general change request"));
    assert!(!answer.contains("only file this run wrote"));
    assert!(!answer.contains("Recorded and verified"));
    assert!(!answer.contains("You are an AI issue solver"));
    assert!(
        calls.is_empty(),
        "unsupported work must not issue auxiliary writes or reads"
    );
    let history = workspace.read(PLAN_PATH).expect("physical history");
    assert_eq!(history, previous);
    assert!(workspace.read("greeting.txt").is_err());
    assert_eq!(plan.verification_command.len(), 0);
}

#[test]
fn offline_fetch_status_is_owned_by_registry_membership_and_preserves_wrapper_bytes() {
    use formal_ai::agentic_coding::{CANONICAL_SOURCE_URL, corpus};
    assert_eq!(
        corpus::web_fetch_result(CANONICAL_SOURCE_URL),
        Ok(corpus::web_fetch(CANONICAL_SOURCE_URL))
    );
    let url = "https://example.com/missing";
    assert_eq!(corpus::web_fetch_result(url), Err(corpus::web_fetch(url)));
}

/// A work item that names no artifact is not turned into one. Fabricating a file
/// the issue never asked for would be the failure #904 opened against, in the
/// opposite direction.
#[test]
fn a_work_item_naming_no_artifact_is_not_invented_into_one() {
    let messages = work_item_transcript("Investigate why the nightly job is slow.");
    let plan = formal_ai::agentic_coding::plan_chat_step(
        &messages,
        &["fetch_url", "write_file", "run_command"],
    );

    if let Some(AgenticPlan::ToolCalls(calls)) = plan {
        for call in &calls {
            assert!(
                !call.arguments.contains("greeting.txt"),
                "no artifact may be invented for a work item that names none: {}",
                call.arguments,
            );
        }
    }
}

/// A transcript in which the client has already fetched the work item, in the
/// shape [`formal_ai::agentic_coding::plan_chat_step`] reads results back from.
fn work_item_transcript(body: &str) -> Vec<ChatMessage> {
    const CALL_ID: &str = "call_work_item";
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");
    let call = formal_ai::protocol::ToolCall::function(
        CALL_ID,
        "fetch_url",
        serde_json::json!({ "url": plan.target }).to_string(),
    );
    let assistant = ChatMessage::assistant_tool_calls(vec![call]);
    let mut result = ChatMessage::user(body);
    result.role = String::from("tool");
    result.tool_call_id = Some(String::from(CALL_ID));
    vec![ChatMessage::user(HARNESS_PROMPT), assistant, result]
}

#[test]
fn structured_owned_issue_stdout_preserves_the_complete_original_requirements() {
    use formal_ai::ToolCall;
    let issues: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/issue-1364-hello-world-issues.json"
    ))
    .expect("public original issue body fixtures");
    let issue = &issues[0];
    let url = issue["html_url"].as_str().expect("original issue URL");
    let title = issue["title"].as_str().expect("original issue title");
    let body = issue["body"]
        .as_str()
        .expect("complete original issue body");
    let command = format!(
        "gh issue view '{url}' --json title --jq .title && echo && gh issue view '{url}' --json body --jq .body; printf '\\n__formal_ai_exit=%s\\n' $?"
    );
    let stdout = format!("{title}\n\n{body}\n__formal_ai_exit=0\n");
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/issue-1364-agent-tool-use.json"
    ))
    .expect("captured Agent envelope fixture");
    let mut typed = fixture["fixture"].clone();
    typed["part"]["state"]["status"] = serde_json::json!("completed");
    typed["part"]["state"]["input"] = serde_json::json!({"command":command});
    typed["part"]["state"]
        .as_object_mut()
        .expect("typed state")
        .remove("error");
    typed["part"]["state"]["metadata"] = serde_json::json!({
        "output": stdout, "exit": 0, "signal": null, "timedOut": false, "aborted": false
    });
    for result in [
        serde_json::json!({"exit_code":0,"stdout":stdout,"stderr":"","complete":true}),
        typed,
    ] {
        let messages = vec![
            ChatMessage::user(format!(
                "Issue to solve: {url}\nImplement every numbered requirement."
            )),
            ChatMessage::assistant_tool_calls_with_content(
                String::new(),
                vec![ToolCall::function(
                    "owned-issue-read".to_owned(),
                    "bash".to_owned(),
                    serde_json::json!({"command":command}).to_string(),
                )],
            ),
            ChatMessage::tool_result("owned-issue-read".to_owned(), "bash", result.to_string()),
        ];
        let plan = formal_ai::agentic_coding::plan_chat_step(&messages, &["bash", "write"])
            .expect("the fully observed original requirements remain actionable");
        let AgenticPlan::ToolCalls(calls) = plan else {
            panic!("accepted complete issue must plan a source effect");
        };
        let call = calls.first().expect("owned source operation");
        assert_eq!(call.tool, "write");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("source arguments");
        assert!(
            arguments["content"]
                .as_str()
                .expect("source bytes")
                .contains("Hello, World!")
        );
    }
}

fn issue_1364_resume_recipe_answer() -> formal_ai::SymbolicAnswer {
    serde_json::from_value(serde_json::json!({
        "intent":"code", "answer":"owned source", "confidence":1.0,
        "evidence_links":[], "links_notation":"",
        "execution_recipe":{"language":"javascript", "path":"owned.js", "source":"expected source\n",
            "supporting_files":[], "commands":["node owned.js"]}
    })).expect("a declared source recipe")
}

#[test]
fn issue_1364_resume_uses_exact_scalar_alias_probe_and_rejects_later_edits() {
    let answer = issue_1364_resume_recipe_answer();
    let tools = ["exec_command", "write", "Edit", "Read"];
    let request = "Issue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2";
    let mut messages = vec![ChatMessage::user(request.to_owned())];
    let Some(AgenticPlan::ToolCalls(calls)) =
        formal_ai::agentic_coding::plan_symbolic_command_reroute(&messages, &tools, &answer)
    else {
        panic!("an existing checkout requires a bounded source probe");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("probe operands");
    let command = arguments["command"].as_str().expect("owned probe command");
    messages.push(ChatMessage::assistant_tool_calls_with_content(
        String::new(),
        vec![formal_ai::protocol::ToolCall::function(
            "owned-probe".to_owned(),
            "exec_command".to_owned(),
            serde_json::json!({"cmd":command}).to_string(),
        )],
    ));
    messages.push(ChatMessage::tool_result("owned-probe".to_owned(), "exec_command", serde_json::json!({
        "exit_code":0, "stdout":serde_json::json!({"schema":"workspace-source-v1","path":"owned.js",
            "success":true,"complete":true,"absent":false,"source":"expected source\n"}).to_string(),
        "complete":true
    }).to_string()));
    let Some(AgenticPlan::ToolCalls(calls)) =
        formal_ai::agentic_coding::plan_symbolic_command_reroute(&messages, &tools, &answer)
    else {
        panic!("the exact cmd alias must retain source authority");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("verification operands");
    assert_eq!(arguments["command"], "node owned.js");
    messages.push(ChatMessage::assistant_tool_calls_with_content(String::new(), vec![formal_ai::protocol::ToolCall::function(
        "later-edit".to_owned(), "Edit".to_owned(), serde_json::json!({"path":"owned.js","old_string":"expected source\n","new_string":"other source\n"}).to_string(),
    )]));
    messages.push(ChatMessage::tool_result(
        "later-edit".to_owned(),
        "Edit",
        serde_json::json!({"exit_code":0,"stdout":"","complete":true}).to_string(),
    ));
    assert!(matches!(
        formal_ai::agentic_coding::plan_symbolic_command_reroute(&messages, &tools, &answer),
        Some(AgenticPlan::Final(_))
    ));
}

#[test]
fn issue_1364_resume_binds_the_actual_raw_canonical_patch_and_preserves_plain_issue_writes() {
    let answer = issue_1364_resume_recipe_answer();
    let tools = ["exec_command", "apply_patch"];
    let mut messages = vec![ChatMessage::user("Issue to solve: https://github.com/fixture/project/issues/1\nYour prepared Pull Request: https://github.com/fixture/project/pull/2".to_owned())];
    messages.push(ChatMessage::assistant_tool_calls_with_content(
        String::new(),
        vec![formal_ai::protocol::ToolCall::function(
            "owned-source-create".to_owned(),
            "apply_patch".to_owned(),
            "*** Begin Patch\n*** Add File: owned.js\n+expected source\n*** End Patch\n".to_owned(),
        )],
    ));
    messages.push(ChatMessage::tool_result(
        "owned-source-create".to_owned(),
        "apply_patch",
        serde_json::json!({"exit_code":0,"stdout":"","complete":true}).to_string(),
    ));
    let Some(AgenticPlan::ToolCalls(calls)) =
        formal_ai::agentic_coding::plan_symbolic_command_reroute(&messages, &tools, &answer)
    else {
        panic!("the exact raw declaration must certify its own newly created source");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("verification operands");
    assert_eq!(arguments["command"], "node owned.js");
    let plain = [ChatMessage::user(
        "Issue to solve: https://github.com/fixture/project/issues/1".to_owned(),
    )];
    let Some(AgenticPlan::ToolCalls(calls)) =
        formal_ai::agentic_coding::plan_symbolic_command_reroute(
            &plain,
            &["bash", "write"],
            &answer,
        )
    else {
        panic!("the ordinary issue route must preserve source creation");
    };
    assert_eq!(calls[0].tool, "write");
}

#[test]
fn issue_1364_tool_call_identity_is_stable_per_request_and_changes_after_history_progress() {
    let request_value = serde_json::json!({
        "model":"formal-ai",
        "messages":[{"role":"user","content":"Issue to solve: https://github.com/fixture/project/issues/1\nImplement every numbered requirement."}],
        "tools":[{"type":"function","function":{"name":"bash","parameters":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false}}}]
    });
    let request: formal_ai::ChatCompletionRequest =
        serde_json::from_value(request_value.clone()).expect("declared chat request");
    let solver = formal_ai::UniversalSolver::new(formal_ai::SolverConfig {
        agent_mode: true,
        ..Default::default()
    });
    let first = formal_ai::create_chat_completion_with_solver(&request, &solver);
    let repeated = formal_ai::create_chat_completion_with_solver(&request, &solver);
    let first_call = first.choices[0]
        .message
        .tool_calls
        .first()
        .expect("owned issue fetch");
    let repeated_call = repeated.choices[0]
        .message
        .tool_calls
        .first()
        .expect("same owned issue fetch");
    assert_eq!(first_call.id, repeated_call.id);
    let mut progressed_value = request_value;
    progressed_value["messages"].as_array_mut().expect("messages").push(serde_json::json!({"role":"assistant","content":"I am preparing the owned issue read."}));
    let progressed_request: formal_ai::ChatCompletionRequest =
        serde_json::from_value(progressed_value).expect("progressed request");
    let progressed = formal_ai::create_chat_completion_with_solver(&progressed_request, &solver);
    let progressed_call = progressed.choices[0]
        .message
        .tool_calls
        .first()
        .expect("same pending issue fetch");
    assert_eq!(first_call.function.name, progressed_call.function.name);
    assert_eq!(
        first_call.function.arguments,
        progressed_call.function.arguments
    );
    assert_ne!(first_call.id, progressed_call.id);
}
