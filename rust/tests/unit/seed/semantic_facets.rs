use formal_ai::seed::lexicon;

struct SurfaceCase {
    meaning: &'static str,
    language: &'static str,
    surface: &'static str,
}

#[test]
fn semantic_facet_seed_surfaces_cover_supported_languages() {
    let lex = lexicon();
    let cases = [
        SurfaceCase {
            meaning: "semantic_facet",
            language: "en",
            surface: "semantic facet",
        },
        SurfaceCase {
            meaning: "semantic_facet",
            language: "ru",
            surface: "семантический аспект",
        },
        SurfaceCase {
            meaning: "semantic_facet",
            language: "hi",
            surface: "अर्थ पक्ष",
        },
        SurfaceCase {
            meaning: "semantic_facet",
            language: "zh",
            surface: "语义方面",
        },
    ];

    for case in cases {
        let meaning = lex
            .meaning(case.meaning)
            .unwrap_or_else(|| panic!("{} meaning must exist", case.meaning));
        let surface = meaning.word_in(case.language);
        assert_eq!(
            surface,
            Some(case.surface),
            "{} must include `{}` for language {}, got {:?}",
            case.meaning,
            case.surface,
            case.language,
            surface
        );
    }
}

#[test]
fn root_link_semantic_facets_resolve_through_public_seed_api() {
    let lex = lexicon();
    let expected = [
        ("notation", "links_notation_format"),
        ("annotation", "semantic_gloss"),
        ("denotation", "relation"),
        ("connotation", "concept"),
    ];

    for (kind, target) in expected {
        let resolved = lex.semantic_facet_meanings("link", kind);
        assert!(
            resolved.iter().any(|meaning| meaning.slug == target),
            "link {kind} facet must resolve to {target}, got {:?}",
            resolved
                .iter()
                .map(|meaning| meaning.slug.as_str())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn semantic_meta_word_forms_expose_meaning_linked_facets() {
    let lex = lexicon();
    let meaning = lex
        .meaning("semantic_facet")
        .expect("semantic_facet meaning must exist");
    let form = meaning
        .word_forms()
        .find(|form| form.text == "semantic facet")
        .expect("semantic facet English surface must exist");

    assert!(
        form.semantic_facet_targets("notation")
            .any(|target| target == "word_surface"),
        "word form should link notation to the word_surface meaning"
    );
    assert!(
        form.semantic_facet_targets("denotation")
            .any(|target| target == "semantic_facet"),
        "word form should link denotation back to the semantic_facet meaning"
    );
    assert!(
        form.semantic_facet_targets("part_of_speech")
            .any(|target| target == "noun_phrase"),
        "word form should expose a meaning-backed part of speech"
    );
}

#[test]
fn ordinary_word_forms_expose_structural_notation_and_denotation() {
    let lex = lexicon();
    let meaning = lex.meaning("link").expect("link meaning must exist");
    let form = meaning
        .word_forms()
        .find(|form| form.text == "link")
        .expect("link English surface must exist");

    assert!(
        form.semantic_facet_targets("notation")
            .any(|target| target == "word_surface"),
        "word form should expose its literal surface as a word_surface notation"
    );
    assert!(
        form.semantic_facet_targets("denotation")
            .any(|target| target == "link"),
        "word form should expose its parent meaning as the denotation"
    );
}
