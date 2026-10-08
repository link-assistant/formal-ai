//! R1188-U18: page formalization.
//!
//! Every sentence of a page becomes formal statements, nothing is dropped
//! silently, and the statements deformalize back to the same known terms. The
//! JavaScript twin is `rust/tests/web/web-formalization.test.mjs`; both pin the
//! same outputs of the shared text formalizer.

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use formal_ai::formalization::page::formalize_page;
use formal_ai::formalization::statement_rendering::deformalize_statement;
use formal_ai::formalization::text_statements::{
    FUNCTION_WORD_ROLES, NEGATION_ROLE, Token, formalize_text, indexed_meaning, phrase_key,
    statement_identity, tokens,
};
use formal_ai::seed::{lexicon, parse_lino};

fn identities(text: &str, language: &str) -> Vec<String> {
    formalize_text(text, language)
        .iter()
        .map(statement_identity)
        .collect()
}

fn token(surface: &str, han: bool) -> Token {
    Token {
        surface: surface.to_owned(),
        han,
    }
}

#[test]
fn tokens_keep_apostrophes_and_numbers_whole_and_set_han_runs_apart() {
    assert_eq!(
        tokens("Earth's orbit is 378,000 km; 1.2 billion. 地球的卫星（月球）"),
        vec![
            token("Earth's", false),
            token("orbit", false),
            token("is", false),
            token("378,000", false),
            token("km", false),
            token("1.2", false),
            token("billion", false),
            token("地球的卫星", true),
            token("月球", true),
        ]
    );
}

#[test]
fn an_anaphor_and_a_clause_continuation_keep_the_subject_and_a_negation_cue_denies() {
    assert_eq!(
        identities(
            "The Moon orbits Earth. It does not emit light, and it has 1.2 billion craters.",
            "en"
        ),
        vec![
            "asserted|moon|name:earth unknown:orbit",
            "denied|moon|file_whole_write_action unknown:light",
            "asserted|moon|number:1.2 unknown:billion unknown:crater",
        ]
    );
    assert_eq!(
        identities("Луна вращается вокруг Земли. Она не светится.", "ru"),
        vec![
            "asserted|moon|name:земли unknown:вокруг unknown:враща",
            "denied|moon|unknown:свет",
        ]
    );
}

#[test]
fn every_language_reads_through_the_same_lexicon() {
    assert_eq!(
        identities("सेब एक फल है।", "hi"),
        vec!["asserted|wikidata_item_apple|wikidata_item_fruit"]
    );
    assert_eq!(
        identities("La manzana es una fruta.", "es"),
        vec!["asserted|wikidata_item_apple|wikidata_item_fruit wikidata_property_instance_of"]
    );
    assert_eq!(
        identities("我喜欢苹果。", "zh"),
        vec!["asserted|software_object_lead_word|unknown:喜欢 wikidata_item_apple"]
    );
}

/// PR #1188 (LEXEMES): the surface count spans every language, so one meaning
/// holds a shared word in each language, and the rank still tells meanings
/// apart once every meaning is written in all five languages.
#[test]
fn a_shared_surface_goes_to_more_languages_then_fewer_surfaces_then_declaration_order() {
    assert_eq!(
        indexed_meaning("apple", "en").as_deref(),
        Some("wikidata_item_apple")
    );
    assert_eq!(
        indexed_meaning("सेब", "hi").as_deref(),
        Some("wikidata_item_apple")
    );
    // A rank orders lower first: more languages, fewer surfaces, earlier declared.
    let mut ranks: HashMap<String, (Reverse<usize>, usize, usize, String)> = HashMap::new();
    for (order, meaning) in lexicon().meanings.iter().enumerate() {
        let excluded = FUNCTION_WORD_ROLES
            .iter()
            .any(|role| meaning.has_role(role))
            || meaning.has_role(NEGATION_ROLE);
        if excluded {
            continue;
        }
        let languages: BTreeSet<&str> = meaning
            .lexemes
            .iter()
            .map(|lexeme| lexeme.language.as_str())
            .collect();
        let size: usize = meaning
            .lexemes
            .iter()
            .map(|lexeme| lexeme.words.len())
            .sum();
        let Some(english) = meaning
            .lexemes
            .iter()
            .find(|lexeme| lexeme.language == "en")
        else {
            continue;
        };
        for word in &english.words {
            let Some(key) = phrase_key(&word.text) else {
                continue;
            };
            let rank = (Reverse(languages.len()), size, order, meaning.slug.clone());
            let better = ranks
                .get(&key)
                .is_none_or(|held| (&rank.0, rank.1, rank.2) < (&held.0, held.1, held.2));
            if better {
                ranks.insert(key, rank);
            }
        }
    }
    for (key, rank) in ranks {
        assert_eq!(indexed_meaning(&key, "en"), Some(rank.3), "{key}");
    }
}

#[test]
fn a_sentence_opening_word_is_a_name_only_when_the_text_writes_it_as_one_elsewhere() {
    assert_eq!(
        identities("Earth is orbited by the Moon. The Moon orbits Earth.", "en"),
        vec![
            "asserted|name:earth|moon unknown:orbit",
            "asserted|moon|name:earth unknown:orbit",
        ]
    );
    assert_eq!(
        identities("Yesterday the Moon rose.", "en"),
        vec!["asserted|unknown:yesterday|moon unknown:rose"]
    );
}

#[test]
fn a_statement_deformalizes_subject_first_in_any_language() {
    let statements = formalize_text("The Moon does not orbit Earth.", "en");
    let statement = statements.first().expect("one statement");
    assert_eq!(
        deformalize_statement(statement, "en"),
        "Moon not orbit Earth"
    );
    assert_eq!(
        deformalize_statement(statement, "ru"),
        "Луна не orbit Earth"
    );
    assert_eq!(deformalize_statement(statement, "zh"), "月球不 orbit Earth");
}

#[test]
fn a_page_reports_every_sentence_its_unknown_words_and_its_surviving_facts() {
    let report = formalize_page(
        "The Moon orbits Earth. It does not emit light, and it has 1.2 billion craters. La la la.",
        "en",
    );
    assert_eq!(
        (
            report.covered,
            report.statements,
            report.factual,
            report.survived,
            report.terms,
            report.unknown
        ),
        (0, 4, 3, 3, 13, 7)
    );
    let unknown: Vec<Vec<String>> = report
        .sentences
        .iter()
        .map(|sentence| sentence.unknown.clone())
        .collect();
    assert_eq!(
        unknown,
        vec![
            vec!["orbits".to_owned()],
            vec![
                "light".to_owned(),
                "billion".to_owned(),
                "craters".to_owned()
            ],
            vec!["La".to_owned(), "la".to_owned(), "la".to_owned()],
        ]
    );
}

#[test]
fn a_sentence_whose_terms_are_all_known_is_covered_and_its_fact_survives() {
    let report = formalize_page("सेब एक फल है।", "hi");
    assert_eq!((report.covered, report.survived, report.unknown), (1, 1, 0));
}

#[test]
fn the_cached_corpus_holds_the_same_topics_in_every_language() {
    for language in ["en", "ru", "hi", "zh", "es"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data/benchmarks/web-formalization")
            .join(format!("{language}.lino"));
        let text = fs::read_to_string(&path).expect("the cached corpus is committed");
        let root = parse_lino(&text);
        let pages: Vec<_> = root
            .children
            .iter()
            .filter(|node| node.name == "web-formalization-page")
            .collect();
        assert_eq!(pages.len(), 8, "{language}");
        for page in pages {
            assert!(!page.find_child_value("source-url").is_empty());
            assert!(!page.find_child_value("revision-id").is_empty());
        }
    }
}
