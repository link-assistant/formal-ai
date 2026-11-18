//! Issue #1159 (sub-issue of #1162): a Hello World delivery whose compiler is
//! missing recovers through the pinned publisher procedure when a workspace
//! grant allows it, and otherwise stops with a "grant required" answer that
//! names the missing program and the procedure a grant would run.
//!
//! The replay is the Agent CLI Scala run: its first compile step answered
//! `/bin/sh: 1: scalac: not found` with exit status 127.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const COMPILE: &str = "scalac Main.scala";
const NOT_FOUND: &str = "Output: \nError: /bin/sh: 1: scalac: not found\nExit Code: 127";
const PUBLISHER: &str = "https://api.github.com/repos/scala/scala3/releases/latest";
const INSTALLER: &str = ".formal-ai/bootstrap/prerequisite-installer.py";
const TOOLS: [&str; 4] = ["bash", "read", "write", "edit"];

fn record(messages: &mut Vec<ChatMessage>, id: &str, tool: &str, arguments: &str, result: &str) {
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.to_owned(),
        tool.to_owned(),
        arguments.to_owned(),
    )]));
    messages.push(ChatMessage::tool_result(id.to_owned(), tool, result));
}

/// The replayed transcript: the request, then the failed first compile step.
fn replay() -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user("Compile the requested program.")];
    let compile = serde_json::json!({ "command": COMPILE }).to_string();
    record(&mut messages, "scala-0", "bash", &compile, NOT_FOUND);
    messages
}

fn single_call(plan: Option<AgenticPlan>) -> PlannedToolCall {
    match plan {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1, "{calls:?}");
            calls[0].clone()
        }
        other => panic!("expected one tool call, got {other:?}"),
    }
}

#[test]
fn without_a_grant_the_plan_stops_naming_scalac_and_its_publisher_procedure() {
    let plan = plan_chat_step(&replay(), &TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("a denied recovery stops with a final answer, got {plan:?}");
    };
    assert!(
        answer.contains("Missing prerequisite `scalac`. Grant required"),
        "{answer}"
    );
    assert!(
        answer.contains("FORMAL_AI_INSTALL_GRANT=workspace"),
        "{answer}"
    );
    assert!(
        answer.contains(PUBLISHER),
        "the answer names the procedure: {answer}"
    );
    assert!(
        answer.contains("would be re-probed") && answer.contains("retried once"),
        "{answer}"
    );
    assert!(answer.contains(&format!("`{COMPILE}`")), "{answer}");
    assert!(answer.contains("scalac: not found"), "{answer}");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn with_a_workspace_grant_the_publisher_procedure_reprobe_and_original_step_are_planned_next() {
    // The grant is process-global configuration set once at startup, so the
    // granted replay runs in a child process and the default-deny tests in
    // this binary keep their default.
    const CHILD: &str = "FORMAL_AI_ISSUE_1159_GRANT_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = std::thread::current().name().expect("test name").to_owned();
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .expect("isolated grant child");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let grant = include_str!("../../../data/meta/workspace-prerequisite-grant.json");
    formal_ai::prerequisite::configure_workspace_grant(Some(grant)).expect("caller grant");

    let mut messages = replay();
    // 1. The canonical bounded installer is written first.
    let write = single_call(plan_chat_step(&messages, &TOOLS));
    assert_eq!(write.tool, "write");
    let written: serde_json::Value = serde_json::from_str(&write.arguments).unwrap();
    assert_eq!(written["path"], INSTALLER);
    assert_eq!(
        written["content"],
        include_str!("../../../data/meta/prerequisite-installer.py")
    );
    record(&mut messages, "scala-1", "write", &write.arguments, "");

    // 2. Then one transaction: the pinned scalac publisher procedure, the
    // re-probe that publishes the active binary, then the original step.
    let run = single_call(plan_chat_step(&messages, &TOOLS));
    assert_eq!(run.tool, "bash");
    let command = serde_json::from_str::<serde_json::Value>(&run.arguments).unwrap()["command"]
        .as_str()
        .expect("command")
        .to_owned();
    let publisher = command.find(PUBLISHER).expect("publisher procedure");
    let reprobe = command
        .find(".formal-ai/toolchains/active-scalac.bin")
        .expect("re-probe of the installed scalac");
    let retry = command
        .find("# __formal_ai_prerequisite_retry\nscalac Main.scala")
        .expect("original step retried");
    assert!(
        command.starts_with("python3 ") && command[..publisher].contains(INSTALLER),
        "{command}"
    );
    assert!(command.contains(r#""program":"scalac""#), "{command}");
    assert!(publisher < reprobe && reprobe < retry, "{command}");
    assert!(command.ends_with(COMPILE), "{command}");

    // 3. A successful transaction is not planned again, and the answer reports
    // the recovery: the program, the procedure that ran, the re-probe and the
    // retried command.
    record(
        &mut messages,
        "scala-2",
        "bash",
        &run.arguments,
        "Output: \nExit Code: 0",
    );
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("a recovered compile finishes");
    };
    let report = format!(
        "Recovered the missing prerequisite `scalac` under FORMAL_AI_INSTALL_GRANT=workspace: the pinned `scalac` publisher setup procedure ({PUBLISHER}) ran, `scalac` was re-probed, and `{COMPILE}` was retried once and succeeded.\n\n"
    );
    assert!(answer.starts_with(&report), "{answer}");
}

/// R1159-4: the delivered workflow sets up the toolchain the recovery
/// verified with. The Scala workflow's compiler pin is resolved (issue #1168)
/// from the same release feed the pinned `scalac` publisher record names, and
/// the workflow installs `scalac` at that version.
#[test]
fn the_scala_workflow_installs_scalac_from_the_release_feed_of_the_recovery_publisher() {
    let publisher = formal_ai::prerequisite::publisher::seed_publishers()
        .into_iter()
        .find(|publisher| publisher.program == "scalac")
        .expect("scalac has a pinned publisher");
    assert_eq!(publisher.documentation_url(), PUBLISHER);
    let versions = formal_ai::version_resolution::VersionSet::baseline()
        .expect("the toolchains seed carries every baseline");
    assert_eq!(
        versions.scala.source_url,
        publisher.documentation_url(),
        "the workflow's Scala version and the recovery come from one publisher record"
    );
    let answer = formal_ai::program_contract::answer(
        "Write a Scala program that prints \"Hello, World!\" to stdout.",
        &mut formal_ai::EventLog::new(),
    )
    .expect("the program contract answers");
    let recipe = answer.execution_recipe.expect("a recipe");
    let workflow = formal_ai::agentic_coding::render_ci_workflow(&recipe, &versions);
    let scala = &versions.scala.tag;
    assert!(
        workflow.contains(&format!("apps: scala:{scala} scalac:{scala}\n")),
        "{workflow}"
    );
    for command in &recipe.commands {
        assert!(
            workflow.contains(&format!("      - run: |\n          {command}\n")),
            "the workflow runs the verified command `{command}`: {workflow}"
        );
    }
}
