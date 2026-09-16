use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use formal_ai::coding_function_catalog::python_docs::{StdlibIndex, StdlibPart};
use formal_ai::coding_task_spec::recognise;
use formal_ai::composition::compose;
use formal_ai::concept_discovery::{DiscoveryCatalog, discover};
use formal_ai::needs::NeedState;

const CORPUS: &str = "data/benchmarks/coding-discovery-paraphrases.lino";

#[derive(Debug)]
struct Paraphrase {
    family: String,
    language: String,
    prompt: String,
}

#[test]
fn five_language_paraphrases_share_specs_concepts_and_verified_compositions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = paraphrases(&fs::read_to_string(root.join(CORPUS)).expect("paraphrase corpus"));
    assert_eq!(cases.len(), 25);
    let catalog = catalog();
    let mut identities: BTreeMap<String, (String, String)> = BTreeMap::new();

    for case in cases {
        let prompt = format!("{}\n{}", case.prompt, assertions(&case.family));
        let spec = recognise(&prompt).unwrap_or_else(|| panic!("{} did not parse", case.language));
        let concepts = discover(&spec, &catalog);
        let outcome = compose(&spec, &concepts);
        let selected = outcome.selected.unwrap_or_else(|| {
            panic!(
                "{} {} did not verify: {}",
                case.family, case.language, outcome.research_trail
            )
        });
        if case.family == "count_to_100" {
            assert!(selected.source.contains("range(1, 100 + 1)"));
            assert_eq!(selected.assertion_count, 1);
        }
        let identity = (spec.signature_identity(), concepts.identity());
        if let Some(english) = identities.get(&case.family) {
            assert_eq!(&identity, english, "{} {}", case.family, case.language);
        } else {
            assert_eq!(case.language, "en", "English must lead each family");
            identities.insert(case.family, identity);
        }
    }
    assert_eq!(identities.len(), 5);
}

#[test]
fn count_to_one_hundred_requests_derive_the_full_inclusive_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = paraphrases(&fs::read_to_string(root.join(CORPUS)).expect("paraphrase corpus"));
    for case in cases
        .into_iter()
        .filter(|case| case.family == "count_to_100")
    {
        let spec = recognise(&case.prompt).unwrap_or_else(|| panic!("{}", case.language));
        assert_eq!(spec.name, "main");
        let concepts = discover(&spec, &DiscoveryCatalog::default());
        let selected = compose(&spec, &concepts)
            .selected
            .unwrap_or_else(|| panic!("{} did not derive count-to-N", case.language));
        assert!(selected.source.contains("range(1, 100 + 1)"));
    }
}

#[test]
fn held_out_sentences_are_not_seed_lexemes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = paraphrases(&fs::read_to_string(root.join(CORPUS)).expect("paraphrase corpus"));
    let seed = fs::read_dir(root.join("data/seed"))
        .expect("seed directory")
        .filter_map(Result::ok)
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .map(|text| normalize(&text))
        .collect::<Vec<_>>()
        .join("\n");
    for case in cases {
        assert!(
            !seed.contains(&normalize(&case.prompt)),
            "held-out sentence entered seed: {}",
            case.prompt
        );
    }
}

// Issue #1138, plan 01 L10 and plan 02 L20: the five-language invariant must
// hold for families whose key word is in no seed file, which is what makes the
// identity a property of retrieval rather than of memorisation.

/// The held-out corpora this bottleneck adds, none of whose sentences may enter
/// the seed.
const ISSUE_1138_CORPORA: [&str; 3] = [
    "data/benchmarks/concept-lookup-paraphrases.lino",
    "data/benchmarks/formalization-depth-requirements.lino",
    "data/benchmarks/coding-composition-from-sources.lino",
];

fn corpus(relative: &str) -> Vec<Paraphrase> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    paraphrases(&fs::read_to_string(root.join(relative)).expect("held-out corpus"))
}

/// The languages a declared source actually publishes a sense of the held-out
/// family's word in, **measured** through the production path on 2026-09-16 and
/// replayed here from the committed captures under
/// `tests/fixtures/issue-1138-b1/`.
///
/// | language | senses | which source, or why none |
/// | --- | --- | --- |
/// | en | 2 | `wordnet` (the cartographic sense) and `wikipedia` |
/// | ru | **0** | no article (HTTP 404); the language edition's own Wiktionary has an entry that publishes no definition |
/// | hi | **0** | no article (HTTP 404 and zero search hits), no Wiktionary entry, no Wikidata lexeme and no Wikidata item in `hi` |
/// | zh | 2 | the language edition's own Wiktionary, through `language_api` |
/// | es | 1 | `wikipedia` |
///
/// The endpoints those rows were measured against are declared in
/// `data/seed/sources-registry.lino` — `wikipedia`'s five-language REST
/// summary, `wiktionary`'s `language_api`, and `wikidata`'s `lexeme_api` — so
/// the attempt is rediscoverable and this table can be re-measured by
/// `FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_1138_concept_lookup_capture`.
const SERVED_LANGUAGES: [&str; 3] = ["en", "zh", "es"];

#[test]
fn held_out_unknown_word_tasks_share_one_concept_map_identity_in_five_languages() {
    let catalog = catalog();
    let mut identities: BTreeMap<String, String> = BTreeMap::new();
    let cases = corpus(ISSUE_1138_CORPORA[0])
        .into_iter()
        .filter(|case| case.family == "isogram")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 5, "five languages for the held-out family");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-1138-b1");

    for case in cases {
        let spec = recognise(&case.prompt)
            .unwrap_or_else(|| panic!("{} {} did not parse", case.family, case.language));
        let client = formal_ai::source_fetch::CachedSourceClient::new(
            &fixtures,
            formal_ai::source_fetch::CurlSourceTransport,
        )
        .with_online(false);
        let preferences = formal_ai::how_to_guide::ServicePreferences::default();
        let mut availability = formal_ai::service_accessibility::ServiceAccessibilityCache::new(
            std::env::temp_dir().join(format!("formal-ai-1138-l10-{}", case.language)),
        );
        let mut lookup = formal_ai::concept_lookup::RegistryConceptLookup::new(
            formal_ai::concept_lookup::RegistrySourceLookup::new(
                &client,
                &preferences,
                &mut availability,
                formal_ai::source_walk::LookupBounds::default(),
                &case.language,
                u64::MAX / 2,
            ),
        );
        let concepts = formal_ai::concept_discovery::discover_with_lookup(
            &spec,
            &catalog,
            &mut lookup,
            formal_ai::concept_discovery::DiscoveryBounds::default(),
        );

        if SERVED_LANGUAGES.contains(&case.language.as_str()) {
            // The identity must be shared *because the word was resolved*, not
            // because every language understood equally little: an empty map
            // has a stable identity too, and that would prove nothing.
            assert!(
                !concepts.evidence.is_empty(),
                "{}: the held-out word must be resolved through retrieval, not skipped",
                case.language
            );
            assert_eq!(
                concepts.needs[0].status(),
                "satisfied",
                "{}: the requirement sentence is grounded once the word is retrieved: {:?}",
                case.language,
                concepts.needs
            );
        } else {
            // The amendment this leaf carried, and its reason. As written in
            // wave T the case demanded a retrieved sense in all five languages.
            // It was then measured: for the held-out family, `hi` and `ru` are
            // served by *no* declared endpoint, and the attempts are listed in
            // `SERVED_LANGUAGES` above. A five-language identity through
            // retrieval is not achievable against these sources, and inventing
            // a gloss to reach it is the one thing this whole bottleneck
            // exists to stop. So the unserved languages are held to the honest
            // alternative instead: the map must *say* that nobody served the
            // word, in a need of its own, rather than fall silent.
            assert!(
                concepts.evidence.is_empty(),
                "{}: no declared source serves this language; a sense here would be fabricated",
                case.language
            );
            let suffix = format!("@{}", case.language);
            let unserved: Vec<String> = concepts
                .needs
                .iter()
                .filter(|need| need.status() == "unsatisfiable")
                .map(|need| need.phrase().to_owned())
                .collect();
            assert!(
                !unserved.is_empty(),
                "{}: an unserved language is recorded as a need, never as silence: {:?}",
                case.language,
                concepts.needs
            );
            assert!(
                unserved.iter().any(|phrase| phrase.ends_with(&suffix)),
                "{}: one need names the surface and the language nobody served it in: {unserved:?}",
                case.language
            );
        }
        assert!(
            concepts
                .needs
                .iter()
                .all(|need| need.need.state != NeedState::Open),
            "{}: an untried need means the word was never grounded: {:?}",
            case.language,
            concepts.needs
        );

        let identity = concepts.identity();
        if let Some(english) = identities.get(&case.family) {
            assert_eq!(
                &identity, english,
                "{} {} must share the family identity",
                case.family, case.language
            );
        } else {
            assert_eq!(case.language, "en", "English must lead each family");
            identities.insert(case.family.clone(), identity);
        }
    }
}

#[test]
fn composition_from_sources_holds_in_five_languages() {
    let cases = corpus(ISSUE_1138_CORPORA[2]);
    assert_eq!(cases.len(), 25, "five cases, five languages");
    let catalog = catalog();
    let mut identities: BTreeMap<String, String> = BTreeMap::new();

    for case in cases {
        let spec = recognise(&case.prompt)
            .unwrap_or_else(|| panic!("{} {} did not parse", case.family, case.language));
        let concepts = discover(&spec, &catalog);
        let identity = format!("{}|{}", spec.signature_identity(), concepts.identity());
        if let Some(english) = identities.get(&case.family) {
            assert_eq!(
                &identity, english,
                "{} {} must reduce to the family's identity",
                case.family, case.language
            );
        } else {
            assert_eq!(case.language, "en", "English must lead each family");
            identities.insert(case.family.clone(), identity);
        }

        let outcome = compose(&spec, &concepts);
        if case.family == "undefined_operation" {
            assert!(
                outcome.selected.is_none(),
                "{} names an operation no source defines and must stay an honest gap",
                case.language
            );
            assert!(
                !outcome.research_trail.is_empty(),
                "{} must report the trail it followed before refusing",
                case.language
            );
        } else {
            let selected = outcome.selected.unwrap_or_else(|| {
                panic!(
                    "{} {} did not compose: {}",
                    case.family, case.language, outcome.research_trail
                )
            });
            assert!(
                !selected.source.trim().is_empty(),
                "{} {} composed an empty program",
                case.family,
                case.language
            );
        }
    }
    assert_eq!(identities.len(), 5, "five held-out families");
}

#[test]
fn issue_1138_held_out_sentences_are_not_seed_lexemes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let seed = fs::read_dir(root.join("data/seed"))
        .expect("seed directory")
        .filter_map(Result::ok)
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .map(|text| normalize(&text))
        .collect::<Vec<_>>()
        .join("\n");
    for relative in ISSUE_1138_CORPORA {
        for case in corpus(relative) {
            assert!(
                !seed.contains(&normalize(&case.prompt)),
                "held-out sentence entered seed: {}",
                case.prompt
            );
        }
    }
}

fn assertions(family: &str) -> &'static str {
    match family {
        "gcd" => {
            "assert greatest_common_divisor(25, 15) == 5\nassert greatest_common_divisor(3, 5) == 1"
        }
        "sum_product" => {
            "assert summarize_numbers([]) == (0, 1)\nassert summarize_numbers([1, 2, 3]) == (6, 6)"
        }
        "near_pair" => {
            "assert near_pair([1.0, 2.0, 3.0], 0.5) == False\nassert near_pair([1.0, 2.8, 3.0], 0.3) == True"
        }
        "count_distinct" => "assert unique_count([1, 1, 2, 3]) == 3\nassert unique_count([]) == 0",
        "count_to_100" => "assert count_until() == list(range(1, 101))",
        other => panic!("unknown family {other}"),
    }
}

fn catalog() -> DiscoveryCatalog {
    let parts = vec![
        part(
            "math.gcd",
            "math",
            "Return the greatest common divisor of the specified integer arguments.",
            "https://docs.python.org/3.12/library/math.html#math.gcd",
        ),
        part(
            "sum",
            "builtins",
            "Return the sum of an iterable of numbers.",
            "https://docs.python.org/3.12/library/functions.html#sum",
        ),
        part(
            "math.prod",
            "math",
            "Calculate the product of all the elements in the input iterable.",
            "https://docs.python.org/3.12/library/math.html#math.prod",
        ),
    ];
    DiscoveryCatalog::new(StdlibIndex::from_parts(parts), Vec::new(), Vec::new())
}

fn part(symbol: &str, module: &str, description: &str, source_url: &str) -> StdlibPart {
    StdlibPart {
        symbol: symbol.to_owned(),
        module: module.to_owned(),
        signature: format!("{symbol}(...)"),
        description: description.to_owned(),
        source_url: source_url.to_owned(),
        license: "PSF-2.0".to_owned(),
        sha256: "c".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
    }
}

fn paraphrases(text: &str) -> Vec<Paraphrase> {
    let mut out = Vec::new();
    let mut current: Option<Paraphrase> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("  paraphrase ") {
            if let Some(record) = current.take() {
                out.push(record);
            }
            current = Some(Paraphrase {
                family: String::new(),
                language: String::new(),
                prompt: String::new(),
            });
        } else if let Some(record) = &mut current {
            if let Some(value) = trimmed.strip_prefix("family ") {
                record.family = unquote(value);
            } else if let Some(value) = trimmed.strip_prefix("language ") {
                record.language = unquote(value);
            } else if let Some(value) = trimmed.strip_prefix("prompt ") {
                record.prompt = unquote(value);
            }
        }
    }
    if let Some(record) = current {
        out.push(record);
    }
    out
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').replace("\"\"", "\"")
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
