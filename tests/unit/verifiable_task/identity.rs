//! Issue #1138 B8 (plan 08, L3): identity is a procedure, not an answer.
//!
//! Two problems with the same narrative shape and different numbers share a task
//! identity, so a recall replays the *derivation* and recomputes. If a literal
//! value entered the identity, a renumbered paraphrase would miss the ledger and
//! the generalization claim would be untestable; if the identity carried the
//! value, memorization would pass.

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

    let first = recognise_verifiable(&original.prompt)
        .expect("the seeded narrative must be recognised");
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

/// No stated quantity may appear in the identity, in any language.
#[test]
fn identity_carries_no_literal_value() {
    for language in LANGUAGES {
        let paraphrase = case("arithmetic_narrative", language);
        let task = recognise_verifiable(&paraphrase.prompt)
            .unwrap_or_else(|| panic!("{language}: the narrative must be recognised"));
        let identity = task.identity();
        assert!(
            !identity.is_empty(),
            "{language}: a recognised task has an identity"
        );
        for quantity in &task.quantities {
            assert!(
                !identity.contains(&quantity.value),
                "{language}: the identity may not carry the literal value {}: {identity}",
                quantity.value
            );
        }
    }
}
