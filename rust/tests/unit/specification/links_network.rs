//! Links-network level tests.
//!
//! `VISION.md` defines the network as the AI itself: every fact, rule and
//! intent is a Links Notation record. These tests pin down structural
//! properties of the network: doublet links, dynamic type system, add-only
//! history, concept uniqueness, and trace records.

use formal_ai::{FormalAiEngine, SymbolicAnswer, knowledge_links_notation};
use lino_objects_codec::format::parse_indented;

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: the knowledge export already speaks Links Notation.
// ---------------------------------------------------------------------------

#[test]
fn knowledge_export_is_non_empty_links_notation() {
    let notation = knowledge_links_notation();
    assert_ne!(notation, "");
    assert!(notation.contains("formal_ai_knowledge"));
}

#[test]
fn knowledge_records_parse_as_links_notation() {
    let notation = knowledge_links_notation();
    for record in notation.split("\n\n").filter(|chunk| !chunk.is_empty()) {
        parse_indented(record)
            .unwrap_or_else(|err| panic!("record {record:?} should parse: {err:?}"));
    }
}

#[test]
fn every_answer_includes_links_notation_trace() {
    let response = answer("Hi");
    assert_ne!(response.links_notation, "");
    let (id, _root) =
        parse_indented(&response.links_notation).expect("trace should be valid Links Notation");
    assert!(id.starts_with("answer_"));
}

#[test]
fn every_answer_records_intent_in_evidence_links() {
    let response = answer("Hi");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("intent:"))
    );
}

#[test]
fn distinct_prompts_lead_to_distinct_links_traces() {
    let first = answer("Hi");
    let second = answer("Who are you?");
    assert_ne!(first.links_notation, second.links_notation);
}

#[test]
fn knowledge_export_uses_untyped_links_notation_only() {
    let notation = knowledge_links_notation();
    assert!(
        !notation.contains("(str ") && !notation.contains("(int "),
        "knowledge dataset must be untyped Links Notation"
    );
}

// ---------------------------------------------------------------------------
// full-scope expectations: structural properties from VISION.md / REQUIREMENTS.md.
// ---------------------------------------------------------------------------

#[test]
fn knowledge_export_is_reducible_to_doublet_links() {
    let notation = knowledge_links_notation();
    assert!(
        notation.contains("doublets") || notation.contains("from") && notation.contains("to"),
        "the knowledge export must explicitly expose its doublet reduction"
    );
}

#[test]
fn dynamic_type_system_publishes_subtype_chains() {
    let notation = knowledge_links_notation();
    assert!(
        notation.contains("Type") && notation.contains("SubType"),
        "the network should expose Type -> SubType -> Value chains"
    );
}

#[test]
fn concepts_are_unique_and_referenced_by_id() {
    let notation = knowledge_links_notation();
    let greeting_occurrences = notation.matches("intent: greeting").count();
    assert_eq!(
        greeting_occurrences, 1,
        "greeting concept should be defined once and referenced by id, not duplicated"
    );
}

#[test]
fn history_is_append_only() {
    let before = knowledge_links_notation();
    let _ = answer("Hi");
    let after = knowledge_links_notation();
    assert!(
        after.starts_with(&before),
        "subsequent answers must only append; existing records must not change"
    );
}

#[test]
fn every_fact_carries_a_source_link() {
    let notation = knowledge_links_notation();
    for record in notation.split("\n\n").filter(|chunk| !chunk.is_empty()) {
        if record.contains("fact") {
            assert!(
                record.contains("source"),
                "fact records must carry a source link, got: {record}"
            );
        }
    }
}

#[test]
fn every_answer_has_a_trace_link_pointer() {
    let response = answer("Hi");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:")),
        "answers must carry a trace link pointing to the reasoning steps"
    );
}

#[test]
fn trace_record_lists_ordered_reasoning_steps() {
    let response = answer("Write me hello world program in Rust");
    assert!(
        response.links_notation.contains("step") || response.links_notation.contains("reasoning"),
        "trace must enumerate ordered reasoning steps"
    );
}

#[test]
fn knowledge_dataset_declares_schema_version() {
    let notation = knowledge_links_notation();
    assert!(
        notation.contains("schema_version") || notation.contains("dataset_version"),
        "the dataset should declare a schema/dataset version for migration safety"
    );
}

#[test]
fn records_are_addressable_by_stable_id() {
    let response = answer("Hi");
    let (id, _root) = parse_indented(&response.links_notation).unwrap();
    let other = answer("Hi");
    let (other_id, _) = parse_indented(&other.links_notation).unwrap();
    assert_eq!(
        id, other_id,
        "identical prompts must produce identical, content-addressed trace ids"
    );
}

#[test]
fn ill_formed_links_notation_input_is_rejected() {
    let response = answer("teach this fact: ((((((( unbalanced");
    assert_eq!(response.intent, "unknown");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("error:")),
        "malformed teach-the-network inputs must surface a parser error link"
    );
}

// Keep source-qualified parser fixtures in the integration test harness.
#[path = "../../fixtures/source-qualified-definition-slots.rs"]
mod source_qualified_definition_slots;

#[test]
fn answer_record_keeps_six_ordered_fields_and_response_link_thinking() {
    let response = answer("Hi");
    assert_eq!(response.answer, "Hi, how may I help you?");
    let lines: Vec<_> = response.links_notation.lines().take(7).collect();
    assert_eq!(lines[0], "answer_prompt_09275f07b5bb95ba");
    assert_eq!(lines[1], "  prompt \"Hi\"");
    assert_eq!(lines[2], "  intent \"greeting\"");
    assert_eq!(lines[3], "  answer \"Hi, how may I help you?\"");
    assert!(lines[4].starts_with("  trace \"trace_"));
    assert!(lines[5].starts_with("  steps "));
    assert!(lines[6].starts_with("  thinking_steps "));
    let (_, fields) = parse_indented(&response.links_notation).expect("actual answer record");
    assert_eq!(fields.len(), 6);
    assert!(
        fields
            .get("steps")
            .expect("steps field")
            .starts_with("step_0 impulse Hi;")
    );
    let thinking = fields.get("thinking_steps").expect("thinking steps field");
    assert!(thinking.starts_with("step_0 impulse high impulse Hi;"));
    assert!(thinking.ends_with("deformalize high response response:greeting"));
    assert_eq!(
        response.thinking_steps.last().unwrap().detail,
        response.answer
    );
}

#[test]
fn answer_record_retains_actual_prior_turns_before_the_current_impulse() {
    use formal_ai::{ConversationTurn, solve_with_history};
    let prior = answer("Hi");
    assert_eq!(prior.answer, "Hi, how may I help you?");
    let history = [
        ConversationTurn::user("Hi"),
        ConversationTurn::assistant(&prior.answer),
    ];
    let response = solve_with_history("What is 2 + 2?", &history);
    assert_eq!(response.answer, "2 + 2 = 4");
    let (_, fields) =
        parse_indented(&response.links_notation).expect("actual history answer record");
    let steps = fields.get("steps").expect("steps field");
    assert!(steps.starts_with("step_0 prior_turn:user Hi; step_1 prior_turn:assistant Hi, how may I help you?; step_2 impulse What is 2 + 2?;"));
    assert!(steps.contains("calculation:engine link-calculator;"));
}
