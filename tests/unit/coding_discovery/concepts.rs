use std::collections::BTreeMap;

use formal_ai::coding_function_catalog::python_docs::{StdlibIndex, StdlibPart};
use formal_ai::coding_function_catalog::wikifunctions::{
    FunctionMatch, FunctionPart, Implementation,
};
use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Parameter};
use formal_ai::concept_discovery::{
    CatalogFunction, ConceptEvidence, DiscoveryBounds, DiscoveryCatalog, UnknownConceptLookup,
    discover, discover_with_lookup,
};

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

impl UnknownConceptLookup for CountingLookup {
    fn lookup(&mut self, phrase: &str, depth: usize) -> Option<ConceptEvidence> {
        self.calls += 1;
        Some(ConceptEvidence {
            phrase: phrase.to_owned(),
            definition: "a fixture definition".to_owned(),
            source_url: "https://en.wiktionary.org/wiki/florpquux".to_owned(),
            depth,
        })
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
