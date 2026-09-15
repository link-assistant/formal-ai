use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::coding_function_catalog::wikifunctions::{
    fetch_abstract_implementation, fetch_function, fetch_function_descriptors,
    fetch_implementations, fetch_testers, search_functions,
};
use formal_ai::coding_recurrence::{formalize, python_assertions};
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let fixture = if url
            .contains("wikilambdasearch_functions_search=greatest%20common%20divisor")
        {
            "search-greatest-common-divisor-en.json"
        } else if url.contains("wikilambdasearch_functions_search=qzxjvkplmnbvcx") {
            "search-no-match-en.json"
        } else if url.contains("zids=Z13612") {
            "fetch-Z13612-en.json"
        } else if url.contains("zids=Z14707%7CZ14857%7CZ29084%7CZ13642%7CZ13639") {
            "fetch-gcd-implementations-en.json"
        } else if url.contains("zids=Z13614%7CZ13615%7CZ13616%7CZ13613") {
            "fetch-gcd-testers-en.json"
        } else if url.contains("wikilambdasearch_functions_search=nth%20Fibonacci%20number") {
            "search-nth-fibonacci-number-en.json"
        } else if url.contains("wikilambdasearch_functions_search=factorial") {
            "search-factorial-en.json"
        } else if url.contains("zids=Z13835") {
            "fetch-Z13835-en.json"
        } else if url.contains("zids=Z13864") {
            "fetch-Z13864-abstract-en.json"
        } else if url.contains("zids=Z13667") {
            "fetch-Z13667-en.json"
        } else if url.contains("zids=Z13863") {
            "fetch-Z13863-abstract-en.json"
        } else if url.contains("zids=Z802%7CZ13695%7CZ13521%7CZ13582%7CZ13569%7CZ13522%7CZ13539") {
            "fetch-recurrence-operators-en.json"
        } else if url.contains("zids=Z13869%7CZ17391%7CZ17398%7CZ34276") {
            "fetch-recurrence-testers-en.json"
        } else if url.contains("zids=Z13840%7CZ13865%7CZ13866%7CZ13867") {
            "fetch-factorial-testers-en.json"
        } else {
            return Err(FetchError::Transport(format!(
                "unexpected fixture URL: {url}"
            )));
        };
        fs::read(fixture_root().join(fixture))
            .map_err(|error| FetchError::Transport(error.to_string()))
    }
}

fn recurrence_operator_ids() -> Vec<String> {
    [
        "Z802", "Z13695", "Z13521", "Z13582", "Z13569", "Z13522", "Z13539",
    ]
    .map(str::to_owned)
    .to_vec()
}

#[test]
fn abstract_recurrences_are_derived_from_source_labels_and_validate_source_tests() {
    let cache = temp_cache("wikifunctions-recurrences");
    let _ = fs::remove_dir_all(&cache);
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(|| 1_789_344_000);
    let operators = fetch_function_descriptors(&client, &recurrence_operator_ids())
        .expect("operator descriptors");

    let fibonacci_matches =
        search_functions(&client, "nth Fibonacci number", "en").expect("search Fibonacci");
    assert_eq!(
        fibonacci_matches
            .first()
            .map(|item| item.page_title.as_str()),
        Some("Z13835")
    );
    let fibonacci_definition = fetch_function(&client, "Z13835").expect("Fibonacci definition");
    let fibonacci_abstract =
        fetch_abstract_implementation(&client, "Z13864").expect("Fibonacci abstract definition");
    assert!(
        fibonacci_abstract
            .expression
            .function_zids()
            .contains(&"Z13835".to_owned())
    );
    let fibonacci = formalize(&fibonacci_definition, &fibonacci_abstract, &operators)
        .expect("formalize Fibonacci recurrence");
    assert_eq!(fibonacci.predecessor_offsets, [1, 2]);
    let fibonacci_testers = fetch_testers(
        &client,
        &[
            "Z13869".to_owned(),
            "Z17391".to_owned(),
            "Z17398".to_owned(),
            "Z34276".to_owned(),
        ],
    )
    .expect("Fibonacci source tests");
    let fibonacci_python = python_assertions(&fibonacci, "sequence_value", &fibonacci_testers);
    assert!(fibonacci_python.contains("sequence_value(n - 1)"));
    assert!(fibonacci_python.contains("sequence_value(n - 2)"));
    run_python(&fibonacci_python);

    let factorial_matches = search_functions(&client, "factorial", "en").expect("search factorial");
    assert_eq!(
        factorial_matches
            .first()
            .map(|item| item.page_title.as_str()),
        Some("Z13667")
    );
    let factorial_definition = fetch_function(&client, "Z13667").expect("factorial definition");
    let factorial_abstract =
        fetch_abstract_implementation(&client, "Z13863").expect("factorial abstract definition");
    let factorial = formalize(&factorial_definition, &factorial_abstract, &operators)
        .expect("formalize factorial recurrence");
    assert_eq!(factorial.predecessor_offsets, [1]);
    let factorial_testers = fetch_testers(
        &client,
        &[
            "Z13840".to_owned(),
            "Z13865".to_owned(),
            "Z13866".to_owned(),
            "Z13867".to_owned(),
        ],
    )
    .expect("factorial source tests");
    let factorial_python = python_assertions(&factorial, "folded_product", &factorial_testers);
    assert!(factorial_python.contains("n * folded_product(n - 1)"));
    run_python(&factorial_python);

    for recurrence in [&fibonacci, &factorial] {
        let evidence = recurrence.to_links_notation();
        assert!(evidence.contains("source_observation"), "{evidence}");
        assert!(evidence.contains("derived_formalization"), "{evidence}");
        assert!(evidence.contains("termination_measure"), "{evidence}");
        assert!(evidence.contains("license \"CC0-1.0\""), "{evidence}");
    }
    fs::remove_dir_all(cache).expect("remove temporary cache");
}

fn run_python(source: &str) {
    let path = temp_cache("recurrence-script.py");
    fs::write(&path, source).expect("write temporary Python program");
    let status = std::process::Command::new("python3")
        .arg(&path)
        .status()
        .expect("run Python");
    let _ = fs::remove_file(path);
    assert!(status.success(), "generated source failed:\n{source}");
}

fn fixture_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/coding-discovery/wikifunctions")
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "formal-ai-coding-discovery-{label}-{}",
        std::process::id()
    ))
}

#[test]
fn search_fetch_and_language_filter_replay_the_captured_api_offline() {
    let cache = temp_cache("wikifunctions");
    let _ = fs::remove_dir_all(&cache);
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(|| 1_789_344_000);

    let matches = search_functions(&online, "greatest common divisor", "en").expect("search");
    assert_eq!(
        matches.first().map(|item| item.page_title.as_str()),
        Some("Z13612")
    );
    assert_eq!(matches[0].label, "greatest common divisor");
    assert!((matches[0].match_rate - 1.0).abs() < f64::EPSILON);

    let function = fetch_function(&online, "Z13612").expect("function");
    assert_eq!(function.argument_types, ["Z13518", "Z13518"]);
    assert_eq!(function.return_type, "Z13518");
    assert_eq!(
        function.implementation_zids,
        ["Z14707", "Z14857", "Z29084", "Z13642", "Z13639"]
    );
    assert!(function.tester_zids.contains(&"Z13613".to_owned()));
    assert_eq!(function.license, "CC0-1.0");

    let implementations =
        fetch_implementations(&online, &function.implementation_zids).expect("implementations");
    let python = implementations
        .iter()
        .filter(|implementation| implementation.language == "python")
        .collect::<Vec<_>>();
    assert_eq!(
        python
            .iter()
            .map(|item| item.zid.as_str())
            .collect::<Vec<_>>(),
        ["Z14857", "Z13642"]
    );
    assert!(python[0].code.contains("math.gcd"));
    assert!(
        implementations
            .iter()
            .any(|item| item.language == "javascript")
    );
    for implementation in &implementations {
        assert_eq!(implementation.license, "Apache-2.0");
        assert_eq!(implementation.sha256.len(), 64);
        let links = implementation.to_links_notation();
        assert!(links.contains("license \"Apache-2.0\""), "{links}");
        assert!(links.contains("sha256"), "{links}");
        assert!(links.contains("fetched_at"), "{links}");
    }

    let tester_ids = function.tester_zids[..4].to_vec();
    let testers = fetch_testers(&online, &tester_ids).expect("testers");
    assert!(
        testers
            .iter()
            .any(|test| test.arguments == ["42", "18"] && test.expected == "6")
    );

    let calls_after_live = requests.load(Ordering::SeqCst);
    let offline = CachedSourceClient::new(&cache, transport);
    let replay =
        search_functions(&offline, "greatest common divisor", "en").expect("offline replay");
    assert_eq!(replay, matches);
    assert_eq!(requests.load(Ordering::SeqCst), calls_after_live);
    fs::remove_dir_all(cache).expect("remove temporary cache");
}

#[test]
fn an_empty_search_is_not_an_error_and_offline_never_calls_the_transport() {
    let cache = temp_cache("wikifunctions-empty");
    let _ = fs::remove_dir_all(&cache);
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(|| 1_789_344_000);
    assert!(
        search_functions(&online, "qzxjvkplmnbvcx", "en")
            .expect("empty search")
            .is_empty()
    );
    let live_calls = requests.load(Ordering::SeqCst);

    let offline = CachedSourceClient::new(&cache, transport);
    assert!(
        search_functions(&offline, "qzxjvkplmnbvcx", "en")
            .expect("cached empty search")
            .is_empty()
    );
    assert_eq!(requests.load(Ordering::SeqCst), live_calls);
    fs::remove_dir_all(cache).expect("remove temporary cache");
}
