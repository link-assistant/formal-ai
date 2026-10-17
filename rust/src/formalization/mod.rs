//! Deep formalization: segmentation, need emission, concept grounding,
//! procedure extraction and the concept graph (issue #1138, plan 04).
//!
//! The formalizer takes plan 01's `SourceLookup` rather than a transport, so it
//! owns no cache policy, no settings reading and no source list of its own. A
//! document with an unresolved need can never be reported as covered.

pub mod concept_links;
pub mod concepts;
pub mod dialogue;
pub mod needs;
pub mod page;
pub mod procedures;
pub mod segment;
pub mod source_qualified_definition;
pub mod statement_rendering;
pub mod text_statements;
pub mod typed_meaning;

/// Exact language-neutral grammar facts shared by formalization and translation.
#[must_use]
pub fn structural_rule_values(rule: &str, kind: &str) -> Vec<String> {
    let root = text_statements::seed_record().expect("shared formalization grammar");
    let records = root
        .children
        .iter()
        .filter(|node| node.name == "structural-rule" && node.id == rule)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "unique structural grammar rule");
    let values = records[0]
        .children
        .iter()
        .filter(|node| node.name == kind)
        .map(|node| node.id.trim_matches('"').to_owned())
        .collect::<Vec<_>>();
    assert!(
        !values.is_empty() && values.iter().all(|value| !value.is_empty()),
        "nonempty structural grammar facts"
    );
    assert_eq!(
        values
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        values.len(),
        "unique structural grammar facts"
    );
    values
}

/// The shared schema prefix for resolved or losslessly unresolved numerals.
#[must_use]
pub fn numeric_identity_prefix(unresolved: bool) -> String {
    let values = structural_rule_values(
        "numeric-identity",
        if unresolved {
            "unresolved-prefix"
        } else {
            "resolved-prefix"
        },
    );
    assert_eq!(values.len(), 1, "unique numeric identity prefix");
    values[0].clone()
}
