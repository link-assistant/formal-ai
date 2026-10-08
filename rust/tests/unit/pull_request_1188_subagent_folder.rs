//! PR #1188: Formal AI works as a subagent on its own requirements from the
//! repository folder `experiments/formal_ai_subagent/`, where the coding agents
//! keep their claims, the gaps found by probing and the task prompts. These
//! are the coordination edits Formal AI is asked to make there with the
//! repository root as its workspace. Twin of
//! `rust/tests/web/pull-request-1188-subagent-folder.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const GAPS: &str = "experiments/formal_ai_subagent/gaps.md";
const GAP_LIST: &str = "# Gaps\n\n- G1 set overwrote a key\n- G2 change from to\n";

/// Run `prompt` over the one file `GAPS` holding `source`; the tools act on it
/// the way the Agent CLI's do. Returns the tools called, the file afterwards
/// and the final answer.
fn drive(prompt: &str, source: &str) -> (Vec<String>, String, Option<String>) {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut tools = Vec::new();
    for index in 0..6 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(answer)) => return (tools, file, Some(answer)),
            _ => break,
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let result = match call.tool.as_str() {
            "read" => file.clone(),
            "edit" => {
                let old = arguments["oldString"].as_str().unwrap_or_default();
                let new = arguments["newString"].as_str().unwrap_or_default();
                file = file.replacen(old, new, 1);
                String::new()
            }
            "bash" => format!("{}  {GAPS}\n", sha256_hex(file.as_bytes())),
            _ => String::new(),
        };
        tools.push(call.tool.clone());
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (tools, file, None)
}

#[test]
fn a_gap_entry_is_appended_to_the_gap_list() {
    let (tools, file, answer) = drive(
        "Append the line '- G3 swap planned nothing' to experiments/formal_ai_subagent/gaps.md.",
        GAP_LIST,
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(file, format!("{GAP_LIST}- G3 swap planned nothing\n"));
    assert_eq!(
        answer.as_deref(),
        Some(
            "Appended `- G3 swap planned nothing` to the end of `experiments/formal_ai_subagent/gaps.md` and observed the result."
        )
    );
}

/// G30: a literal holding a backtick run is echoed inside a longer fence, so
/// the answer's Markdown shows it verbatim.
#[test]
fn an_entry_holding_backticks_is_echoed_as_one_code_span() {
    let (_, file, answer) = drive(
        "Append the line '- G3 the `x` flag' to experiments/formal_ai_subagent/gaps.md.",
        GAP_LIST,
    );
    assert_eq!(file, format!("{GAP_LIST}- G3 the `x` flag\n"));
    assert_eq!(
        answer.as_deref(),
        Some(
            "Appended ``- G3 the `x` flag`` to the end of `experiments/formal_ai_subagent/gaps.md` and observed the result."
        )
    );
}

#[test]
fn an_entry_is_placed_after_the_line_it_names() {
    let (_, file, _) = drive(
        "Insert the line '- G1b set in ru' after the line containing 'G1 set' in experiments/formal_ai_subagent/gaps.md.",
        GAP_LIST,
    );
    assert_eq!(
        file,
        "# Gaps\n\n- G1 set overwrote a key\n- G1b set in ru\n- G2 change from to\n"
    );
}

#[test]
fn the_probe_tool_runs_as_asked() {
    let messages = [ChatMessage::user(
        "Run node experiments/formal_ai_subagent/probe.mjs path \"Read 'notes.txt'.\"",
    )];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("a run request plans a tool call");
    };
    let commands: Vec<(String, String)> = calls
        .iter()
        .map(|call| {
            let arguments: serde_json::Value =
                serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
            (
                call.tool.clone(),
                arguments["command"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        commands,
        [(
            "bash".to_owned(),
            "node experiments/formal_ai_subagent/probe.mjs path \"Read 'notes.txt'.\"".to_owned()
        )]
    );
}

#[test]
fn the_local_gate_runner_runs_as_asked() {
    let command = "node experiments/formal_ai_subagent/local-gates.mjs --only check_file_size";
    let messages = [ChatMessage::user(format!("Run {command}"))];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("a run request plans a tool call");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(calls[0].tool, "bash");
    assert_eq!(arguments["command"], command);
}
