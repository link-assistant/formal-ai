//! Transparent-state tests.
//!
//! `VISION.md`, `GOALS.md`, and `NON-GOALS.md` describe the network as a
//! transparent, user-inspectable substrate. Users can query the network
//! through chat, but the chat surface should not leak diagnostic ids or
//! internal state into the user-facing prose by default.

use formal_ai::{knowledge_links_notation, FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations.
// ---------------------------------------------------------------------------

#[test]
fn knowledge_export_is_inspectable_at_runtime() {
    let notation = knowledge_links_notation();
    assert!(notation.contains("formal_ai_knowledge"));
}

#[test]
fn evidence_links_are_exposed_through_the_public_struct() {
    let response = answer("Hi");
    assert!(!response.evidence_links.is_empty());
}

#[test]
fn links_notation_trace_is_always_present() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
}

// ---------------------------------------------------------------------------
// MVP expectations.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "MVP-target: querying 'show me the network' should return a Links Notation snapshot"]
fn querying_the_network_returns_snapshot() {
    let response = answer("Show me the current network");
    assert!(
        response.answer.contains("formal_ai_knowledge") || response.answer.contains("```links"),
        "the user should be able to inspect the network via chat"
    );
}

#[test]
#[ignore = "MVP-target: 'what do you know about X' should list links involving X"]
fn querying_a_concept_returns_its_links() {
    let response = answer("What do you know about 'greeting'?");
    assert!(
        response.answer.contains("intent") && response.answer.contains("greeting"),
        "the user should be able to introspect a concept by name"
    );
}

#[test]
#[ignore = "MVP-target: diagnostic link ids must not leak into default chat prose"]
fn diagnostic_ids_never_leak_into_default_chat_prose() {
    let response = answer("Hi");
    let lower = response.answer.to_lowercase();
    assert!(
        !lower.contains("prompt:") && !lower.contains("intent:") && !lower.contains("trace:"),
        "default chat answers must not show internal link ids in user-facing prose"
    );
}

#[test]
#[ignore = "MVP-target: the user must be able to opt-in to diagnostic mode and see all links inline"]
fn diagnostic_mode_can_be_enabled_per_message() {
    let response = answer("[diagnostic] Hi");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("trace:") || lower.contains("evidence:"),
        "explicit diagnostic mode must include trace and evidence links inline"
    );
}

#[test]
#[ignore = "MVP-target: 'why' meta-questions should explain the previous answer using links"]
fn why_meta_question_explains_previous_answer() {
    let _ = answer("Hi");
    let response = answer("Why did you answer that?");
    assert!(
        response.intent == "meta_explanation" || response.intent == "explanation",
        "why-questions should resolve to a meta-explanation intent"
    );
    assert!(response.answer.contains("because") || response.answer.contains("evidence"));
}

#[test]
#[ignore = "MVP-target: 'forget X' must be refused unless the user invokes the explicit retraction protocol"]
fn forget_request_requires_explicit_retraction_protocol() {
    let response = answer("Forget the greeting concept");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:add_only_history"),
        "the network is append-only; retraction must use a documented protocol"
    );
}

#[test]
#[ignore = "MVP-target: 'export network' should return a downloadable Links Notation snapshot"]
fn export_network_returns_links_notation_snapshot() {
    let response = answer("Export the network");
    assert!(response.answer.contains("```links") || response.answer.contains("links-notation"));
    assert!(response.answer.contains("formal_ai_knowledge"));
}

#[test]
#[ignore = "MVP-target: 'list my facts' should return only facts the current user contributed"]
fn list_my_facts_filters_by_user() {
    let response = answer("List the facts I have contributed");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("filter:user")),
        "personal queries must declare a user filter"
    );
}
