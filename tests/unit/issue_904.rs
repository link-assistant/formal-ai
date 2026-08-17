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
        for placeholder in ["{target}", "{plan_path}", "{plan}"] {
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
        panic!("the first step must persist the plan record")
    };
    assert_eq!(calls[0].tool, "write_file");

    let outcome = run_agentic_task(HARNESS_PROMPT).expect("Agent CLI replay");
    assert!(!outcome.hit_turn_cap);
    // The driver advertises a fetch tool, so the run reads the work item before
    // concluding anything (the follow-up below). Its offline corpus holds no
    // such issue, so the read comes back empty and the record is still the only
    // honest end — but no step reads that record back, which is the defect this
    // test was opened for.
    assert_eq!(
        outcome
            .steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["web_fetch", "write_file"],
        "the run must read the work item, then record it — and never read its own plan back",
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
    assert_eq!(calls[0].tool, "fetch_url");
    assert!(
        calls[0].arguments.contains(&plan.target),
        "the fetch must address the issue Hive Mind named: {}",
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
    assert_eq!(calls[0].tool, "fetch_url");
    assert!(
        calls[0].arguments.contains(&plan.target),
        "the fetch must address the work item, not some other page: {}",
        calls[0].arguments,
    );
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
    let AgenticPlan::ToolCalls(calls) = formal_ai::agentic_coding::plan_chat_step(
        &[ChatMessage::user(HARNESS_PROMPT)],
        &["write_file"],
    )
    .expect("a write-only client still records the reference") else {
        panic!("the plan record is still written")
    };
    assert_eq!(
        calls[0].tool, "write_file",
        "with no way to read the work item, recording it is still the honest end",
    );

    let outcome = run_agentic_task(HARNESS_PROMPT).expect("Agent CLI replay");
    assert!(
        outcome.final_answer.contains("Planned"),
        "a run that could not read the work item must still say so: {}",
        outcome.final_answer,
    );
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
    let plan = compose_general_change_plan(HARNESS_PROMPT).expect("repository work-item plan");
    let call = formal_ai::protocol::ToolCall::function(
        "call_work_item",
        "fetch_url",
        serde_json::json!({ "url": plan.target }).to_string(),
    );
    let assistant = ChatMessage::assistant_tool_calls(vec![call.clone()]);
    let mut result = ChatMessage::user(body);
    result.role = String::from("tool");
    result.tool_call_id = Some(call.id.clone());
    vec![ChatMessage::user(HARNESS_PROMPT), assistant, result]
}
