//! Issue #1138 B8 (plan 08, L3): identity is a procedure, not an answer.
//!
//! Two problems with the same narrative shape and different numbers share a task
//! identity, so a recall replays the *derivation* and recomputes. If a literal
//! value entered the identity, a renumbered paraphrase would miss the ledger and
//! the generalization claim would be untestable; if the identity carried the
//! value, memorization would pass.

use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec};
use formal_ai::verifiable_task::recognise_verifiable;

use super::{LANGUAGES, case};

/// Renumbering the narrative changes the answer, never the procedure.
#[test]
fn a_renumbered_paraphrase_shares_the_task_identity() {
    let original = case("arithmetic_narrative", "en");
    let renumbered = original
        .prompt
        .replace("24", "31")
        .replace("18", "27")
        .replace("35", "44")
        .replace('4', "6");

    let first =
        recognise_verifiable(&original.prompt).expect("the seeded narrative must be recognised");
    let second =
        recognise_verifiable(&renumbered).expect("the renumbered narrative must be recognised");

    assert_ne!(
        original.prompt, renumbered,
        "the two prompts must actually differ"
    );
    assert_eq!(
        first.identity(),
        second.identity(),
        "two narratives of the same shape share one procedure identity"
    );
}

/// No stated quantity may appear in the inspectable identity material, in any
/// language. The final identity is a hexadecimal digest, so searching it for a
/// one-digit value would confuse a random digest nibble with retained input.
#[test]
fn identity_carries_no_literal_value() {
    for language in LANGUAGES {
        let paraphrase = case("arithmetic_narrative", language);
        let task = recognise_verifiable(&paraphrase.prompt)
            .unwrap_or_else(|| panic!("{language}: the narrative must be recognised"));
        let identity = task.identity();
        let payload = task.identity_payload();
        assert!(
            !identity.is_empty(),
            "{language}: a recognised task has an identity"
        );
        for quantity in &task.quantities {
            assert!(
                !payload
                    .split(|character: char| !character.is_alphanumeric())
                    .any(|token| token == quantity.value),
                "{language}: identity material may not carry the literal value {}: {payload:?}",
                quantity.value
            );
        }
    }
}

/// The serialised task is inspectable evidence: it names the expectation and
/// quantity offsets while preserving the original prompt outside the stable id.
#[test]
fn links_projection_exposes_the_task_shape_and_evidence_offsets() {
    let paraphrase = case("named_unknown", "en");
    let task = recognise_verifiable(&paraphrase.prompt).expect("the equation is recognised");
    let links = task.to_links_notation();

    assert!(links.starts_with("verifiable_task\n"));
    assert!(links.contains("expectation \"unknown\""));
    assert!(links.contains("answer_shape \"delimited_value\""));
    assert!(links.contains("offset "));
    assert!(links.contains(&task.identity()));
}

/// Every non-coding expectation projects onto the same executable program
/// shape consumed by coding discovery; it does not create a parallel IR.
#[test]
fn a_verifiable_task_projects_onto_the_shared_program_spec() {
    for family in [
        "arithmetic_narrative",
        "counted_category",
        "instructed_edit",
        "named_unknown",
        "unit_conversion",
    ] {
        let paraphrase = case(family, "en");
        let task = recognise_verifiable(&paraphrase.prompt)
            .unwrap_or_else(|| panic!("{family}: task must be recognised"));
        let spec = CodingTaskSpec::from(&task);
        assert_eq!(spec.artifact_shape, ArtifactShape::Program, "{family}");
        assert_eq!(spec.name, "main", "{family}");
        assert_eq!(spec.prose_language, task.prose_language, "{family}");
        assert_eq!(
            spec.requirement_sentences, task.requirement_sentences,
            "{family}"
        );
    }
}
