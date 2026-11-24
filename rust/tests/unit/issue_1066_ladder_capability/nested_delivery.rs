//! The same source observation supports a pinned proof and a structured effect.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

use crate::tool_workspace::ToolWorkspace;

/// The fourteen tool names `@link-assistant/agent` advertises, in the order the
/// live ladder trace recorded them.
pub(super) const LADDER_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];

/// How many turns a replayed run is given to reach the file it was asked for.
///
/// The ladder runs its nodes with a turn budget too; this is a smaller one,
/// because a literal write that has not happened within a handful of turns is
/// the failure these tests exist to catch, not a run that needs longer.
pub(super) const LADDER_TURN_CAP: usize = 6;

#[test]
fn nested_delivery_carries_the_observation_into_the_outer_effect() {
    // One investigation feeds two outputs in the real ladder: a human-readable
    // proof and a machine-checked effect. Once the proof write succeeded, its
    // "Recorded the findings" status used to become the effect's `result`,
    // discarding the observation that both artifacts were meant to record.
    let prompt = "Inspect the existing task-decomposition data model and identify where a node \
        stores its children. Create `audit-effects/decomposition.lino` with these exact field \
        lines: `subject=decomposition`, `kind=inspection`, and `result=` followed by the \
        observed result. Leave supporting evidence in `.audit/decomposition-proof.md`. The \
        first line must be exactly `proof_for=decomposition`.";
    assert_nested_delivery_observes_source(prompt);
}

#[test]
fn an_angle_delimited_result_instruction_records_the_observation() {
    let prompt = "Inspect the existing task-decomposition data model and identify where a node \
        stores its children. Create `audit-effects/decomposition.lino` with these exact field lines: \
        `subject=decomposition`, `kind=inspection`, and `result=<the result actually observed>`. \
        Leave supporting evidence in `.audit/decomposition-proof.md`. \
        The first line must be exactly `proof_for=decomposition`.";
    assert_nested_delivery_observes_source(prompt);
}

#[test]
fn the_result_and_proof_retain_each_verified_edit_in_a_sequence() {
    let source = "settings/options.txt";
    let prompt = "Insert the line «cobalt dawn» after the line «azure noon» in settings/options.txt, +        then delete the line «stale entry» from settings/options.txt. Create +        `agent-ladder-effects/node.lino` with these exact field lines: `kind=edit` and +        `result=<each verified edit>`. Leave supporting evidence in .agent-ladder/node-proof.md. +        The first line must be exactly `proof_for=edit`.";
    let mut workspace = ToolWorkspace::new(prompt);
    workspace
        .write_initial(source, "azure noon\nstale entry\n")
        .unwrap();
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..14 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &LADDER_TOOLS) else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            let id = format!("sequence-delivery-{turn}-{index}");
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                &id,
                &call.tool,
                &call.arguments,
            )]));
            messages.push(workspace.execute(&id, call));
        }
    }
    assert_eq!(workspace.read(source).unwrap(), "azure noon\ncobalt dawn\n");
    let effect = workspace.read("agent-ladder-effects/node.lino").unwrap();
    let proof = workspace.read(".agent-ladder/node-proof.md").unwrap();
    assert!(proof.starts_with("proof_for=edit\n"), "{proof}");
    let result = effect
        .lines()
        .find_map(|line| line.strip_prefix("result="))
        .expect("verified edits have a result");
    for evidence in [result, &proof] {
        assert!(evidence.contains("cobalt dawn"), "{evidence}");
        assert!(evidence.contains("stale entry"), "{evidence}");
    }
}

fn assert_nested_delivery_observes_source(prompt: &str) {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut workspace = ToolWorkspace::new(prompt);
    let declaration = "children: Vec<Self>";
    let observed = format!("src/task_decomposition.rs:79: pub {declaration}");

    for turn in 0..LADDER_TURN_CAP {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &LADDER_TOOLS) else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            let id = format!("nested-delivery-{turn}-{index}");
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                &id,
                &call.tool,
                &call.arguments,
            )]));
            let result = if call.tool == "grep" {
                ChatMessage::tool_result(id, &call.tool, &observed)
            } else {
                workspace.execute(&id, call)
            };
            messages.push(result);
        }
    }

    let proof = workspace
        .read(".audit/decomposition-proof.md")
        .expect("the nested delivery writes its proof");
    assert!(proof.starts_with("proof_for=decomposition\n"), "{proof}");
    assert!(proof.contains(declaration), "{proof}");

    let effect = workspace
        .read("audit-effects/decomposition.lino")
        .expect("the nested delivery writes its outer effect");
    assert!(
        effect
            .lines()
            .any(|line| line.starts_with("result=") && line.contains(declaration)),
        "the proof writer's status replaced the observed task result: {effect:?}",
    );
    assert!(!effect.contains("Recorded the findings"), "{effect}");
}
