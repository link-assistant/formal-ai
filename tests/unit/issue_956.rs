//! Regressions for issue #956 (E104): `formal-ai agent --task` with a custom
//! formalization subject must formalize *that* text. Before the fix the planner
//! routed every formalization-keyword task to the seeded fairy-tale recipe and
//! silently substituted «Сказка о рыбаке и рыбке» for the supplied source.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

/// The exact repro from the audit: a quoted custom sentence, tool-less client.
/// The final answer must be about the supplied sentence, not the fairy tale.
#[test]
fn custom_quoted_task_formalizes_the_supplied_text() {
    let messages = vec![ChatMessage::user(
        "Formalize «The cat sat on the mat» into a Links Notation knowledge base.",
    )];
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
        panic!("expected a final answer");
    };
    assert!(answer.contains("The cat sat on the mat"), "{answer}");
    assert!(!answer.contains("рыбке"), "{answer}");
    assert!(answer.contains("doc:input"), "{answer}");
}

/// With tools advertised, the recipe must not search or fetch the canonical
/// tale: the task already carries its source text, so the first step writes a
/// knowledge base containing the supplied sentence.
#[test]
fn custom_quoted_task_skips_the_canonical_search_and_fetch() {
    let messages = vec![ChatMessage::user(
        "Formalize «The cat sat on the mat» into a Links Notation knowledge base.",
    )];
    let tools = ["web_search", "web_fetch", "write", "bash"];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("expected a planned tool call");
    };
    let call = &calls[0];
    assert_eq!(call.tool, "write", "planned call: {call:?}");
    assert!(
        call.arguments.contains("The cat sat on the mat"),
        "{call:?}"
    );
    assert!(!call.arguments.contains("рыб"), "{call:?}");
}

/// Multilingual repro per the issue: ru guillemets, zh 《…》 title marks, hi
/// text in guillemets. Each answer must reflect its own supplied sentence.
#[test]
fn custom_quoted_task_is_honored_in_every_language() {
    let prompts = [
        (
            "Формализуй «Кот сидел на коврике» в базу знаний Links Notation.",
            "Кот сидел на коврике",
        ),
        (
            "把《猫坐在垫子上》形式化为 Links Notation 知识库。",
            "猫坐在垫子上",
        ),
        (
            "«बिल्ली चटाई पर बैठी» को Links Notation knowledge base में formalize करें।",
            "बिल्ली चटाई पर बैठी",
        ),
    ];
    for (prompt, expected) in prompts {
        let messages = vec![ChatMessage::user(prompt)];
        let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
            panic!("expected a final answer for {prompt}");
        };
        assert!(answer.contains(expected), "{prompt}\n{answer}");
        assert!(!answer.contains("рыбке"), "{prompt}\n{answer}");
    }
}

/// The canonical task quotes the tale's *title*; a quoted title still selects
/// the full canonical tale rather than formalizing the five-word name.
#[test]
fn quoted_canonical_title_still_selects_the_full_tale() {
    let messages = vec![ChatMessage::user(
        "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.",
    )];
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
        panic!("expected a final answer");
    };
    assert!(answer.contains("«Сказка о рыбаке и рыбке»"), "{answer}");
    assert!(answer.contains("tale:fisherman-and-fish"), "{answer}");
}
