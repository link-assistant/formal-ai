//! Regression coverage for issue #1069's real Agent CLI entry point.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::recursive_execution::{
    RecursiveExecution, RecursiveTask, TaskAttempt, TaskExecutor, solve_recursively_within,
};
use formal_ai::task_decomposition::SplittingExecutor;

use crate::tool_workspace as observed_tool_workspace;

const ISSUE_URL: &str = "https://github.com/link-assistant/formal-ai/issues/1069";

#[test]
fn solve_issue_request_reads_the_work_item_before_project_lookup() {
    let task = format!(
        "Solve {ISSUE_URL} in this checkout as one whole task. Read the entire issue before acting."
    );
    let tools = ["web_fetch", "write_file", "run_command"];

    let Some(AgenticPlan::ToolCalls(calls)) =
        plan_chat_step(&[ChatMessage::user(task.clone())], &tools)
    else {
        panic!("a software-authoring request naming an issue must produce a tool call");
    };

    // Read the issue through the preferred shell tool before project lookup.
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "run_command");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("structured shell arguments");
    assert_eq!(
        arguments["command"],
        format!(
            "gh issue view '{ISSUE_URL}' --json title --jq .title && echo && gh issue view '{ISSUE_URL}' --json body --jq .body; printf '\n__formal_ai_exit=%s\n' $?"
        )
    );
    assert!(
        calls[0].arguments.contains(ISSUE_URL),
        "{}",
        calls[0].arguments
    );

    // A client with no shell still reads the work item, through its fetch tool.
    let fetch_only = ["web_fetch", "write_file"];
    let Some(AgenticPlan::ToolCalls(fetched)) =
        plan_chat_step(&[ChatMessage::user(task)], &fetch_only)
    else {
        panic!("a fetch-only client must still read the work item");
    };
    assert_eq!(fetched[0].tool, "web_fetch");
    assert!(
        fetched[0].arguments.contains(ISSUE_URL),
        "{}",
        fetched[0].arguments
    );
}

#[test]
fn fetched_issue_prose_does_not_pair_content_with_a_later_filename() {
    let task = format!("Solve {ISSUE_URL} in this checkout as one whole task.");
    let tools = ["web_fetch", "write_file", "run_shell_command"];
    let workspace = observed_tool_workspace::ToolWorkspace::new(&task);
    let mut messages = vec![ChatMessage::user(task.clone())];
    let body = "Produce a legitimate release with no fabricated evidence.\n\
         Adding a bypass flag to check-self-development-release.rs is not acceptable.";

    let Some(AgenticPlan::ToolCalls(fetches)) = plan_chat_step(&messages, &tools) else {
        panic!("the issue must be fetched before its body is planned");
    };
    assert_eq!(fetches.len(), 1);
    let fetch = &fetches[0];
    assert_eq!(fetch.tool, "run_shell_command");
    let arguments: serde_json::Value = serde_json::from_str(&fetch.arguments).unwrap();
    assert_eq!(
        arguments["command"],
        format!(
            "gh issue view '{ISSUE_URL}' --json title --jq .title && echo && gh issue view '{ISSUE_URL}' --json body --jq .body; printf '\n__formal_ai_exit=%s\n' $?"
        )
    );
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "fetch-issue",
        &fetch.tool,
        fetch.arguments.clone(),
    )]));
    // Prose alone cannot prove that a shell read succeeded. Retry through Fetch.
    messages.push(ChatMessage::tool_result("fetch-issue", &fetch.tool, body));
    let Some(AgenticPlan::ToolCalls(retries)) = plan_chat_step(&messages, &tools) else {
        panic!("an unverified shell receipt must retry the issue read");
    };
    assert_eq!(retries.len(), 1);
    assert_eq!(retries[0].tool, "web_fetch");
    let retry: serde_json::Value = serde_json::from_str(&retries[0].arguments).unwrap();
    assert_eq!(retry["url"], ISSUE_URL);

    // The fallback's successful Fetch receipt owns the unchanged source prose.
    let retry_fetch = &retries[0];
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "fetch-retry",
        &retry_fetch.tool,
        retry_fetch.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "fetch-retry",
        &retry_fetch.tool,
        body,
    ));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("unsupported prose must settle without auxiliary writes or source effects");
    };
    let evidence: serde_json::Value = serde_json::from_str(&answer).expect("structured gap");
    assert_eq!(evidence["schema"], "repository-work-item-evidence/v1");
    assert_eq!(evidence["reason"], "MissingReadTool");
    assert_eq!(evidence["workItem"], ISSUE_URL);
    assert_eq!(evidence["authored"], false);
    assert_eq!(evidence["verified"], false);
    assert_eq!(
        evidence["taskIdentity"],
        formal_ai::source_fetch::sha256_hex(body.trim().as_bytes())
    );
    for requirement in evidence["requirements"]
        .as_array()
        .expect("original clauses")
    {
        let start =
            usize::try_from(requirement["span"][0].as_u64().expect("start")).expect("start offset");
        let end =
            usize::try_from(requirement["span"][1].as_u64().expect("end")).expect("end offset");
        assert_eq!(
            &body.trim().as_bytes()[start..end],
            requirement["text"]
                .as_str()
                .expect("original clause")
                .as_bytes()
        );
        assert_eq!(requirement["status"], "unbound");
    }
    assert!(!answer.contains("Recorded and verified"));
    assert!(!answer.contains("Created and verified"));
    assert!(
        workspace
            .read(".formal-ai/general-change-plan.lino")
            .is_err()
    );
    assert!(workspace.read("check-self-development-release.rs").is_err());
}

#[test]
fn agent_compaction_continuation_resumes_the_summarized_task() {
    let summary = "Conversation summary: Inspect the repository's queue model and identify \
        where each queue stores its pending entries. Record the result in \
        `.agent-evidence/queue.md`.\n\nTitle: Queue model inspection\n\nUser turns:\n  \
        1. Inspect the repository's queue model and identify where each queue stores its \
        pending entries. Record the result in `.agent-evidence/queue.md`.";
    let messages = vec![
        ChatMessage::user("What did we do so far?"),
        ChatMessage::assistant(summary),
        ChatMessage::user("Continue if you have next steps"),
    ];
    let tools = ["grep", "write", "websearch"];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("a compacted Agent task must continue through its remaining tool steps");
    };

    assert!(
        calls.iter().all(|call| {
            !call
                .arguments
                .to_lowercase()
                .contains("continue if you have next steps")
        }),
        "the Agent continuation phrase replaced the summarized task: {calls:#?}",
    );
    assert!(
        calls.iter().any(|call| {
            let arguments = call.arguments.to_lowercase();
            arguments.contains("queue") && arguments.contains("model")
        }),
        "the compacted task was not restored: {calls:#?}",
    );
}

#[test]
fn repeated_agent_compaction_finds_an_embedded_summary_envelope() {
    // A second live Agent compaction summarized the preceding compaction turn.
    // Its assistant message no longer began with the envelope: it repeated the
    // protocol continuation first, then embedded the recoverable task summary.
    // The numbered user appendix was empty, so only the summary prose remained.
    let summary = "Continue if you have next steps.  Conversation summary: Inspect the \
        repository's task-decomposition data model and identify where a node stores its \
        children. Create `agent-ladder-effects/node-1.1.1.1.1.lino` with these exact \
        field lines: `node_path=1.1.1.1.1`, `node_depth=5`, `node_kind=leaf`, and \
        `result=` followed by the observed result. Leave supporting evidence in . \
        agent-ladder/node-1.1.1.1.1-proof.md. The first line must be exactly \
        node_path=1.1.1.1.1 and the body must state the concrete result.\n\nTitle: What \
        did we do so\n\nUser turns:\n  1.\n  2. Continue if you have next steps";
    let messages = vec![
        ChatMessage::user("What did we do so far?"),
        ChatMessage::assistant(summary),
        ChatMessage::user("Continue if you have next steps"),
    ];
    let tools = ["grep", "write", "websearch"];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("a repeatedly compacted Agent task must resume from its embedded summary");
    };

    assert!(
        calls.iter().all(|call| {
            !call
                .arguments
                .to_lowercase()
                .contains("continue if you have next steps")
        }),
        "the outer continuation replaced the embedded summarized task: {calls:#?}",
    );
    assert!(
        calls.iter().any(|call| {
            let arguments = call.arguments.to_lowercase();
            arguments.contains("task") && arguments.contains("decomposition")
        }),
        "the embedded task summary was not restored: {calls:#?}",
    );
}

#[test]
fn issue_1069_delivery_recovers_exact_paths_from_the_preserved_user_turn() {
    let summary = "Conversation summary: Create `. agent-evidence/queue.txt` containing \
        queue ready.\n\nTitle: Queue evidence\n\nUser turns:\n  1. Create \
        `.agent-evidence/queue.txt` containing queue ready.\n  2.";
    let messages = vec![
        ChatMessage::user("What did we do so far?"),
        ChatMessage::assistant(summary),
        ChatMessage::user("Continue if you have next steps"),
    ];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["write"]) else {
        panic!("the exact compacted write must remain executable");
    };

    assert!(
        calls
            .iter()
            .any(|call| call.arguments.contains(".agent-evidence/queue.txt")),
        "the prose summary's whitespace corruption replaced the exact user turn: {calls:#?}",
    );
}

#[derive(Default)]
struct RefusingExecutor {
    attempted: Vec<String>,
}

impl TaskExecutor for RefusingExecutor {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        self.attempted.push(task.goal.clone());
        TaskAttempt::failed("required workspace effect was missing")
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }
}

#[test]
fn failed_atomic_strategy_stages_are_not_split_into_the_same_strategy_again() {
    let task = format!("Solve {ISSUE_URL} in this checkout as one whole task.");
    let root = RecursiveTask::leaf("issue-1069", task);
    let mut executor = SplittingExecutor::new(RefusingExecutor::default());

    let run = solve_recursively_within(&root, &mut executor, 3);

    assert_eq!(run.status, RecursiveExecution::Blocked);
    assert_eq!(run.split_depth_reached(), 1, "{run:#?}");
    assert_eq!(
        executor.productive_splits().len(),
        1,
        "{:#?}",
        executor.splits()
    );
    assert_eq!(run.blocked_leaves().len(), 4, "{run:#?}");
    assert!(
        executor.inner().attempted.iter().all(|attempt| {
            !attempt
                .starts_with("Record independently checkable requirements for Record independently")
                && !attempt.starts_with(
                    "Add a regression test that reproduces Record independently checkable",
                )
        }),
        "an atomic stage was planned again: {:#?}",
        executor.inner().attempted,
    );
}
