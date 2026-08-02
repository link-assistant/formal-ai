use super::*;
use std::collections::HashMap as StdHashMap;
use std::sync::Mutex;

struct StubHttp {
    responses: Mutex<StdHashMap<String, String>>,
    calls: Mutex<Vec<String>>,
}

impl StubHttp {
    fn new(responses: &[(&str, &str)]) -> Self {
        Self {
            responses: Mutex::new(
                responses
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
            ),
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl HttpClient for StubHttp {
    fn get(&self, url: &str) -> Result<String, HttpError> {
        self.calls.lock().unwrap().push(url.to_owned());
        self.responses
            .lock()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| HttpError::Status {
                status: 404,
                body: format!("stub had no response for {url}"),
            })
    }
}

fn temp_dir(slug: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "formal-ai-cache-{slug}-{}",
        std::process::id() ^ rand_u32()
    ));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    u32::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u128::from(d.subsec_nanos())),
    )
    .unwrap_or(0)
}

#[test]
fn cache_key_is_stable_across_runs() {
    let one = cache_key("https://example.com/foo");
    let two = cache_key("https://example.com/foo");
    assert_eq!(one, two);
    let other = cache_key("https://example.com/bar");
    assert_ne!(one, other);
}

#[test]
fn cache_hit_short_circuits_transport() {
    let dir = temp_dir("hit");
    let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
    let url = "https://example.com/cached";
    let (body_path, meta_path) = cache.cache_paths(url);
    fs::create_dir_all(body_path.parent().unwrap()).unwrap();
    fs::write(&body_path, "cached body").unwrap();
    fs::write(&meta_path, url).unwrap();
    assert_eq!(cache.get(url).unwrap(), "cached body");
}

#[test]
fn cache_miss_offline_returns_transport_error() {
    let dir = temp_dir("offline-miss");
    let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
    let error = cache.get("https://example.com/missing").unwrap_err();
    match error {
        HttpError::Transport(message) => {
            assert!(message.contains("cache miss"), "got: {message}");
            assert!(message.contains("FORMAL_AI_LIVE_API"), "got: {message}");
        }
        other @ HttpError::Status { .. } => {
            panic!("expected Transport error, got {other:?}")
        }
    }
}

#[test]
fn cache_miss_online_populates_and_returns_body() {
    let dir = temp_dir("online-miss");
    let url = "https://example.com/foo";
    let stub = StubHttp::new(&[(url, "fetched body")]);
    let cache = CachedHttpClient::new(&dir, stub).with_online(true);
    assert_eq!(cache.get(url).unwrap(), "fetched body");
    let again = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
    assert_eq!(again.get(url).unwrap(), "fetched body");
}

#[test]
fn cache_paths_use_semantic_subdirectories() {
    let dir = PathBuf::from("cache-root");
    let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
    let (body, meta) = cache.cache_paths("https://example.com/x");
    assert!(
        body.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("body")),
        "got: {}",
        body.display()
    );
    assert!(
        meta.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("url")),
        "got: {}",
        meta.display()
    );
    let location = cache_location("https://example.com/x");
    assert_eq!(location.directory, PathBuf::from("http-cache").join("misc"),);
    assert!(!location.stem.is_empty());
}

#[test]
fn wiktionary_url_lands_under_per_language_subdirectory() {
    let location = cache_location(
            "https://en.wiktionary.org/w/api.php?action=parse&page=apple&prop=wikitext&formatversion=2&format=json&redirects=1",
        );
    assert_eq!(
        location.directory,
        PathBuf::from("wiktionary-cache").join("en")
    );
    assert_eq!(location.stem, "apple");
}

#[test]
fn wikidata_search_url_keyed_by_search_term() {
    let location = cache_location(
            "https://www.wikidata.org/w/api.php?action=wbsearchentities&format=json&language=en&type=lexeme&srsearch=apple&limit=3",
        );
    assert_eq!(
        location.directory,
        PathBuf::from("wikidata-cache").join("search")
    );
    assert_eq!(location.stem, "apple");
}

#[test]
fn long_unicode_cache_segment_truncates_on_a_character_boundary() {
    let value = "любая формальная система либо неполна, либо противоречива";
    let segment = sanitize_segment(value);

    assert!(segment.ends_with(&cache_key(value)[..8]));
    assert!(segment.is_char_boundary(segment.len()));
    assert!(
        segment.len() <= 105,
        "got {} bytes: {segment}",
        segment.len()
    );
}

#[test]
fn wikidata_sparql_url_lands_in_sparql_bucket() {
    let location = cache_location(
        "https://query.wikidata.org/sparql?format=json&query=SELECT%20%3Flemma%20WHERE%20%7B%20%7D",
    );
    assert_eq!(
        location.directory,
        PathBuf::from("wikidata-cache").join("sparql")
    );
    assert_eq!(location.stem.len(), 16);
}

#[test]
fn parse_seed_bundle_round_trips_through_write_seed_record() {
    let body = r#"{"parse":{"title":"apple","wikitext":"* Russian: {{t+|ru|яблоко}}"}}"#;
    let url = "https://en.wiktionary.org/w/api.php?action=parse&page=apple&prop=wikitext&formatversion=2&format=json&redirects=1";
    let mut buf = String::new();
    write_seed_record(&mut buf, "wiktionary_en_apple", url, body);
    let parsed = parse_seed_bundle(&buf);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].0, url);
    assert_eq!(parsed[0].1, body);
}

#[test]
fn parse_seed_bundle_concatenates_chunked_body() {
    let bundle =
        "response_chunky\n  url \"https://example.org/x\"\n  body \"hel\"\n  body \"lo\"\n";
    let parsed = parse_seed_bundle(bundle);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].1, "hello");
}

#[test]
fn parse_seed_chunks_yields_one_pair_per_record() {
    let bundle = "response_a\n  url \"https://example.org/x\"\n  body \"hel\"\n\nresponse_b\n  url \"https://example.org/x\"\n  body \"lo\"\n";
    let chunks = parse_seed_chunks(bundle);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].0, "https://example.org/x");
    assert_eq!(chunks[0].1, "hel");
    assert_eq!(chunks[1].0, "https://example.org/x");
    assert_eq!(chunks[1].1, "lo");
}

#[test]
fn escape_round_trips_quotes_backslashes_and_unicode() {
    let cases: &[&str] = &[
        "",
        "plain ascii",
        "with \"quotes\"",
        "with \\backslash\\",
        "{\"wikitext\":\"== {{-ru-}} ==\\n=== {{з|}} ===\"}",
        "яблоко 苹果 🍎",
        "trailing-quote\"",
        "leading-quote: \"abc",
        "double\"\"middle",
    ];
    for case in cases {
        let escaped = escape_lino_string(case);
        let back = unescape_lino_string(&escaped);
        assert_eq!(back, *case, "round trip failed for {case:?}");
    }
}

#[test]
fn split_body_into_chunks_respects_char_boundaries() {
    let body = "яблоко 苹果 🍎"; // mix of 2/3/4-byte UTF-8 chars
    let chunks = split_body_into_chunks(body, 4);
    let recombined: String = chunks.concat();
    assert_eq!(recombined, body);
    for chunk in &chunks {
        assert!(chunk.chars().count() <= 4);
    }
}

#[test]
fn split_body_into_chunks_never_starts_chunk_with_quote() {
    // Boundary case: a `"` immediately after the target chunk size
    // would normally split into chunk N ending mid-stream and chunk
    // N+1 starting with `"`. The latter breaks Links Notation parsing
    // because `"` + chunk-content's leading `"` reads as a 2-quote
    // delimiter. The chunker must defer the break past any run of
    // quotes so the next chunk starts on a non-`"` byte.
    let body = "aaaa\"bbbb\"cccc";
    let chunks = split_body_into_chunks(body, 4);
    for (idx, chunk) in chunks.iter().enumerate() {
        assert!(
            !chunk.starts_with('"'),
            "chunk[{idx}] starts with a quote: {chunk:?}",
        );
    }
    let recombined: String = chunks.concat();
    assert_eq!(recombined, body);
}

#[test]
fn escaped_record_parses_as_links_notation() {
    // The committed bundle is round-tripped through
    // `lino_objects_codec::format::parse_indented` in
    // `tests/unit/data_files.rs`. Mirror that contract here so the
    // escape rules stay aligned with Links Notation expectations.
    let mut buf = String::new();
    write_seed_record(
        &mut buf,
        "demo",
        "https://example.org/q",
        r#"{"key":"value with \"escaped\" quotes","arr":[""]}"#,
    );
    lino_objects_codec::format::parse_indented(buf.trim()).unwrap_or_else(|error| {
        panic!("record should be valid Links Notation: {error}\nbuffer:\n{buf}");
    });
}

#[test]
fn seed_files_stay_under_per_file_line_cap() {
    for (name, contents) in seed_files() {
        let lines = contents.lines().count();
        assert!(
            lines <= MAX_SEED_LINES_PER_FILE,
            "{name} has {lines} lines, exceeds MAX_SEED_LINES_PER_FILE={MAX_SEED_LINES_PER_FILE}",
        );
    }
}

#[test]
fn seed_response_is_consulted_before_disk_or_transport() {
    // Use a URL we expect to be present in the committed bundle. If the
    // bundle is empty during early bootstrapping, skip the assertion —
    // the test exists to lock the precedence once the bundle is filled.
    let any_url = seed_index().keys().next().cloned();
    let Some(url) = any_url else {
        return;
    };
    let dir = temp_dir("seed-precedence");
    let cache = CachedHttpClient::new(&dir, StubHttp::new(&[])).with_online(false);
    let body = cache.get(&url).expect("seeded response must hit");
    assert!(!body.is_empty());
}
