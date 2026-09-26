use std::fs;

use formal_ai::coding_function_catalog::python_docs::fetch_index;
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

#[derive(Clone, Copy)]
struct FixtureTransport;

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let file = url
            .rsplit('/')
            .next()
            .filter(|name| {
                std::path::Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("html"))
            })
            .ok_or_else(|| FetchError::Transport(format!("unexpected docs URL: {url}")))?;
        fs::read(fixture_root().join(file))
            .map_err(|error| FetchError::Transport(error.to_string()))
    }
}

fn fixture_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/coding-discovery/python-docs")
}

#[test]
fn official_documentation_becomes_a_ranked_provenance_index() {
    let source = formal_ai::seed::source_record("python_docs")
        .expect("Python documentation registry record");
    assert_eq!(source.license_name, "PSF-2.0");
    assert!(source.api.contains("{title}"));
    assert!(source.cache_path.ends_with("python-docs/"));
    let cache = std::env::temp_dir().join(format!(
        "formal-ai-python-docs-index-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&cache);
    let client = CachedSourceClient::new(&cache, FixtureTransport)
        .with_online(true)
        .with_clock(|| 1_789_344_000);
    let index = fetch_index(&client).expect("Python documentation index");

    let sum = index.parts_for_phrase("sum of all the integers");
    assert_eq!(
        sum.first().map(|part| part.symbol.as_str()),
        Some("sum"),
        "{:?}",
        sum.iter()
            .take(8)
            .map(|part| (
                &part.symbol,
                part.match_score("sum of all the integers"),
                &part.description
            ))
            .collect::<Vec<_>>()
    );
    let product = index.parts_for_phrase("product of all the integers");
    assert_eq!(
        product.first().map(|part| part.symbol.as_str()),
        Some("math.prod")
    );
    let largest = index.parts_for_phrase("return the largest element");
    assert_eq!(
        largest.first().map(|part| part.symbol.as_str()),
        Some("max")
    );

    for part in [sum[0], product[0], largest[0]] {
        assert_eq!(part.license, source.license_name);
        assert_eq!(part.sha256.len(), 64);
        assert!(
            part.source_url
                .starts_with("https://docs.python.org/3.12/library/")
        );
        assert!(part.to_links_notation().contains("stdlib_part"));
    }

    let count = index
        .part("str.count")
        .expect("str.count extracted from stdtypes.html");
    assert!(count.description.to_lowercase().contains("non-overlapping"));
    assert!(
        count.match_score("count overlapping occurrences") < 1.0,
        "the documented non-overlapping operation must rank below the structural overlapping meaning: {count:?}"
    );

    fs::remove_dir_all(cache).expect("remove temporary cache");
}
