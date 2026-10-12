//! Wikidata-grounded membership taxonomy for verifiable counting (#1138).
//!
//! Plan 08 deleted the memorized `OBJECT_CATEGORIES` table on purpose: category
//! membership must come from trusted sources, never from a runtime table
//! shaped like the benchmark. These tests pin the seed-side replacement —
//! meanings grounded in Wikidata items whose `defined-by` edges mirror the
//! items' P279 (subclass of) claims, each meaning owned by exactly one shard.

use formal_ai::seed::{Meaning, lexicon};

fn seeded(slug: &str) -> &'static Meaning {
    (lexicon().meaning(slug).unwrap_or_else(|| {
        panic!("seed lexicon should carry meaning `{slug}` for the membership taxonomy")
    })) as _
}

/// Walk the public `defined_by` graph the way the solver's `seeded_is_a` does,
/// so the seed alone — not a runtime table — decides category membership.
fn seed_reaches(entity: &str, category: &str) -> bool {
    fn walk(current: &str, category: &str, visited: &mut Vec<String>) -> bool {
        if current == category {
            return true;
        }
        if visited.contains(&current.to_owned()) {
            return false;
        }
        visited.push(current.to_owned());
        let Some(meaning) = lexicon().meaning(current) else {
            return false;
        };
        meaning
            .defined_by
            .iter()
            .any(|parent| walk(parent, category, visited))
    }
    walk(entity, category, &mut Vec::new())
}

#[test]
fn violin_keeps_its_lexemes_and_gains_its_subclass_edge() {
    let meaning = seeded("violin");
    assert_eq!(meaning.wikidata, "Q8355");
    assert!(
        meaning
            .defined_by
            .contains(&String::from("bowed-string-instrument"))
    );
    for language in ["en", "ru", "hi", "zh"] {
        assert!(
            meaning
                .lexemes
                .iter()
                .any(|lexeme| lexeme.language == language),
            "violin must keep the four lexemes the import batch established ({language} lost)"
        );
    }
}

#[test]
fn membership_chains_reach_musical_instrument_from_wikidata_groundings() {
    let instruments = [
        ("clarinet", "Q8343"),
        ("violin", "Q8355"),
        ("flute", "Q11405"),
    ];
    for (slug, qid) in instruments {
        let meaning = seeded(slug);
        assert_eq!(
            meaning.wikidata, qid,
            "`{slug}` must stay grounded in its Wikidata item"
        );
        assert!(
            seed_reaches(slug, "musical-instrument"),
            "`{slug}` must reach musical-instrument over seed defined-by edges"
        );
    }
    let category = seeded("musical-instrument");
    assert_eq!(category.wikidata, "Q34379");
    assert!(category.lexemes.iter().any(|lexeme| {
        lexeme
            .words
            .iter()
            .any(|word| word.text == "musical instrument")
    }));
}

#[test]
fn non_instruments_do_not_reach_the_category() {
    seeded("spoon");
    assert!(
        !seed_reaches("spoon", "musical-instrument"),
        "spoon (cutlery, Q81895) must not reach musical-instrument"
    );
}

#[test]
fn membership_edges_carry_source_attestations() {
    // Every hop of every chain is a distinct Wikidata item with its own
    // grounding, so the count answer can cite the category and each member.
    let chain_hops = [
        ("clarinet", "Q8343"),
        ("transposing-instrument", "Q217306"),
        ("flute", "Q11405"),
        ("wind-instrument", "Q173453"),
        ("aerophone", "Q659216"),
        ("bowed-string-instrument", "Q192096"),
        ("string-instrument", "Q1798603"),
        ("chordophone", "Q1051772"),
        ("musical-instrument", "Q34379"),
    ];
    for (slug, qid) in chain_hops {
        let meaning = seeded(slug);
        assert_eq!(
            meaning.wikidata, qid,
            "`{slug}` must be grounded in {qid} so membership evidence can cite it"
        );
    }
}
