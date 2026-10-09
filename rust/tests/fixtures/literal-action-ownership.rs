//! Full production planner controls; tool observations come from the actual bounded provider.
use super::observed_tool_workspace::ToolWorkspace;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
const TOOLS: &[&str] = &["read", "write", "edit", "bash", "grep"];
struct Observation {
    workspace: ToolWorkspace,
    calls: Vec<(String, String)>,
    answer: Option<String>,
}
fn observe(task: &str, initial: &[(&str, &str)], tools: &[&str], turns: usize) -> Observation {
    let mut workspace = ToolWorkspace::new(task);
    for (path, bytes) in initial {
        workspace
            .write_initial(path, bytes)
            .expect("original preimage");
    }
    let mut out = Observation {
        workspace,
        calls: Vec::new(),
        answer: None,
    };
    let mut messages = vec![ChatMessage::user(task)];
    for turn in 0..turns {
        match plan_chat_step(&messages, tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                for (index, call) in calls.into_iter().enumerate() {
                    let id = format!("owned-{turn}-{index}");
                    let result = out.workspace.execute(&id, &call);
                    out.calls.push((call.tool.clone(), call.arguments.clone()));
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        &id,
                        &call.tool,
                        call.arguments,
                    )]));
                    messages.push(result);
                }
            }
            Some(AgenticPlan::Final(answer)) => {
                out.answer = Some(answer);
                break;
            }
            None => break,
        }
    }
    out
}
fn verified(out: &Observation, path: &str) {
    assert!(
        out.calls.iter().any(|(tool, arguments)| {
            let arguments: serde_json::Value = serde_json::from_str(arguments).expect("arguments");
            tool == "bash"
                && [format!("cat {path}"), format!("sha256sum -- {path}")]
                    .iter()
                    .any(|command| arguments["command"].as_str() == Some(command.as_str()))
        }),
        "independent verification for {path}"
    );
}
#[test]
fn original_seed_creation_owns_internal_edit_lexemes() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("literal-action-promotion.json"))
            .expect("original fixture");
    let out = observe(fixture["prompt"].as_str().unwrap(), &[], TOOLS, 8);
    assert_eq!(
        out.workspace
            .read("data/seed/learned-program-rules.lino")
            .unwrap(),
        fixture["desired"].as_str().unwrap()
    );
    verified(&out, "data/seed/learned-program-rules.lino");
    assert!(
        !out.answer
            .as_deref()
            .unwrap_or_default()
            .contains("does not occur")
    );
}
#[test]
fn original_span_owned_create_and_independent_edit_deliver_both() {
    for task in [
        "First, create file a.txt containing «replace x with y». Second, in b.txt replace «old» with «new».",
        "Create file a.txt containing «replace x with y». Then in b.txt replace «old» with «new».",
    ] {
        let out = observe(task, &[("b.txt", "old")], TOOLS, 8);
        assert_eq!(out.workspace.read("a.txt").unwrap(), "replace x with y");
        assert_eq!(out.workspace.read("b.txt").unwrap(), "new");
        verified(&out, "a.txt");
        verified(&out, "b.txt");
        assert!(out.answer.is_some());
    }
}
#[test]
fn unicode_payload_does_not_own_later_outer_edit() {
    let body = "İK𐐷 😀; Second, in b.txt replace old with stolen.\nλ";
    let task =
        format!("Create file α.txt containing «{body}». Then in b.txt replace «old» with «new».");
    let out = observe(&task, &[("b.txt", "old")], TOOLS, 8);
    assert_eq!(out.workspace.read("α.txt").unwrap(), body);
    assert_eq!(out.workspace.read("b.txt").unwrap(), "new");
    verified(&out, "α.txt");
    verified(&out, "b.txt");
}
#[test]
fn unsupported_later_goal_stays_open_after_known_artifact() {
    for task in [
        "Create file a.txt containing «old». Then design a safe rollback mechanism.",
        "Create file a.txt containing «old». Then Run node --version.",
    ] {
        let out = observe(task, &[], TOOLS, 8);
        assert_eq!(out.workspace.read("a.txt").unwrap(), "old");
        verified(&out, "a.txt");
        assert!(
            out.answer
                .as_deref()
                .unwrap_or_default()
                .contains("no_artifact_in_clause")
        );
        assert!(
            !out.answer
                .as_deref()
                .unwrap_or_default()
                .contains("Completed the general change request")
        );
    }
}
#[test]
fn inline_semantic_tail_is_not_literal_bytes_or_completion() {
    let out = observe(
        "Create file a.txt containing «old» and design a safe rollback mechanism. Then in b.txt replace «old» with «new».",
        &[("b.txt", "old")],
        TOOLS,
        8,
    );
    assert!(out.workspace.read("a.txt").is_err());
    assert_eq!(out.workspace.read("b.txt").unwrap(), "old");
    assert!(out.calls.is_empty());
    assert!(
        out.answer
            .unwrap_or_default()
            .contains("no_artifact_in_clause")
    );
}
#[test]
fn nested_quotes_cannot_bypass_existing_fault_refusal() {
    let out = observe(
        "Create file a.txt containing 'a('x')'. Then in b.txt replace «old» with «new».",
        &[("b.txt", "old")],
        TOOLS,
        8,
    );
    assert!(out.workspace.read("a.txt").is_err());
    assert_eq!(out.workspace.read("b.txt").unwrap(), "old");
    assert!(out.calls.is_empty());
}
#[test]
fn failed_edit_preserves_preimage_and_cannot_certify_all_goals() {
    let out = observe(
        "Create file a.txt containing «replace x with y». Then in b.txt replace «old» with «new».",
        &[("b.txt", "foreign")],
        TOOLS,
        8,
    );
    assert_eq!(out.workspace.read("a.txt").unwrap(), "replace x with y");
    assert_eq!(out.workspace.read("b.txt").unwrap(), "foreign");
    assert!(
        !out.answer
            .as_deref()
            .unwrap_or_default()
            .contains("Replaced")
    );
}
#[test]
fn actual_operation_object_owns_ambiguous_authoring_action() {
    for task in [
        "haz una búsqueda web de rust ownership",
        "Make a web search for Rust ownership",
        "Сделай поиск в интернете о Rust",
        "वेब पर rust ownership खोजें",
        "在网络上查找 rust ownership",
    ] {
        let plan = plan_chat_step(
            &[ChatMessage::user(task)],
            &[
                "web_fetch",
                "web_search",
                "read_file",
                "write_file",
                "exec_command",
            ],
        )
        .expect("search plan");
        let AgenticPlan::ToolCalls(calls) = plan else {
            panic!("known search must not become a coding Gap")
        };
        assert_eq!(calls[0].tool, "web_search");
    }
    let out = observe("Implement bounded transactional observation", &[], TOOLS, 8);
    assert!(out.calls.is_empty());
    assert!(out.answer.unwrap_or_default().contains("MissingContract"));
}
