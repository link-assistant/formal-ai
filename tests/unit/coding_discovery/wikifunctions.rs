use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::coding_function_catalog::wikifunctions::{
    fetch_function, fetch_implementations, fetch_testers, search_functions,
};
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let fixture =
            if url.contains("wikilambdasearch_functions_search=greatest%20common%20divisor") {
                "search-greatest-common-divisor-en.json"
            } else if url.contains("wikilambdasearch_functions_search=qzxjvkplmnbvcx") {
                "search-no-match-en.json"
            } else if url.contains("zids=Z13612") {
                "fetch-Z13612-en.json"
            } else if url.contains("zids=Z14707%7CZ14857%7CZ29084%7CZ13642%7CZ13639") {
                "fetch-gcd-implementations-en.json"
            } else if url.contains("zids=Z13614%7CZ13615%7CZ13616%7CZ13613") {
                "fetch-gcd-testers-en.json"
            } else {
                return Err(FetchError::Transport(format!(
                    "unexpected fixture URL: {url}"
                )));
            };
        fs::read(fixture_root().join(fixture))
            .map_err(|error| FetchError::Transport(error.to_string()))
    }
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
