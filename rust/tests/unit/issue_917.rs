use std::collections::BTreeSet;

use formal_ai::FormalAiEngine;
use formal_ai::translation::{formal_language_targets, translate_statement};

const FORMAL_STATEMENT: &str = "P31(Q89, Q3314483)";
const NATURAL_STATEMENTS: [(&str, &str); 5] = [
    ("en", "apple is a fruit"),
    ("ru", "яблоко это фрукт"),
    ("hi", "सेब फल है"),
    ("zh", "苹果是水果"),
    ("es", "manzana es una fruta"),
];

#[test]
fn every_seed_language_round_trips_through_a_seeded_formal_target() {
    let exercised = NATURAL_STATEMENTS
        .iter()
        .map(|(language, _)| (*language).to_owned())
        .collect::<BTreeSet<_>>();
    let registered = formal_ai::language::registered_languages()
        .iter()
        .map(|language| language.slug().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(exercised, registered);
    assert_eq!(formal_language_targets(), vec!["fol"]);

    for (source_language, natural_statement) in NATURAL_STATEMENTS {
        let formal = translate_statement(natural_statement, source_language, "fol")
            .expect("the seeded statement should formalize");
        assert_eq!(formal.surface, FORMAL_STATEMENT);
        assert_eq!(formal.meaning, "statement:P31(Q89,Q3314483)");

        for (target_language, expected_statement) in NATURAL_STATEMENTS {
            let natural = translate_statement(FORMAL_STATEMENT, "fol", target_language)
                .expect("the formal statement should naturalize");
            assert_eq!(natural.surface, expected_statement);
            assert_eq!(natural.meaning, formal.meaning);
        }
    }
}

#[test]
fn whole_task_translation_uses_the_formal_projection_in_both_directions() {
    let formalized = FormalAiEngine.answer("Translate `apple is a fruit` from English to FOL.");
    assert_eq!(formalized.answer, "P31(Q89, Q3314483)");
    assert_eq!(formalized.intent, "translate_en_to_fol");
    assert_eq!(
        formalized
            .evidence_links
            .iter()
            .find(|link| link.starts_with("meaning:"))
            .map(String::as_str),
        Some("meaning:statement:P31(Q89,Q3314483)")
    );

    let naturalized = FormalAiEngine.answer("Translate `P31(Q89, Q3314483)` from FOL to Russian.");
    assert_eq!(naturalized.answer, "яблоко это фрукт");
    assert_eq!(naturalized.intent, "translate_fol_to_ru");
    assert_eq!(
        naturalized
            .evidence_links
            .iter()
            .find(|link| link.starts_with("meaning:")),
        formalized
            .evidence_links
            .iter()
            .find(|link| link.starts_with("meaning:"))
    );

    let spanish_formalized =
        FormalAiEngine.answer("Translate `manzana es una fruta` from Spanish to FOL.");
    assert_eq!(spanish_formalized.answer, "P31(Q89, Q3314483)");
    let spanish_naturalized =
        FormalAiEngine.answer("Translate `P31(Q89, Q3314483)` from FOL to Spanish.");
    assert_eq!(spanish_naturalized.answer, "manzana es una fruta");
    assert_eq!(
        spanish_naturalized
            .evidence_links
            .iter()
            .find(|link| link.starts_with("meaning:")),
        spanish_formalized
            .evidence_links
            .iter()
            .find(|link| link.starts_with("meaning:"))
    );
}

#[test]
fn formal_projection_rejects_ids_in_the_wrong_semantic_roles() {
    assert!(translate_statement("Q89(P31, Q3314483)", "fol", "en").is_err());
    assert!(translate_statement("P31(Q3314483, P31)", "fol", "en").is_err());
}

/// R917-2 and R917-5: the formal targets and the natural word orders are one
/// seed catalog, and the interpreter that reads it names no projection of its
/// own. Every slug the seed declares is the whole list the engine exposes,
/// every natural projection the seed declares renders the formal statement,
/// and no slug appears as a literal in the interpreter, so adding another
/// target or natural projection is a change to the seed file alone.
#[test]
fn projection_catalog_is_seed_data_and_the_interpreter_names_no_projection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits above the crate");
    let catalog = std::fs::read_to_string(root.join("data/seed/formal-language-projections.lino"))
        .expect("the projection catalog is seed data");
    let declared = |kind: &str| -> Vec<String> {
        catalog
            .lines()
            .filter_map(|line| line.trim().strip_prefix(kind))
            .map(|rest| rest.trim().trim_matches('"').to_owned())
            .collect()
    };
    let formal = declared("formal_language ");
    let natural = declared("natural_language ");
    assert_eq!(formal_language_targets(), formal);
    assert_eq!(
        natural.iter().cloned().collect::<BTreeSet<_>>(),
        NATURAL_STATEMENTS
            .iter()
            .map(|(language, _)| (*language).to_owned())
            .collect::<BTreeSet<_>>()
    );
    for target in &formal {
        for language in &natural {
            let rendered = translate_statement(FORMAL_STATEMENT, target, language)
                .expect("every seeded natural projection renders the statement");
            let back = translate_statement(&rendered.surface, language, target)
                .expect("every seeded natural projection parses back");
            assert_eq!(back.meaning, rendered.meaning);
        }
    }

    let interpreter =
        std::fs::read_to_string(root.join("rust/src/translation/formal_statement.rs"))
            .expect("the projection interpreter is readable");
    for slug in formal.iter().chain(&natural) {
        assert!(
            !interpreter.contains(&format!("\"{slug}\"")),
            "formal_statement.rs names the projection `{slug}`; it must come from the seed catalog"
        );
    }
}
