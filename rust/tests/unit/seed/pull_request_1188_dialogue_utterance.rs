//! A greeting is not a bare term for the open web. The opencode leg of the
//! Agentic CLI Matrix (CI run 37710079108) sent "hi" with eleven tools
//! advertised; the capability table read it as `(bare_term, retrieve, web)`,
//! searched the web for it and read a dictionary page before answering. A
//! request made only of the surfaces of the seeded `dialogue_utterance_role`s
//! is now left to the engine. The JavaScript twin is pinned by
//! `rust/tests/web/pull-request-1188-dialogue-utterance.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::capability_routing::{dialogue_utterance_roles, is_dialogue_utterance};
use formal_ai::protocol::ChatMessage;

const TOOLS: [&str; 11] = [
    "bash",
    "edit",
    "glob",
    "grep",
    "read",
    "skill",
    "task",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];

fn plan_for(text: &str) -> Option<AgenticPlan> {
    plan_chat_step(&[ChatMessage::user(text)], &TOOLS)
}

#[test]
fn the_seed_names_dialogue_roles_each_with_surfaces() {
    let roles = dialogue_utterance_roles();
    assert!(
        roles.iter().any(|role| role == "social_greeting"),
        "{roles:?}"
    );
    for role in &roles {
        assert!(
            !formal_ai::seed::lexicon().words_for_role(role).is_empty(),
            "{role} has no surface in the seed"
        );
    }
}

#[test]
fn every_greeting_farewell_and_thanks_surface_plans_no_tool_call() {
    for role in [
        "social_greeting",
        "social_farewell",
        "social_courtesy_response",
    ] {
        for surface in formal_ai::seed::lexicon().words_for_role(role) {
            assert!(
                is_dialogue_utterance(&surface),
                "{surface} is not a dialogue act"
            );
            assert!(
                plan_for(&surface).is_none(),
                "{surface} planned a tool call"
            );
        }
    }
}

#[test]
fn a_request_beyond_the_greeting_still_reaches_research() {
    assert!(!is_dialogue_utterance("hi, what is the weather in Paris"));
    assert!(!is_dialogue_utterance("rust ownership"));
    match plan_for("hi, what is the weather in Paris") {
        Some(AgenticPlan::ToolCalls(calls)) => assert_eq!(calls[0].tool, "websearch"),
        other => panic!("expected a websearch, got {other:?}"),
    }
}
