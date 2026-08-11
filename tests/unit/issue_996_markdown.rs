//! Regressions for issue #996 (hive-mind #2146): a final message that inlines
//! machine text must fence it. Final answers travel into GitHub comments,
//! where unfenced space-indented Links Notation collapses into flowing prose
//! and the plan/knowledge-base dump becomes unreadable.

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};

/// The content of the first fenced `lino` block in `answer`.
fn lino_block(answer: &str) -> &str {
    let start = answer.find("```lino\n").expect("opening lino fence");
    let body = &answer[start + "```lino\n".len()..];
    let end = body.find("\n```").expect("closing lino fence");
    &body[..end]
}

/// The repository work-item shape from the captured hive-mind run: the answer
/// quotes the recorded plan event, which must arrive as one fenced block.
#[test]
fn planned_not_executed_answer_fences_the_plan_event() {
    let plan = compose_general_change_plan(
        "Issue to solve: https://github.com/link-assistant/formal-ai/issues/996\n\
         Implement the fix in the repository and verify it with tests.",
    )
    .expect("repository work-item plan");

    let answer = plan.planned_not_executed_answer();
    let block = lino_block(&answer);
    assert!(block.contains("general_change_plan"), "{answer}");
    assert!(block.contains("planned_not_executed"), "{answer}");
}

/// The executed-and-verified answer quotes the same plan event and must fence
/// it too.
#[test]
fn completed_answer_fences_the_plan_event() {
    let task = "Execute the auto-learning task. Run 'printf learned-output' and write its exact \
                stdout to reports/learned.txt";
    let tools = ["write", "bash"];
    let mut messages = vec![ChatMessage::user(task)];

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
        let id = format!("command-output-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            &id,
            &call.tool,
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }

    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("executed plan must end in a final answer");
    };
    assert!(
        answer.contains("Completed the general change request"),
        "{answer}"
    );
    assert!(
        lino_block(&answer).contains("general_change_plan"),
        "{answer}"
    );
}

/// The canonical formalization answer inlines the knowledge base; it must be a
/// fenced block instead of hundreds of indented lines.
#[test]
fn formalization_answer_fences_the_knowledge_base() {
    let messages = vec![ChatMessage::user("formalize the fisherman tale")];
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
        panic!("expected a final answer");
    };
    assert!(lino_block(&answer).contains("knowledge_base"), "{answer}");
}
