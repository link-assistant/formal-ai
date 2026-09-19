use std::collections::BTreeMap;

use formal_ai::coding_function_catalog::python_docs::{StdlibIndex, StdlibPart};
use formal_ai::coding_function_catalog::wikifunctions::{
    FunctionMatch, FunctionPart, Implementation,
};
use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Parameter};
use formal_ai::concept_discovery::{
    CatalogFunction, ConceptEvidence, ConceptMap, DiscoveryBounds, DiscoveryCatalog,
    discover_with_lookup,
};
use formal_ai::concept_lookup::{ConceptSense, LookupOutcome};
use formal_ai::needs::Need;
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::source_walk::{LookupBounds, SourceLookup};

#[derive(Default)]
struct EmptyLookup;

impl SourceLookup for EmptyLookup {
    fn lookup(&mut self, _need: &Need, _bounds: &LookupBounds) -> LookupOutcome {
        LookupOutcome::NotFound {
            consulted: Vec::new(),
        }
    }
}

fn discover(spec: &CodingTaskSpec, catalog: &DiscoveryCatalog) -> ConceptMap {
    let mut lookup = EmptyLookup;
    formal_ai::concept_discovery::discover(spec, catalog, &mut lookup)
}

fn stdlib(symbol: &str, description: &str) -> StdlibPart {
    StdlibPart {
        symbol: symbol.to_owned(),
        module: symbol
            .split_once('.')
            .map_or("builtins", |(module, _)| module)
            .to_owned(),
        signature: String::new(),
        description: description.to_owned(),
        source_url: format!("https://docs.python.org/3.12/library/functions.html#{symbol}"),
        license: "PSF-2.0".to_owned(),
        sha256: "a".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
    }
}

fn match_record(zid: &str, label: &str) -> FunctionMatch {
    FunctionMatch {
        page_title: zid.to_owned(),
        label: label.to_owned(),
        match_label: label.to_owned(),
        match_lang: "Z1002".to_owned(),
        match_rate: 1.0,
        source_url: "https://www.wikifunctions.org/w/api.php".to_owned(),
        sha256: "b".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
    }
}

fn discovery_catalog() -> DiscoveryCatalog {
    let mut labels = BTreeMap::new();
    labels.insert("en".to_owned(), "greatest common divisor".to_owned());
    let definition = FunctionPart {
        zid: "Z13612".to_owned(),
        labels,
        argument_keys: vec!["Z13612K1".to_owned(), "Z13612K2".to_owned()],
        argument_types: vec!["Z13518".to_owned(), "Z13518".to_owned()],
        return_type: "Z13518".to_owned(),
        implementation_zids: vec!["Z14857".to_owned(), "Z13642".to_owned()],
        tester_zids: Vec::new(),
        license: "CC0-1.0".to_owned(),
        source_url: "https://www.wikifunctions.org/wiki/Z13612".to_owned(),
        sha256: "c".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
    };
    let implementation = |zid: &str, code: &str| Implementation {
        zid: zid.to_owned(),
        function_zid: "Z13612".to_owned(),
        label: None,
        language: "python".to_owned(),
        code: code.to_owned(),
        license: "Apache-2.0".to_owned(),
        source_url: format!("https://www.wikifunctions.org/wiki/{zid}"),
        sha256: "d".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
    };
    DiscoveryCatalog::new(
        StdlibIndex::from_parts(vec![
            stdlib("sum", "Return the sum and total of an iterable."),
            stdlib("math.prod", "Calculate the product of all elements."),
            stdlib(
                "math.gcd",
                "Return the greatest common divisor of integer arguments.",
            ),
            stdlib("any", "Return true if any element is true."),
            stdlib(
                "itertools.combinations",
                "Return combinations of input elements.",
            ),
            stdlib("abs", "Return the absolute value of a number."),
        ]),
        vec![match_record("Z13558", "product of list (natural numbers)")],
        vec![CatalogFunction {
            definition,
            implementations: vec![
                implementation("Z14857", "import math\nreturn math.gcd(a, b)"),
                implementation("Z13642", "while b:\n    a, b = b, a % b\nreturn a"),
            ],
            recurrence: None,
            source_tests: Vec::new(),
        }],
    )
}

fn spec(name: &str, sentence: &str, prose_language: &str) -> CodingTaskSpec {
    CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Function,
        name: name.to_owned(),
        parameters: vec![Parameter {
            name: "items".to_owned(),
            annotation: None,
        }],
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: vec![sentence.to_owned()],
        examples: Vec::new(),
        expected_stdout: None,
        prose_language: prose_language.to_owned(),
    }
}

#[test]
fn requirements_become_structures_catalog_parts_and_need_ledger_rows() {
    let catalog = discovery_catalog();
    let gcd = discover(
        &spec(
            "greatest_common_divisor",
            "Return a greatest common divisor of two integers a and b.",
            "en",
        ),
        &catalog,
    );
    assert_eq!(gcd.candidate_ids(), ["math.gcd", "Z13642", "Z14857"]);

    let sum_product = discover(
        &spec(
            "sum_product",
            "Return a tuple consisting of a sum and a product of all the integers in a list.",
            "en",
        ),
        &catalog,
    );
    for structure in ["tuple_of", "reduce_sum", "reduce_product"] {
        assert!(
            sum_product.structure_ids().contains(&structure.to_owned()),
            "{sum_product:?}"
        );
    }
    for candidate in ["sum", "math.prod", "Z13558"] {
        assert!(
            sum_product.candidate_ids().contains(&candidate.to_owned()),
            "{sum_product:?}"
        );
    }

    let close = discover(
        &spec(
            "has_close_elements",
            "Determine whether any two numbers are closer to each other than the given threshold.",
            "en",
        ),
        &catalog,
    );
    for structure in [
        "quantifier_any",
        "pairwise_distinct",
        "predicate_abs_diff_lt",
    ] {
        assert!(
            close.structure_ids().contains(&structure.to_owned()),
            "{close:?}"
        );
    }
    let links = close.to_links_notation();
    assert!(links.contains("need_ledger"), "{links}");
    assert!(links.contains("status \"satisfied\""), "{links}");
}

#[test]
fn five_languages_reduce_to_the_same_concept_identity() {
    let catalog = discovery_catalog();
    let cases = [
        (
            "en",
            "Return a tuple consisting of a sum and a product of all integers.",
        ),
        (
            "ru",
            "Верни кортеж состоящий из суммы и произведение всех чисел.",
        ),
        ("hi", "सभी पूर्णांकों के योग और गुणनफल से बना टपल लौटाएँ।"),
        ("zh", "返回由所有整数的总和与乘积组成的元组。"),
        (
            "es",
            "Devuelve una tupla compuesta por la suma y el producto de todos los enteros.",
        ),
    ];
    let identities = cases.map(|(language, sentence)| {
        discover(&spec("sum_product", sentence, language), &catalog).identity()
    });
    assert!(
        identities.windows(2).all(|pair| pair[0] == pair[1]),
        "{identities:?}"
    );
}

#[derive(Default)]
struct CountingLookup {
    calls: usize,
}

impl SourceLookup for CountingLookup {
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome {
        self.calls += 1;
        LookupOutcome::Found(vec![ConceptSense {
            surface: need.subject.clone(),
            lemma: need.subject.clone(),
            language: need.language.clone(),
            gloss: "a fixture definition".to_owned(),
            part_of_speech: "noun".to_owned(),
            synonyms: Vec::new(),
            source_id: "wiktionary".to_owned(),
            source_url: "https://en.wiktionary.org/wiki/florpquux".to_owned(),
            sha256: "a".repeat(64),
            fetched_at: "2026-09-16T00:00:00Z".to_owned(),
            cached: true,
            tier: SourceTier::IndependentCorroboration,
            license_name: "CC-BY-SA-4.0".to_owned(),
            license_url: "https://creativecommons.org/licenses/by-sa/4.0/".to_owned(),
            depth: bounds.max_depth.min(1),
        }])
    }
}

#[test]
fn an_unknown_phrase_triggers_one_bounded_lookup_and_is_recorded() {
    let catalog = DiscoveryCatalog::new(StdlibIndex::default(), Vec::new(), Vec::new());
    let mut lookup = CountingLookup::default();
    let map = discover_with_lookup(
        &spec("transform_value", "Return the florpquux.", "en"),
        &catalog,
        &mut lookup,
        DiscoveryBounds {
            max_depth: 2,
            max_pages: 8,
        },
    );
    assert_eq!(lookup.calls, 1);
    assert_eq!(map.evidence.len(), 1);
    assert!(map.to_links_notation().contains("florpquux"));
}

// Issue #1138, plan 01 L8–L9 and plan 04 L3: the coding path must ask about the
// words it does not know even when the rest of the sentence was understood, a
// retrieved sense must become a candidate the composer can read, and the
// coding path and the formalizer must share one need record.

#[test]
fn a_partially_understood_sentence_still_asks_about_its_unresolved_words() {
    let catalog = discovery_catalog();
    let sentence = "Return true when the word is an isogram.";
    let understood = discover(&spec("is_isogram", sentence, "en"), &catalog);

    let surfaces = formal_ai::concept_discovery::unresolved_surfaces(
        sentence,
        &understood
            .needs
            .iter()
            .flat_map(|need| need.structures.clone())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        surfaces,
        vec![String::from("isogram")],
        "a sentence whose verb is understood still has an unresolved noun"
    );

    let mut lookup = CountingLookup::default();
    let asked = discover_with_lookup(
        &spec("is_isogram", sentence, "en"),
        &catalog,
        &mut lookup,
        DiscoveryBounds {
            max_depth: 2,
            max_pages: 8,
        },
    );
    assert_eq!(
        lookup.calls, 1,
        "the unresolved surface is asked about once, not the whole sentence"
    );
    assert_eq!(asked.evidence.len(), 1);
}

#[test]
fn retrieved_evidence_becomes_a_candidate_part_the_composer_can_read() {
    let evidence = ConceptEvidence {
        phrase: "isogram".to_owned(),
        definition: "a word in which no letter is repeated".to_owned(),
        source_url: "https://en.wiktionary.org/wiki/isogram".to_owned(),
        depth: 0,
    };
    let candidate = formal_ai::concept_discovery::concept_candidate(&evidence);

    assert_eq!(candidate.kind, "concept_sense");
    assert_eq!(candidate.label, "a word in which no letter is repeated");
    assert_eq!(
        candidate.source_url,
        "https://en.wiktionary.org/wiki/isogram"
    );
    assert!(
        candidate.code.is_none(),
        "a definition is evidence, never a program"
    );
    assert!(
        !candidate.license.is_empty() && candidate.sha256.len() == 64,
        "a candidate part carries the provenance of the bytes it came from"
    );
}

// The fixture repair this case needed, recorded at the case.
//
// As written in wave T the case asked `discovery_catalog()` — a catalog built
// through `DiscoveryCatalog::new`, which leaves `source_candidates` empty — for
// a `source_program` candidate. `source_program` parts exist only in
// `DiscoveryCatalog::source_candidates` (`src/coding/synthesis_runtime.rs`
// fills them from the sequence-source walk), so the case panicked
// `both a program and a sense must be offered` with `(None, Some(3))` before
// comparing anything: the sense *was* ranked last, and the check could not see
// it. The repair is in the case's own setup — the catalog now offers the
// program the assertion is about, and the requirement sentence carries one word
// nobody has seeded, so the lookup offers the sense the assertion is about —
// and the assertion itself is untouched (issue #1138, plan 01 L9).
//
// The sentence needed the second half of the repair for the same class of
// reason: after plan 01 L8, a sense reaches a need only through
// `unresolved_surfaces`, and every word of
// "Return the greatest common divisor of the two integers." is accounted for by
// the structures that sentence matches, so the need offered no sense at all.
fn retrieved_program(id: &str, composition: &str) -> formal_ai::concept_discovery::CandidatePart {
    formal_ai::concept_discovery::CandidatePart {
        id: id.to_owned(),
        kind: "source_program".to_owned(),
        label: composition.to_owned(),
        language: Some("python".to_owned()),
        code: Some("while b:\n    a, b = b, a % b\nreturn a".to_owned()),
        callable_name: Some("gcd".to_owned()),
        source_tests: Vec::new(),
        license: "GFDL-1.2-or-later".to_owned(),
        source_url: "https://rosettacode.org/wiki/Greatest_common_divisor".to_owned(),
        sha256: "e".repeat(64),
        fetched_at: "2026-09-15T00:00:00Z".to_owned(),
        score: 1.0,
    }
}

#[test]
fn concept_senses_rank_below_retrieved_implementations() {
    let catalog = discovery_catalog().with_source_candidates(vec![retrieved_program(
        "rosettacode:greatest_common_divisor",
        "greatest common divisor",
    )]);
    let mut lookup = CountingLookup::default();
    let map = discover_with_lookup(
        &spec(
            "greatest_common_divisor",
            "Return the greatest common divisor of the two integers, the largest florpquux they share.",
            "en",
        ),
        &catalog,
        &mut lookup,
        DiscoveryBounds {
            max_depth: 2,
            max_pages: 8,
        },
    );

    let kinds: Vec<String> = map
        .needs
        .iter()
        .flat_map(|need| need.candidates.iter().map(|part| part.kind.clone()))
        .collect();
    let sense = kinds.iter().position(|kind| kind == "concept_sense");
    let program = kinds.iter().position(|kind| kind == "source_program");
    match (program, sense) {
        (Some(program), Some(sense)) => assert!(
            program < sense,
            "a retrieved definition never outranks a retrieved implementation: {kinds:?}"
        ),
        other => panic!("both a program and a sense must be offered: {other:?} in {kinds:?}"),
    }
}

#[test]
fn the_coding_path_and_the_formalizer_share_one_need_type_and_one_status_enum() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = std::fs::read_to_string(root.join("src/coding/concept_discovery.rs"))
        .expect("the coding discovery module");
    assert!(
        discovery.contains("pub use crate::needs::Need as ConceptNeed"),
        "`ConceptNeed` is a re-export of the one need record, not a second struct"
    );
    assert!(
        !discovery.contains("pub status: String"),
        "a need's state is the contract enum, never a free-text string"
    );

    let meta_frame =
        std::fs::read_to_string(root.join("src/meta_frame.rs")).expect("the meta frame");
    assert!(
        meta_frame.contains("NeedState"),
        "`meta_frame::NeedStatus` maps onto the one need-state vocabulary"
    );
    for producerless in ["Deferred", "Rejected"] {
        assert!(
            !meta_frame.contains(&format!("    {producerless},")),
            "`{producerless}` has no producer and must be removed with the merge"
        );
    }
}

#[test]
fn coding_discovery_uses_only_the_shared_source_lookup_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = std::fs::read_to_string(root.join("src/coding/concept_discovery.rs"))
        .expect("the coding discovery module");
    let lookup = std::fs::read_to_string(root.join("src/concept_lookup.rs"))
        .expect("the concept lookup module");

    assert!(
        discovery.contains("lookup: &mut dyn SourceLookup"),
        "coding discovery must accept the shared lookup contract directly"
    );
    for retired in ["UnknownConceptLookup", "NoLookup", "RegistryConceptLookup"] {
        assert!(
            !discovery.contains(retired) && !lookup.contains(retired),
            "the retired adapter `{retired}` must not return"
        );
    }
}

#[test]
fn an_aggregate_domain_cue_is_not_a_predicate_quantifier() {
    // "The sum and product of every integer" states what the reduction
    // ranges over, not a filter the program must enforce; the same
    // partitive reading carries the ru/hi/zh/es paraphrases. Raising the
    // universal structure here forces every candidate to materialize an
    // `all(...)` gate over an aggregation, and the plain tuple shape is
    // priced out of the typed frontier.
    let structures = formal_ai::concept_discovery::structures_for(
        "define python function summarize_numbers numbers return a tuple of the sum and product of every integer",
    );
    let ids = structures
        .iter()
        .map(|structure| structure.id.as_str())
        .collect::<Vec<_>>();
    assert!(
        !ids.contains(&"quantifier_all"),
        "an aggregate noun phrase must not raise the universal quantifier structure: {ids:?}"
    );
    assert!(
        ids.contains(&"reduce_sum") && ids.contains(&"reduce_product") && ids.contains(&"tuple_of"),
        "the reduction and tuple structures still carry the request: {ids:?}"
    );
}
