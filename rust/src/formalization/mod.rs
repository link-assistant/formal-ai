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
    let root = grammar_root().expect("shared formalization grammar");
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

/// The parsed shared grammar, retaining the original embedded seed custody.
#[must_use]
pub fn grammar_root() -> Option<&'static crate::seed::parser::LinoNode> {
    static ROOT: std::sync::OnceLock<crate::seed::parser::LinoNode> = std::sync::OnceLock::new();
    grammar_root_from(
        ROOT.get_or_init(|| crate::seed::parse_lino(crate::seed::TEXT_FORMALIZATION_LINO)),
    )
}

fn grammar_root_from(
    root: &crate::seed::parser::LinoNode,
) -> Option<&crate::seed::parser::LinoNode> {
    if root.name == "text-formalization" {
        return Some(root);
    }
    root.children
        .iter()
        .find(|node| node.name == "text-formalization")
}

/// Raw owned vocabulary; normalization belongs to the requesting formalizer.
#[must_use]
pub fn grammar_vocabulary(language: &str, kind: &str) -> Vec<String> {
    grammar_root().map_or_else(Vec::new, |root| {
        grammar_vocabulary_from(root, language, kind)
    })
}

fn grammar_vocabulary_from(
    root: &crate::seed::parser::LinoNode,
    language: &str,
    kind: &str,
) -> Vec<String> {
    let Some(record) = root
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)
    else {
        return Vec::new();
    };
    let local: Vec<_> = record
        .children
        .iter()
        .filter(|node| node.name == kind)
        .collect();
    let values = if local.is_empty() {
        root.children
            .iter()
            .find(|node| node.name == "defaults")
            .map(|defaults| {
                defaults
                    .children
                    .iter()
                    .filter(|node| node.name == kind)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        local
    };
    values
        .into_iter()
        .map(|node| node.id.trim_matches('"').to_owned())
        .collect()
}

#[cfg(test)]
#[path = "../../tests/source/source_tests/formalization/grammar_native_parity.rs"]
mod grammar_native_parity;
