use formal_ai::seed::{lexicon, Meaning};

fn assert_defined_by(meaning: &Meaning, expected: &[&str]) {
    for target in expected {
        assert!(
            meaning.defined_by.iter().any(|defined| defined == target),
            "{} must be defined_by {target}; got {:?}",
            meaning.slug,
            meaning.defined_by
        );
    }
}

#[test]
fn links_theory_root_terms_are_seed_meanings() {
    let lex = lexicon();
    let expected = [
        ("reference", &["reference_action", "link"][..]),
        (
            "reference_action",
            &["makes", "reference", "point_at", "link"],
        ),
        (
            "link_action",
            &["makes", "link", "from", "any_of_reference"],
        ),
        ("any_of_reference", &["reference", "repeatable_from_zero"]),
        ("any_of_link", &["link", "repeatable_from_zero"]),
        (
            "repeatable_from_zero",
            &["same", "link_action", "zero", "more"],
        ),
    ];

    for (slug, defined_by) in expected {
        let meaning = lex
            .meaning(slug)
            .unwrap_or_else(|| panic!("{slug} meaning must exist"));
        assert!(
            meaning.has_role("links_root"),
            "{slug} must be part of the Links Theory semantic root"
        );
        assert_defined_by(meaning, defined_by);
        assert!(
            lex.reaches_root(slug),
            "{slug} must reduce to the root link through meaning references"
        );
    }

    let link = lex.meaning("link").expect("link root meaning");
    assert_defined_by(link, &["link", "link_action", "any_of_reference"]);
}

#[test]
fn self_equations_are_explicit_semantic_facets() {
    let lex = lexicon();
    let facet_kind = lex
        .meaning("self_equation")
        .expect("self_equation facet kind must exist");
    assert!(
        facet_kind.has_role("semantic_facet_kind"),
        "self_equation must be a meaning-backed facet kind"
    );

    for slug in ["type", "not", "same"] {
        let meaning = lex
            .meaning(slug)
            .unwrap_or_else(|| panic!("{slug} self-equation meaning must exist"));
        assert!(
            meaning
                .semantic_facet_targets("self_equation")
                .any(|target| target == slug),
            "{slug} must declare its fixed-point self-equation as a meaning link"
        );
    }
}

#[test]
fn defined_connectives_and_is_senses_are_not_opaque_english() {
    let lex = lexicon();
    let expected = [
        ("of", &["belonging", "from", "part", "to", "whole"][..]),
        ("from", &["source", "end", "direction"]),
        ("to", &["target", "end", "direction"]),
        ("and", &["together_with"]),
        ("is_identity", &["same"]),
        ("is_a_kind_of", &["subtype", "supertype", "direction"]),
        ("held_by", &["from", "property", "to", "entity"]),
    ];

    for (slug, defined_by) in expected {
        let meaning = lex
            .meaning(slug)
            .unwrap_or_else(|| panic!("{slug} connective meaning must exist"));
        assert_defined_by(meaning, defined_by);
        assert!(
            lex.reaches_root(slug),
            "{slug} must remain in the single recursive meaning graph"
        );
    }
}

#[test]
fn ambiguous_bank_surface_is_split_into_distinct_symbols() {
    let lex = lexicon();
    assert!(
        lex.meaning("bank").is_none(),
        "the root seed must not keep an ambiguous bare `bank` meaning"
    );

    let river = lex
        .meaning("bank_river")
        .expect("bank_river split meaning must exist");
    let money = lex
        .meaning("bank_money")
        .expect("bank_money split meaning must exist");

    for meaning in [river, money] {
        assert_defined_by(meaning, &["sense_split"]);
        assert!(
            meaning.word_forms().all(|form| form.text != "bank"),
            "{} must expose an unambiguous surface instead of bare `bank`",
            meaning.slug
        );
    }
    assert_ne!(
        river.word_in("en"),
        money.word_in("en"),
        "the two English bank senses must render as separate symbols"
    );
}
