//! Tests for the bulk lexeme importer (issue #660, R378).
//!
//! These cover the four acceptance dimensions called out in the issue:
//! template emission, facet completeness, denotation bidirectionality, and
//! cache-record grounding — plus the headline guarantee that
//! `formal-ai import lexemes --concepts <file> --offline` reproduces the
//! committed batch byte-for-byte from the committed cache.

use std::fs;
use std::path::PathBuf;

use formal_ai::event_log::EventLog;
use formal_ai::lexeme_import::{
    self, Concept, GroundedLexeme, ImportConfig, DEFINED_BY, GRAMMATICAL_NUMBER, IMPORT_LANGUAGES,
    PART_OF_SPEECH,
};
use formal_ai::seed::parse_lexicon_text;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cache_dir() -> PathBuf {
    repo_root().join("data/cache/wikidata/entity")
}

fn concepts_file() -> PathBuf {
    repo_root().join("data/lexicon-import/common-nouns.lino")
}

fn seed_dir() -> PathBuf {
    repo_root().join("data/seed")
}

/// The committed shard files produced by the importer, sorted by name.
fn committed_shards() -> Vec<PathBuf> {
    let mut shards: Vec<PathBuf> = fs::read_dir(seed_dir())
        .expect("seed directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("meanings-lexicon-import")
                        && std::path::Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("lino"))
                })
        })
        .collect();
    shards.sort();
    shards
}

fn committed_concepts() -> Vec<Concept> {
    let text = fs::read_to_string(concepts_file()).expect("concepts file");
    lexeme_import::parse_concepts(&text)
}

/// Template emission: a rendered block is exactly the 23-line grounded shape,
/// so the seed format stays stable and reviewable.
#[test]
fn template_emission_matches_contract() {
    let lexeme = GroundedLexeme {
        slug: "dog".to_string(),
        qid: "Q144".to_string(),
        labels: IMPORT_LANGUAGES
            .iter()
            .zip(["dog", "собака", "कुत्ता", "犬"])
            .map(|(language, surface)| (language.to_string(), surface.to_string()))
            .collect(),
    };
    let expected = "  dog
    grounded-in Q144
    defined-by entity
    lexeme en
      surface
        text dog
        part_of_speech noun
        grammatical_number singular
    lexeme ru
      surface
        text собака
        part_of_speech noun
        grammatical_number singular
    lexeme hi
      surface
        text कुत्ता
        part_of_speech noun
        grammatical_number singular
    lexeme zh
      surface
        text 犬
        part_of_speech noun
        grammatical_number singular
";
    assert_eq!(lexeme_import::render_block(&lexeme), expected);
}

/// Facet completeness: every surface in every committed shard carries both a
/// `part_of_speech` and a `grammatical_number` facet.
#[test]
fn every_surface_carries_required_facets() {
    let shards = committed_shards();
    assert!(!shards.is_empty(), "expected at least one committed shard");
    let mut checked = 0usize;
    for shard in shards {
        let text = fs::read_to_string(&shard).expect("shard readable");
        let lexicon = parse_lexicon_text(&text);
        for meaning in &lexicon.meanings {
            assert!(
                meaning.defined_by.iter().any(|target| target == DEFINED_BY),
                "{} is not defined-by {DEFINED_BY}",
                meaning.slug
            );
            for lexeme in &meaning.lexemes {
                for form in &lexeme.words {
                    assert_eq!(
                        form.part_of_speech(),
                        Some(PART_OF_SPEECH),
                        "{} {} surface `{}` lacks part_of_speech",
                        meaning.slug,
                        lexeme.language,
                        form.text
                    );
                    assert_eq!(
                        form.grammatical_number(),
                        Some(GRAMMATICAL_NUMBER),
                        "{} {} surface `{}` lacks grammatical_number",
                        meaning.slug,
                        lexeme.language,
                        form.text
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(
        checked >= 100,
        "expected >= 100 grounded surfaces, saw {checked}"
    );
}

/// Denotation bidirectionality: every imported surface denotes its own meaning,
/// closing the surface → meaning link the seed relies on.
#[test]
fn denotation_is_bidirectional() {
    for shard in committed_shards() {
        let text = fs::read_to_string(&shard).expect("shard readable");
        let lexicon = parse_lexicon_text(&text);
        for meaning in &lexicon.meanings {
            for lexeme in &meaning.lexemes {
                for form in &lexeme.words {
                    assert!(
                        form.denotations().any(|target| target == meaning.slug),
                        "{} {} surface `{}` does not denote its meaning",
                        meaning.slug,
                        lexeme.language,
                        form.text
                    );
                }
            }
        }
    }
}

/// Cache-record grounding: every imported concept resolves to a committed
/// entity cache record whose four labels are exactly the emitted surfaces, and
/// the record has its canonical `.lino` sibling.
#[test]
fn cache_records_ground_every_import() {
    let concepts = committed_concepts();
    assert!(
        concepts.len() >= 100,
        "expected >= 100 imported concepts, saw {}",
        concepts.len()
    );

    // Map slug -> emitted labels from the committed shards.
    let mut emitted = std::collections::BTreeMap::new();
    for shard in committed_shards() {
        let text = fs::read_to_string(&shard).expect("shard readable");
        let lexicon = parse_lexicon_text(&text);
        for meaning in &lexicon.meanings {
            let mut labels = std::collections::BTreeMap::new();
            for lexeme in &meaning.lexemes {
                if let Some(form) = lexeme.words.first() {
                    labels.insert(lexeme.language.clone(), form.text.clone());
                }
            }
            emitted.insert(meaning.slug.clone(), (meaning.wikidata.clone(), labels));
        }
    }

    for concept in &concepts {
        let json_path = lexeme_import::entity_json_path(&cache_dir(), &concept.qid);
        assert!(
            json_path.is_file(),
            "missing cache json for {}",
            concept.qid
        );
        let lino_path = lexeme_import::entity_lino_path(&cache_dir(), &concept.qid);
        assert!(
            lino_path.is_file(),
            "missing cache lino for {}",
            concept.qid
        );

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&json_path).expect("json readable"))
                .expect("cache json parses");
        let labels = lexeme_import::labels_from_entity(&value, &concept.qid)
            .unwrap_or_else(|error| panic!("labels for {}: {error}", concept.qid));

        let (qid, emitted_labels) = emitted
            .get(&concept.slug)
            .unwrap_or_else(|| panic!("{} not present in committed shards", concept.slug));
        assert_eq!(qid, &concept.qid, "{} grounding mismatch", concept.slug);
        for language in IMPORT_LANGUAGES {
            assert_eq!(
                labels.get(language),
                emitted_labels.get(language),
                "{} {language} surface differs from its cache record",
                concept.slug
            );
        }
    }
}

/// Byte-for-byte offline reproduction: rerunning the importer over the
/// committed concepts, reading only the committed cache, regenerates every
/// committed shard exactly. This is the headline acceptance criterion.
#[test]
fn offline_import_reproduces_committed_batch_byte_for_byte() {
    let config = ImportConfig {
        concepts: committed_concepts(),
        cache_dir: cache_dir(),
        online: false,
    };
    let mut events = EventLog::new();
    let report = lexeme_import::run(&config, None, &mut events);

    let committed = committed_shards();
    assert_eq!(
        report.shards.len(),
        committed.len(),
        "shard count differs: generated {}, committed {}",
        report.shards.len(),
        committed.len()
    );
    for (generated, path) in report.shards.iter().zip(&committed) {
        assert_eq!(
            generated.file_name,
            path.file_name().and_then(|name| name.to_str()).unwrap(),
            "shard file name differs"
        );
        let on_disk = fs::read_to_string(path).expect("committed shard readable");
        assert_eq!(
            generated.content, on_disk,
            "shard {} is not reproduced byte-for-byte",
            generated.file_name
        );
    }
    assert!(
        report.rejected.is_empty(),
        "committed batch should not be rejected: {:?}",
        report.rejected
    );
}

/// The importer refuses to write entries that fail validation and records an
/// `import_rejected` event instead of emitting them.
#[test]
fn invalid_concepts_are_rejected_not_written() {
    let config = ImportConfig {
        concepts: vec![Concept {
            slug: "missing".to_string(),
            qid: "Q999999999".to_string(),
        }],
        cache_dir: cache_dir(),
        online: false,
    };
    let mut events = EventLog::new();
    let report = lexeme_import::run(&config, None, &mut events);
    assert!(report.accepted.is_empty());
    assert!(report.shards.is_empty());
    assert_eq!(report.rejected.len(), 1);
    assert!(
        events
            .events()
            .iter()
            .any(|event| event.kind == "import_rejected"),
        "a rejection must be recorded as an import_rejected event"
    );
}

/// A canonical sample lexeme, mirroring the `dog`/`Q144` template.
fn sample() -> GroundedLexeme {
    GroundedLexeme {
        slug: "dog".to_string(),
        qid: "Q144".to_string(),
        labels: IMPORT_LANGUAGES
            .iter()
            .zip(["dog", "собака", "कुत्ता", "犬"])
            .map(|(language, surface)| (language.to_string(), surface.to_string()))
            .collect(),
    }
}

/// Concept parsing keeps only well-formed `<slug> <Qid>` pairs, ignoring
/// comments and malformed lines.
#[test]
fn parses_concepts_ignoring_non_pairs() {
    let concepts =
        lexeme_import::parse_concepts("concepts\n  dog Q144\n  # note\n  bad\n  flag Q14660\n");
    assert_eq!(
        concepts,
        vec![
            Concept {
                slug: "dog".into(),
                qid: "Q144".into()
            },
            Concept {
                slug: "flag".into(),
                qid: "Q14660".into()
            },
        ]
    );
}

/// A well-formed sample lexeme validates: its rendered block parses back and
/// every surface denotes its meaning with the required facets.
#[test]
fn rendered_block_validates() {
    assert_eq!(lexeme_import::validate(&sample()), Ok(()));
}

/// Every supported import language keeps its own citation-form surface pinned in
/// the canonical `dog`/`Q144` template, so a single-language regression cannot
/// slip in unnoticed: English `dog`, Russian `собака`, Hindi `कुत्ता`, and
/// Chinese `犬` are each asserted explicitly rather than only in aggregate.
#[test]
fn every_supported_language_surface_is_pinned() {
    let labels = sample().labels;
    let mut expected = IMPORT_LANGUAGES
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(
        labels.keys().cloned().collect::<Vec<_>>(),
        expected,
        "sample must carry exactly one surface per supported language"
    );
    // English (en)
    assert_eq!(labels.get("en").map(String::as_str), Some("dog"));
    // Russian (ru)
    assert_eq!(labels.get("ru").map(String::as_str), Some("собака"));
    // Hindi (hi)
    assert_eq!(labels.get("hi").map(String::as_str), Some("कुत्ता"));
    // Chinese (zh)
    assert_eq!(labels.get("zh").map(String::as_str), Some("犬"));
}

/// A multi-token label (not a single citation-form surface) is rejected.
#[test]
fn multi_token_label_is_rejected() {
    let mut lexeme = sample();
    lexeme
        .labels
        .insert("ru".to_string(), "две собаки".to_string());
    assert!(lexeme_import::validate(&lexeme).is_err());
}
