//! Links-network level tests.
//!
//! `VISION.md` defines the network as the AI itself: every fact, rule and
//! intent is a Links Notation record. These tests pin down structural
//! properties of the network: doublet links, dynamic type system, add-only
//! history, concept uniqueness, and trace records.

use formal_ai::{knowledge_links_notation, FormalAiEngine, SymbolicAnswer};
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
    assert!(!notation.is_empty());
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
    assert!(!response.links_notation.is_empty());
    let (id, _root) =
        parse_indented(&response.links_notation).expect("trace should be valid Links Notation");
    assert!(id.starts_with("answer_"));
}

#[test]
fn every_answer_records_intent_in_evidence_links() {
    let response = answer("Hi");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("intent:")));
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
#[ignore = "tracked requirement: dynamic type system should publish Type -> SubType chains in the network"]
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
#[ignore = "tracked requirement: facts should be recorded with the source link that introduced them"]
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
#[ignore = "tracked requirement: every answer must publish a trace link pointing to a reasoning record"]
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
#[ignore = "tracked requirement: trace records must list ordered reasoning steps in Links Notation"]
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
