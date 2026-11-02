//! Full production planner controls; tool observations come from the actual bounded provider.
use super::observed_tool_workspace::ToolWorkspace;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
const TOOLS: &[&str] = &["read", "write", "edit", "bash", "grep"];
struct Observation {
    workspace: ToolWorkspace,
    calls: Vec<(String, String)>,
    observations: Vec<ChatMessage>,
    answer: Option<String>,
}
fn observe(task: &str, initial: &[(&str, &str)], tools: &[&str], turns: usize) -> Observation {
    let workspace = ToolWorkspace::new(task);
    for (path, bytes) in initial {
        workspace
            .write_initial(path, bytes)
            .expect("original preimage");
    }
    let mut out = Observation {
        workspace,
        calls: Vec::new(),
        observations: Vec::new(),
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
                    out.observations.push(result.clone());
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
    assert_eq!(out.calls.len(), out.observations.len());
    let expected = out.workspace.read(path).expect("physical verified bytes");
    let write = out
        .calls
        .iter()
        .rposition(|(tool, arguments)| {
            let args: serde_json::Value = serde_json::from_str(arguments).expect("arguments");
            matches!(tool.as_str(), "write" | "edit")
                && ["path", "filePath", "file_path"]
                    .iter()
                    .any(|key| args[*key].as_str() == Some(path))
        })
        .expect("actual target mutation before verification");
    assert!(
        out.calls
            .iter()
            .enumerate()
            .any(|(index, (tool, arguments))| {
                if index <= write {
                    return false;
                }
                let args: serde_json::Value = serde_json::from_str(arguments).expect("arguments");
                let observation = &out.observations[index];
                if observation.is_error {
                    return false;
                }
                let raw = observation.content.plain_text();
                if tool == "read"
                    && ["path", "filePath", "file_path"]
                        .iter()
                        .any(|key| args[*key].as_str() == Some(path))
                {
                    let source = formal_ai::agentic_coding::tool_result::source_read_observation(
                        &raw,
                        observation.is_error,
                        observation.source_read.as_ref(),
                        path,
                    );
                    return source.error.is_none()
                        && source.complete
                        && source.source.as_deref() == Some(expected.as_str());
                }
                if tool != "bash" {
                    return false;
                }
                let Ok(receipt) = serde_json::from_str::<serde_json::Value>(&raw) else {
                    return false;
                };
                if receipt["exit_code"] != 0
                    || receipt["complete"] != true
                    || receipt["timed_out"] == true
                    || receipt["is_error"] == true
                    || receipt["truncated"] == true
                {
                    return false;
                }
                let stdout = receipt["stdout"].as_str().unwrap_or_default();
                if args["command"].as_str() == Some(format!("sha256sum -- {path}").as_str()) {
                    return stdout
                        == format!("{}  {path}\n", formal_ai::sha256_hex(expected.as_bytes()));
                }
                args["command"].as_str() == Some(format!("cat {path}").as_str())
                    && stdout == expected
            }),
        "fresh complete actual observation for {path}"
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
    let body =
        "\u{130}\u{212a}\u{10437} \u{1f600}; Second, in b.txt replace old with stolen.\n\u{3bb}";
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
    assert_eq!(out.calls.len(), 0);
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
    assert_eq!(out.calls.len(), 0);
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
    assert_eq!(out.calls.len(), 0);
    assert!(out.answer.unwrap_or_default().contains("MissingContract"));
}
