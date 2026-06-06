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
