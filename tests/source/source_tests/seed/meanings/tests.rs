//! Seed-lexicon invariant tests (issue #386).
//!
//! Extracted from `meanings.rs` so the loader stays under the seed file-size
//! guard; `super` resolves to the [`super`](super) meanings module and
//! `super::super` to [`crate::seed`], so the role imports below are unchanged.

use super::super::roles::{
    ROLE_DEFINITION_COMMAND, ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
    ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION, ROLE_INTERROGATIVE_OPENER,
    ROLE_LINKS_NOTATION_FORMAT, ROLE_MECHANISM_INQUIRY, ROLE_PROCEDURAL_REQUEST,
    ROLE_PROGRAM_ARTIFACT, ROLE_PROGRAM_MODIFICATION, ROLE_TRANSLATION_ACTION,
};
use super::*;

/// The supported languages every meaning must lexicalise so that the
/// concept truly is "translatable to any language" (issue #386).
const SUPPORTED_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

#[test]
fn lexicon_is_non_empty_and_well_formed() {
    let lex = lexicon();
    assert!(lex.meanings.len() >= 10, "expected a real lexicon");
    for meaning in &lex.meanings {
        assert!(!meaning.slug.is_empty(), "meaning needs a slug");
        assert!(
            !meaning.gloss.trim().is_empty(),
            "{} needs a conceptual gloss",
            meaning.slug
        );
        assert!(
            !meaning.wiktionary.trim().is_empty(),
            "{} must be grounded in real lexical data (wiktionary)",
            meaning.slug
        );
        assert!(
            !meaning.roles.is_empty(),
            "{} must declare at least one semantic role",
            meaning.slug
        );
    }
}

#[test]
fn every_meaning_is_self_describing() {
    // relative-meta-logic: each term is defined using other terms. The
    // `defined_by` graph must therefore be closed — every reference
    // resolves to another defined meaning (cycles are allowed and
    // expected; there are no undefined primitives).
    let lex = lexicon();
    let slugs: BTreeSet<&str> = lex.meanings.iter().map(|m| m.slug.as_str()).collect();
    for meaning in &lex.meanings {
        assert!(
            !meaning.defined_by.is_empty(),
            "{} must be defined by other meanings",
            meaning.slug
        );
        for target in &meaning.defined_by {
            assert!(
                slugs.contains(target.as_str()),
                "{} is defined_by `{target}`, which is not itself a defined meaning",
                meaning.slug
            );
        }
    }
}

#[test]
fn semantic_facet_blocks_are_parsed_as_meaning_references() {
    let lex = parse_lexicon(
        r#"
meanings
  meaning "alpha"
    gloss "alpha meaning"
    wiktionary "alpha"
    defined_by "alpha"
    role "ontology_root"
    facet "notation"
      meaning "beta"
    lexeme "en"
      word "alpha"
        description "English alpha."
    lexeme "ru"
      word "альфа"
        description "Russian alpha."
    lexeme "hi"
      word "अल्फा"
        description "Hindi alpha."
    lexeme "zh"
      word "阿尔法"
        description "Chinese alpha."
  meaning "beta"
    gloss "beta meaning"
    wiktionary "beta"
    defined_by "alpha"
    role "ontology_category"
    lexeme "en"
      word "beta"
        description "English beta."
    lexeme "ru"
      word "бета"
        description "Russian beta."
    lexeme "hi"
      word "बीटा"
        description "Hindi beta."
    lexeme "zh"
      word "贝塔"
        description "Chinese beta."
"#,
    );

    let alpha = lex.meaning("alpha").expect("alpha fixture meaning");
    let notation_targets: Vec<&str> = alpha.semantic_facet_targets("notation").collect();
    assert_eq!(
        notation_targets,
        vec!["beta"],
        "facet blocks must parse as meaning references"
    );
    let resolved = lex.semantic_facet_meanings("alpha", "notation");
    assert_eq!(
        resolved.iter().map(|m| m.slug.as_str()).collect::<Vec<_>>(),
        vec!["beta"],
        "facet targets must resolve through the lexicon"
    );
}

#[test]
fn word_form_facet_blocks_are_parsed_as_meaning_references() {
    let lex = parse_lexicon(
        r#"
meanings
  meaning "alpha"
    gloss "alpha meaning"
    wiktionary "alpha"
    defined_by "alpha"
    role "ontology_root"
    lexeme "en"
      word "alpha"
        description "English alpha."
        facet "denotation"
          meaning "alpha"
        facet "part_of_speech"
          meaning "noun"
  meaning "denotation"
    gloss "denotation facet"
    wiktionary "denotation"
    defined_by "alpha"
    role "semantic_facet_kind"
  meaning "part_of_speech"
    gloss "part of speech facet"
    wiktionary "part of speech"
    defined_by "alpha"
    role "semantic_facet_kind"
  meaning "noun"
    gloss "noun part of speech"
    wiktionary "noun"
    defined_by "part_of_speech"
    role "lexical_meta"
"#,
    );

    let alpha = lex.meaning("alpha").expect("alpha fixture meaning");
    let alpha_form = alpha
        .word_forms()
        .find(|form| form.text == "alpha")
        .expect("alpha word form");
    assert_eq!(
        alpha_form
            .semantic_facet_targets("denotation")
            .collect::<Vec<_>>(),
        vec!["alpha"],
        "word-form denotation facet must parse as a meaning reference"
    );
    assert_eq!(
        alpha_form
            .semantic_facet_targets("part_of_speech")
            .collect::<Vec<_>>(),
        vec!["noun"],
        "word-form part_of_speech facet must parse as a meaning reference"
    );
    for target in ["denotation", "part_of_speech", "noun"] {
        assert!(
            lex.meaning(target).is_some(),
            "word-form facet target {target} should resolve in the fixture"
        );
    }
}

#[test]
fn root_link_declares_the_required_semantic_facets() {
    // Issue #398: notation, annotation, denotation, and connotation are
    // meaning links in the lexicon, not English-only prose fields. The root
    // link meaning carries all four so future domain meanings can backfill the
    // same schema without changing parser code.
    let lex = lexicon();
    let root = lex
        .meaning("link")
        .expect("the ontology root meaning must exist");
    let facet_kinds = ["notation", "annotation", "denotation", "connotation"];
    for kind in facet_kinds {
        let kind_meaning = lex
            .meaning(kind)
            .unwrap_or_else(|| panic!("{kind} must itself be a meaning"));
        assert!(
            kind_meaning.has_role("semantic_facet_kind"),
            "{kind} must be identifiable as a semantic facet kind"
        );
        assert!(
            lex.reaches_root(kind),
            "{kind} must reduce to the link ontology root"
        );
        let targets: Vec<&str> = root.semantic_facet_targets(kind).collect();
        assert!(
            !targets.is_empty(),
            "the root link meaning must declare a {kind} facet"
        );
        for target in targets {
            assert!(
                lex.meaning(target).is_some(),
                "link facet {kind} references undefined meaning {target}"
            );
        }
    }
}

#[test]
fn every_semantic_facet_resolves_to_seed_meanings() {
    let lex = lexicon();
    for meaning in &lex.meanings {
        for facet in &meaning.semantic_facets {
            assert!(
                lex.meaning(&facet.kind).is_some(),
                "{} declares undefined semantic facet kind {}",
                meaning.slug,
                facet.kind
            );
            assert!(
                !facet.meanings.is_empty(),
                "{} declares empty semantic facet {}",
                meaning.slug,
                facet.kind
            );
            for target in &facet.meanings {
                assert!(
                    lex.meaning(target).is_some(),
                    "{} semantic facet {} references undefined meaning {}",
                    meaning.slug,
                    facet.kind,
                    target
                );
            }
        }
        for lexeme in &meaning.lexemes {
            for form in &lexeme.words {
                for facet in &form.semantic_facets {
                    assert!(
                        lex.meaning(&facet.kind).is_some(),
                        "{} / {} / {} declares undefined word-form facet kind {}",
                        meaning.slug,
                        lexeme.language,
                        form.text,
                        facet.kind
                    );
                    assert!(
                        !facet.meanings.is_empty(),
                        "{} / {} / {} declares empty word-form facet {}",
                        meaning.slug,
                        lexeme.language,
                        form.text,
                        facet.kind
                    );
                    for target in &facet.meanings {
                        assert!(
                            lex.meaning(target).is_some(),
                            "{} / {} / {} word-form facet {} references undefined meaning {}",
                            meaning.slug,
                            lexeme.language,
                            form.text,
                            facet.kind,
                            target
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn every_word_form_exposes_recursive_notation_and_denotation() {
    // The lexeme/word nesting is itself seed data: every surface belongs to a
    // parent meaning. The parser must expose that structural assertion as
    // meaning-linked facets so consumers are not forced to interpret English
    // `description` text.
    let lex = lexicon();
    for meaning in &lex.meanings {
        for form in meaning.word_forms() {
            assert!(
                form.semantic_facet_targets("notation")
                    .any(|target| target == "word_surface"),
                "{} / {} must expose word_surface as its notation meaning",
                meaning.slug,
                form.text
            );
            assert!(
                form.semantic_facet_targets("denotation")
                    .any(|target| target == meaning.slug),
                "{} / {} must denote its parent meaning",
                meaning.slug,
                form.text
            );
        }
    }
}

#[test]
fn semantic_meta_word_forms_have_recursive_facet_links() {
    // Issue #398 review feedback called out English-only word descriptions.
    // The semantic-meta seed now ratchets the richer shape: each surface form
    // in that cluster carries meaning-linked notation, denotation, and
    // part-of-speech facets. The legacy English `description` remains only as
    // a human annotation while consumers migrate to these links.
    let lex = lexicon();
    let semantic_meta_meanings = [
        "semantic_facet",
        "notation",
        "annotation",
        "denotation",
        "connotation",
        "semantic_gloss",
        "external_knowledge_source",
        "cached_source_response",
    ];
    for slug in semantic_meta_meanings {
        let meaning = lex
            .meaning(slug)
            .unwrap_or_else(|| panic!("{slug} semantic-meta meaning must exist"));
        for form in meaning.word_forms() {
            assert!(
                form.semantic_facet_targets("notation")
                    .any(|target| target == "word_surface"),
                "{slug} / {} must link its notation to word_surface",
                form.text
            );
            assert!(
                form.semantic_facet_targets("denotation")
                    .any(|target| target == slug),
                "{slug} / {} must denote its parent meaning",
                form.text
            );
            assert!(
                form.semantic_facet_targets("part_of_speech")
                    .any(|target| target == "noun" || target == "noun_phrase"),
                "{slug} / {} must declare a meaning-backed part of speech",
                form.text
            );
        }
    }
}

#[test]
fn every_meaning_covers_all_supported_languages() {
    let lex = lexicon();
    for meaning in &lex.meanings {
        let languages = meaning.languages();
        for language in SUPPORTED_LANGUAGES {
            assert!(
                languages.contains(language),
                "{} is missing a `{language}` lexeme — meanings must translate to every supported language",
                meaning.slug
            );
        }
        for lexeme in &meaning.lexemes {
            assert!(
                !lexeme.words.is_empty(),
                "{} / {} lexeme must list at least one surface word",
                meaning.slug,
                lexeme.language
            );
        }
    }
}

#[test]
fn word_forms_round_trip_and_describe_word_resolves() {
    // The surface text iterated by `words()` and the text carried by each
    // `WordForm` must be the same set, and `describe_word` must resolve a
    // recorded form (case-insensitively) while rejecting an unknown one.
    let lex = lexicon();
    let meaning = &lex.meanings[0];
    let from_words: Vec<&str> = meaning.words().collect();
    let from_forms: Vec<&str> = meaning.word_forms().map(|f| f.text.as_str()).collect();
    assert_eq!(
        from_words, from_forms,
        "words() and word_forms() must enumerate the same surfaces in order"
    );
    let first = from_words[0];
    assert!(
        meaning.describe_word(first).is_some(),
        "describe_word must resolve a recorded surface form"
    );
    assert!(
        meaning
            .describe_word("\u{0}-definitely-not-a-recorded-surface")
            .is_none(),
        "describe_word must return None for an unknown surface"
    );
}

#[test]
fn descriptions_are_parsed_from_the_seed() {
    // Proves the `description` child is actually read off the wire:
    // at least one surface form must carry a non-empty self-describing note.
    // (The whole-lexicon enforcement invariant lives in
    // `every_word_form_is_described`.)
    let lex = lexicon();
    let described = lex
        .meanings
        .iter()
        .flat_map(Meaning::word_forms)
        .filter(|f| !f.description.trim().is_empty())
        .count();
    assert!(
        described > 0,
        "the parser must read `description` children off the seed"
    );
}

#[test]
fn every_word_form_is_described() {
    // The self-describing-data contract (issue #386): a meaning may not just
    // *list* a surface form, it must *describe* it. Every `word` in every
    // lexeme of every meaning carries a non-empty `description`, so the seed
    // can be consumed for any purpose — not just the parsing order the code
    // happens to use today. This is the permanent ratchet behind the
    // per-language backfill; a new word with no description fails the build.
    let lex = lexicon();
    let mut missing: Vec<String> = Vec::new();
    for meaning in &lex.meanings {
        for lexeme in &meaning.lexemes {
            for form in &lexeme.words {
                if form.description.trim().is_empty() {
                    missing.push(format!(
                        "{} / {} / {}",
                        meaning.slug, lexeme.language, form.text
                    ));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} word form(s) lack a description, e.g. {}",
        missing.len(),
        missing
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ")
    );
}

#[test]
fn program_roles_are_populated() {
    let lex = lexicon();
    assert!(
        !lex.words_for_role(ROLE_PROGRAM_ARTIFACT).is_empty(),
        "program_artifact role must have surface words"
    );
    assert!(
        !lex.words_for_role(ROLE_PROGRAM_MODIFICATION).is_empty(),
        "program_modification role must have surface words"
    );
}

#[test]
fn word_form_slot_is_derived_from_the_ellipsis_marker() {
    // The slot classification is purely a function of where the `…` (U+2026)
    // marker sits in the surface text (issue #386): no marker is Bare, a
    // trailing marker is Prefix, a leading marker is Suffix, and a middle
    // marker is Circumfix. before_slot/after_slot expose the literals around
    // the slot so a handler can match them without naming the surface.
    let form = |text: &str| WordForm {
        text: text.to_string(),
        description: String::new(),
        action: String::new(),
        semantic_facets: Vec::new(),
    };

    let bare = form("how it works");
    assert_eq!(bare.slot(), Slot::Bare);
    assert_eq!(bare.before_slot(), "how it works");
    assert_eq!(bare.after_slot(), "");

    let prefix = form("how does …");
    assert_eq!(prefix.slot(), Slot::Prefix);
    assert_eq!(prefix.before_slot(), "how does ");
    assert_eq!(prefix.after_slot(), "");

    let suffix = form("… как работает");
    assert_eq!(suffix.slot(), Slot::Suffix);
    assert_eq!(suffix.before_slot(), "");
    assert_eq!(suffix.after_slot(), " как работает");

    let circumfix = form("how … works");
    assert_eq!(circumfix.slot(), Slot::Circumfix);
    assert_eq!(circumfix.before_slot(), "how ");
    assert_eq!(circumfix.after_slot(), " works");
}

#[test]
fn how_cluster_roles_populate_and_classify() {
    let lex = lexicon();

    // mechanism_inquiry surfaces span all four slot kinds; representative
    // surfaces must land in the expected bucket with the expected literals.
    let mech = lex.role_word_forms(ROLE_MECHANISM_INQUIRY);
    assert!(
        !mech.is_empty(),
        "mechanism_inquiry must contribute surface forms"
    );
    assert!(mech.iter().any(|f| f.slot() == Slot::Bare));
    assert!(mech.iter().any(|f| f.slot() == Slot::Prefix));
    assert!(mech.iter().any(|f| f.slot() == Slot::Suffix));
    assert!(mech.iter().any(|f| f.slot() == Slot::Circumfix));
    assert!(
        mech.iter()
            .any(|f| f.slot() == Slot::Bare && f.text == "how it works"),
        "the bare English how-it-works phrase must be present"
    );
    assert!(
        mech.iter()
            .any(|f| f.slot() == Slot::Prefix && f.before_slot() == "how does "),
        "the `how does …` prefix surface must be present"
    );
    assert!(
        mech.iter().any(|f| f.slot() == Slot::Circumfix
            && f.before_slot() == "how "
            && f.after_slot() == " works"),
        "the `how … works` circumfix surface must be present"
    );
    assert!(
        mech.iter()
            .any(|f| f.slot() == Slot::Suffix && f.after_slot() == " как работает"),
        "the `… как работает` suffix surface must be present"
    );

    // procedural_request surfaces are all prefixes; some name a canonical
    // action, others leave the operation to the matched task's first word.
    let proc = lex.role_word_forms(ROLE_PROCEDURAL_REQUEST);
    assert!(
        !proc.is_empty(),
        "procedural_request must contribute surface forms"
    );
    assert!(
        proc.iter().all(|f| f.slot() == Slot::Prefix),
        "every procedural surface positions the task after the slot"
    );
    assert!(
        proc.iter()
            .any(|f| f.before_slot() == "how to do " && f.action == "do"),
        "`how to do …` must name the do action explicitly"
    );
    assert!(
        proc.iter()
            .any(|f| f.before_slot() == "how to " && f.action.is_empty()),
        "`how to …` must leave the action to the task"
    );
    assert!(
        proc.iter()
            .any(|f| f.before_slot() == "如何做 " && f.action == "do"),
        "the Chinese `如何做 …` surface must carry its trailing space and do action"
    );
}

#[test]
fn mentions_role_honours_cjk_and_token_boundaries() {
    let lex = lexicon();
    // Whitespace token (Russian): a substring of a longer token must NOT
    // match, but the standalone token must.
    assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отмени сортировку"));
    assert!(!lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
    // CJK substring: matches inside a space-free run.
    assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "取消排序"));
    assert!(lex.mentions_role(ROLE_PROGRAM_ARTIFACT, "取消排序"));
}

#[test]
fn mentions_role_raw_matches_inflected_stems() {
    // The raw sibling is the looser match a legacy stem recogniser needs: the
    // bare imperative `отмени` is a substring of the longer word
    // `отменительный`, so the raw query fires where the token-bounded one
    // deliberately does not. This is the byte-faithful behaviour the old
    // `normalized.contains("…")` disjunctions relied on.
    let lex = lexicon();
    assert!(lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
    assert!(!lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
    // A whole-token surface still matches under the raw query.
    assert!(lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "отмени сортировку"));
    // A prompt with no modification word matches under neither query.
    assert!(!lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "привет мир"));
}

#[test]
fn words_for_role_partition_by_language() {
    // The translation-action stems split by linguistic typology: the
    // clause-initial English/Russian command verbs versus the head-final
    // Hindi/Chinese ones. Each partition draws only its own languages' words.
    let lex = lexicon();
    let head_initial = lex.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["en", "ru"]);
    assert!(head_initial.iter().any(|w| w == "translate"));
    assert!(head_initial.iter().any(|w| w == "переведи"));
    assert!(head_initial.iter().any(|w| w == "опиши"));
    assert!(!head_initial.iter().any(|w| w == "翻译"));
    let head_final = lex.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["hi", "zh"]);
    assert!(head_final.iter().any(|w| w == "翻译"));
    assert!(head_final.iter().any(|w| w == "अनुवाद"));
    assert!(!head_final.iter().any(|w| w == "translate"));
}

#[test]
fn implementation_language_marker_roles_expose_head_initial_surfaces() {
    // Issue #386: the unknown-implementation-language extractor
    // (`requested_program_language` in intent_formalization.rs and the JS
    // worker's `programLanguageFromPrompt`) reads the language name that
    // *follows* the target marker, so it consults only the head-initial
    // English/Russian surfaces. This locks that seed->code contract: dropping
    // "in"/"на" or "language"/"языке" from the seed would silently break the
    // extractor, so assert the surfaces the code depends on are present and
    // that the head-final Hindi/Chinese forms (carried for coverage) are not
    // mixed into the head-initial partition the extractor reads.
    let lex = lexicon();
    let prepositions =
        lex.words_for_role_in_languages(ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION, &["en", "ru"]);
    assert!(
        prepositions.iter().any(|w| w == "in"),
        "English target preposition surface missing, got: {prepositions:?}"
    );
    assert!(
        prepositions.iter().any(|w| w == "на"),
        "Russian target preposition surface missing, got: {prepositions:?}"
    );
    let nouns = lex.words_for_role_in_languages(ROLE_IMPLEMENTATION_LANGUAGE_NOUN, &["en", "ru"]);
    assert!(
        nouns.iter().any(|w| w == "language"),
        "English language-noun surface missing, got: {nouns:?}"
    );
    assert!(
        nouns.iter().any(|w| w == "языке"),
        "Russian language-noun surface missing, got: {nouns:?}"
    );
    // Both new marker meanings reduce to the single `link` ontology root like
    // every other meaning, so the data stays self-describing end to end.
    for role in [
        ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION,
        ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
    ] {
        let mut count = 0;
        for meaning in lex.meanings_with_role(role) {
            count += 1;
            assert!(
                lex.reaches_root(&meaning.slug),
                "meaning {} (role {role}) must reduce to the link root",
                meaning.slug
            );
        }
        assert_eq!(count, 1, "exactly one meaning should carry role {role}");
    }
}

#[test]
fn define_in_links_roles_expose_the_scanned_surfaces() {
    // Issue #386: the `try_translation` request-gate recognises a
    // define-in-Links-Notation request from meaning, not literals. It scans
    // only the English `definition_command` verb (a clause-initial prefix) and
    // the English/Russian `links_notation_format` markers (space-prefixed
    // substrings). This locks that seed->code contract: dropping any of those
    // surfaces would silently break the gate. The Hindi/Chinese forms are
    // carried for coverage and intentionally excluded from the scanned slice.
    let lex = lexicon();
    // The English `definition_command` slice must be *exactly* {"define"} and the
    // English/Russian `links_notation_format` slice *exactly* {"links notation",
    // "в links"} — the three literals the original gate scanned. Asserting the
    // whole set (not just membership) locks behaviour preservation: silently
    // adding a synonym would broaden the gate, and dropping one would break it.
    let mut verbs = lex.words_for_role_in_languages(ROLE_DEFINITION_COMMAND, &["en"]);
    verbs.sort();
    assert_eq!(
        verbs,
        vec!["define".to_owned()],
        "English define-command surface set drifted from the original gate"
    );
    let mut markers = lex.words_for_role_in_languages(ROLE_LINKS_NOTATION_FORMAT, &["en", "ru"]);
    markers.sort();
    assert_eq!(
        markers,
        vec!["links notation".to_owned(), "в links".to_owned()],
        "English/Russian links-notation marker set drifted from the original gate"
    );
    // Both new meanings reduce to the single `link` ontology root like every
    // other meaning, so the data stays self-describing end to end.
    for role in [ROLE_DEFINITION_COMMAND, ROLE_LINKS_NOTATION_FORMAT] {
        let mut count = 0;
        for meaning in lex.meanings_with_role(role) {
            count += 1;
            assert!(
                lex.reaches_root(&meaning.slug),
                "meaning {} (role {role}) must reduce to the link root",
                meaning.slug
            );
        }
        assert_eq!(count, 1, "exactly one meaning should carry role {role}");
    }
}

#[test]
fn interrogative_opener_role_exposes_head_initial_question_words() {
    // Issue #386: the intent classifier's `starts_with_question_word`
    // (intent_formalization.rs) tells a question from a statement by matching a
    // fronted wh-word, reading the surfaces from this role instead of a
    // hardcoded prefix list. It consults only the head-initial English/Russian
    // partition (a prefix match: the bare word plus a trailing space). Asserting
    // the *whole* English and Russian sets — not just membership — locks
    // behaviour preservation against the original list: silently adding an opener
    // would broaden question detection, and dropping one would miss questions.
    let lex = lexicon();
    let mut english = lex.words_for_role_in_languages(ROLE_INTERROGATIVE_OPENER, &["en"]);
    english.sort();
    assert_eq!(
        english,
        vec![
            "how".to_owned(),
            "what".to_owned(),
            "when".to_owned(),
            "where".to_owned(),
            "which".to_owned(),
            "who".to_owned(),
            "why".to_owned(),
        ],
        "English interrogative-opener set drifted from the original classifier list"
    );
    let mut russian = lex.words_for_role_in_languages(ROLE_INTERROGATIVE_OPENER, &["ru"]);
    russian.sort();
    assert_eq!(
        russian,
        vec![
            "где".to_owned(),
            "как".to_owned(),
            "когда".to_owned(),
            "кто".to_owned(),
            "почему".to_owned(),
            "что".to_owned(),
        ],
        "Russian interrogative-opener set drifted from the original classifier list"
    );
    // The head-final Hindi/Chinese forms are carried for coverage but must not
    // leak into the head-initial partition the front scan reads.
    let head_initial = lex.words_for_role_in_languages(ROLE_INTERROGATIVE_OPENER, &["en", "ru"]);
    for cjk in ["क्या", "什么", "कौन", "谁"] {
        assert!(
            !head_initial.iter().any(|w| w == cjk),
            "head-final surface {cjk} leaked into the head-initial partition"
        );
    }
    // The single meaning reduces to the `link` ontology root like every other
    // meaning, so the data stays self-describing end to end.
    let mut count = 0;
    for meaning in lex.meanings_with_role(ROLE_INTERROGATIVE_OPENER) {
        count += 1;
        assert!(
            lex.reaches_root(&meaning.slug),
            "meaning {} (role interrogative_opener) must reduce to the link root",
            meaning.slug
        );
    }
    assert_eq!(
        count, 1,
        "exactly one meaning should carry role interrogative_opener"
    );
}

#[test]
fn first_role_language_reads_the_command_language() {
    // The source-inferencer asks which language's translation verb a prompt
    // carries; that language is the language the user wrote the command in.
    let lex = lexicon();
    let priority = ["ru", "hi", "zh"];
    assert_eq!(
        lex.first_role_language(ROLE_TRANSLATION_ACTION, "переведи apple", &priority),
        Some("ru")
    );
    assert_eq!(
        lex.first_role_language(ROLE_TRANSLATION_ACTION, "apple का अनुवाद करो", &priority),
        Some("hi")
    );
    assert_eq!(
        lex.first_role_language(ROLE_TRANSLATION_ACTION, "把 apple 翻译成中文", &priority),
        Some("zh")
    );
    // No command verb present → no language inferred (caller defaults to en).
    assert_eq!(
        lex.first_role_language(ROLE_TRANSLATION_ACTION, "what is apple", &priority),
        None
    );
}

#[test]
fn the_ontology_has_a_single_link_root() {
    // The merged ontology has exactly one root — the `link` meaning, which
    // is defined_by itself. A type-system sub-root (`type`) sits under it,
    // realising "Link should be the root of ontology, we can also have Type
    // link, for type system ontology" (issue #386).
    let lex = lexicon();
    let roots: Vec<&Meaning> = lex.meanings_with_role(ROLE_ONTOLOGY_ROOT).collect();
    assert_eq!(
        roots.len(),
        1,
        "the merged ontology must have exactly one root, found {}",
        roots.len()
    );
    let root = roots[0];
    assert_eq!(root.slug, "link", "the ontology root must be `link`");
    assert!(
        root.defined_by.iter().any(|t| t == "link"),
        "the root `link` must be defined by itself (self-rooted)"
    );
    let type_root = lex
        .ontology_type_root()
        .expect("a type-system sub-root (role ontology_type) must exist");
    assert!(
        lex.reaches_root(&type_root.slug),
        "the type sub-root must reduce to the link root"
    );
    // The bridge categories (entity, concept, relation, action, property)
    // each sit under the root too, so every domain genus has a category to
    // root in.
    let categories: Vec<&Meaning> = lex.ontology_categories().collect();
    assert!(
        categories.len() >= 2,
        "the ontology must define top-level categories under the root, found {}",
        categories.len()
    );
    for category in categories {
        assert!(
            lex.reaches_root(&category.slug),
            "ontology category {} must reduce to the link root",
            category.slug
        );
    }
}

#[test]
fn every_meaning_reaches_the_link_root() {
    // The whole lexicon is one connected ontology: following `defined_by`
    // from any meaning eventually arrives at the single `link` root. No
    // meaning is an island of vocabulary disconnected from the root concept.
    let lex = lexicon();
    assert!(lex.ontology_root().is_some(), "an ontology root must exist");
    for meaning in &lex.meanings {
        assert!(
            lex.reaches_root(&meaning.slug),
            "{} does not reach the `link` ontology root via defined_by",
            meaning.slug
        );
    }
}
