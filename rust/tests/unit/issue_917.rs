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
