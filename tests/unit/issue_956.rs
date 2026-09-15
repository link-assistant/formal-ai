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
    assert!(!answer.contains("all nine protocol primitives"), "{answer}");
    assert!(answer.contains("2 of 9 protocol primitives"), "{answer}");
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

fn inline_knowledge_base(prompt: &str) -> String {
    let messages = vec![ChatMessage::user(prompt)];
    let tools = ["web_search", "web_fetch", "write", "bash"];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("expected an artifact for the supplied source: {prompt}");
    };
    assert_eq!(
        calls[0].tool, "write",
        "supplied source must not trigger unrelated research: {:?}",
        calls[0]
    );
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    arguments["content"]
        .as_str()
        .expect("written knowledge-base bytes")
        .to_owned()
}

#[test]
fn ordinary_and_multilingual_quotes_bind_the_same_inline_source() {
    let source = "Observed input is not a guessed replacement.";
    for (open, close) in [
        ("\"", "\""),
        ("'", "'"),
        ("`", "`"),
        ("«", "»"),
        ("“", "”"),
        ("‘", "’"),
        ("「", "」"),
        ("『", "』"),
        ("《", "》"),
    ] {
        let task = format!("Formalize {open}{source}{close} into a Links Notation knowledge base.");
        let knowledge = inline_knowledge_base(&task);
        assert!(knowledge.contains(source), "{task}\n{knowledge}");
        assert!(
            !knowledge.contains("tale:fisherman-and-fish"),
            "{knowledge}"
        );
    }
}

#[test]
fn a_domain_word_does_not_replace_an_original_source_with_a_known_work() {
    for source in [
        "A fisherman records the source of each observation.",
        "Рыбак хранит исходные наблюдения отдельно от кэша.",
        "Сказка может быть примером исходного документа, а не результатом задачи.",
    ] {
        let knowledge = inline_knowledge_base(&format!(
            "Formalize «{source}» into a Links Notation knowledge base."
        ));
        assert!(knowledge.contains(source), "{knowledge}");
        assert!(
            !knowledge.contains("tale:fisherman-and-fish"),
            "{knowledge}"
        );
    }
}

#[test]
fn inline_source_order_is_not_the_order_of_delimiters_in_a_parser_table() {
    let knowledge = inline_knowledge_base(
        "Formalize “First source clause.” into a knowledge base with label «Secondary label».",
    );
    assert!(knowledge.contains("First source clause."), "{knowledge}");
    assert!(!knowledge.contains("Secondary label"), "{knowledge}");
}

#[test]
fn known_title_identity_survives_case_and_spacing_without_domain_word_guessing() {
    for prompt in [
        "Formalize «СКАЗКА  О РЫБАКЕ И РЫБКЕ» into a Links Notation knowledge base.",
        "Formalize “Сказка о рыбаке и рыбке” into a knowledge base with label «Reference».",
    ] {
        let messages = vec![ChatMessage::user(prompt)];
        let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
            panic!("expected the explicitly referenced known work");
        };
        assert!(answer.contains("tale:fisherman-and-fish"), "{answer}");
    }
}
