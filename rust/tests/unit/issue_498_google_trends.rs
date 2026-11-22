use std::collections::BTreeSet;

use formal_ai::{
    GOOGLE_TRENDS_TOP_LIMIT, GoogleTrendPromptVariant, GoogleTrendsCatalog,
    answer_google_trends_catalog, fetch_google_trends_feed, parse_google_trends_rss,
    supported_languages,
};

use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};

pub(super) const HISTORICAL_FEED: &str =
    include_str!("../../../docs/case-studies/issue-498/raw-data/google-trends-us-rss.xml");
const FEED_URL: &str = "https://trends.google.com/trending/rss?geo=US";
const DE_FEED_URL: &str = "https://trends.google.com/trending/rss?geo=DE";
const DE_FEED: &str = include_str!("../../../docs/case-studies/issue-1613/evidence/trends/DE.xml");

struct RecordedFeed;

impl SourceTransport for RecordedFeed {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match url {
            FEED_URL => Ok(HISTORICAL_FEED.as_bytes().to_vec()),
            DE_FEED_URL => Ok(DE_FEED.as_bytes().to_vec()),
            _ => Err(FetchError::OfflineCacheMiss(url.to_owned())),
        }
    }
}

struct ReplayDirectory(std::path::PathBuf);

impl ReplayDirectory {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "formal-ai-trends-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("isolated replay directory");
        Self(path)
    }
}

impl Drop for ReplayDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn captured_feed() -> GoogleTrendsCatalog {
    let directory = ReplayDirectory::new();
    fetch_google_trends_feed(
        &CachedSourceClient::new(&directory.0, RecordedFeed).with_online(true),
        "US",
        "en",
    )
    .expect("recorded publisher feed")
}

pub(super) fn with_trends_replay<R>(run: impl FnOnce() -> R) -> R {
    let directory = ReplayDirectory::new();
    let client = CachedSourceClient::new(&directory.0, RecordedFeed).with_online(true);
    client.fetch(FEED_URL).expect("recorded publisher feed");
    temp_env::with_vars(
        [
            ("FORMAL_AI_SOURCE_CACHE_DIR", Some(directory.0.as_os_str())),
            ("FORMAL_AI_OFFLINE", Some(std::ffi::OsStr::new("1"))),
        ],
        run,
    )
}

const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rss xmlns:ht="https://trends.google.com/trending/rss" version="2.0">
  <channel>
    <title>Daily Search Trends</title>
    <link>https://trends.google.com/trending/rss?geo=US</link>
    <item>
      <title>blue jays</title>
      <ht:approx_traffic>2000+</ht:approx_traffic>
      <link>https://trends.google.com/trending/rss?geo=US</link>
      <pubDate>Wed, 8 Jul 2026 12:50:00 -0700</pubDate>
      <ht:news_item>
        <ht:news_item_title>Blue Jays vs. Giants Game Thread</ht:news_item_title>
        <ht:news_item_url>https://sports.yahoo.com/articles/blue-jays-vs-giants-game-193143634.html</ht:news_item_url>
        <ht:news_item_source>Yahoo Sports</ht:news_item_source>
      </ht:news_item>
    </item>
    <item>
      <title>grok 4.5</title>
      <ht:approx_traffic>500+</ht:approx_traffic>
      <link>https://trends.google.com/trending/rss?geo=US</link>
      <pubDate>Wed, 8 Jul 2026 12:50:00 -0700</pubDate>
    </item>
  </channel>
</rss>"#;

#[test]
fn the_regional_feed_comes_from_verified_source_bytes() {
    use formal_ai::google_trends_catalog::requested_geo;

    let directory = ReplayDirectory::new();
    let offline = CachedSourceClient::new(&directory.0, RecordedFeed).with_online(false);
    assert!(fetch_google_trends_feed(&offline, "DE", "en").is_none());
    assert_eq!(requested_geo("Catalog geo  de?").as_deref(), Some("DE"));
    assert_eq!(requested_geo("Catalog geo=123"), None);
    assert_eq!(requested_geo("Catalog").as_deref(), Some("US"));

    let online = CachedSourceClient::new(&directory.0, RecordedFeed).with_online(true);
    let german = fetch_google_trends_feed(&online, "DE", "en").expect("German RSS");
    let american = fetch_google_trends_feed(&online, "US", "en").expect("American RSS");
    assert_eq!(german.source_url, DE_FEED_URL);
    assert_eq!(german.topics[0].query, "juventus turin");
    assert_ne!(german.topics[0].query, american.topics[0].query);
    assert_ne!(german.source_sha256, american.source_sha256);
    assert_eq!(fetch_google_trends_feed(&offline, "DE", "en"), Some(german));
}

#[test]
fn google_trends_rss_is_converted_to_ranked_topics() {
    let snapshot = parse_google_trends_rss(SAMPLE_RSS, "US", "ru")
        .expect("the RSS converter should parse the Google Trends feed shape");

    assert_eq!(snapshot.source, "google_trends_rss");
    assert_eq!(snapshot.geo, "US");
    assert_eq!(snapshot.locale, "ru");
    assert_eq!(snapshot.topics.len(), 2);

    let first = &snapshot.topics[0];
    assert_eq!(first.rank, 1);
    assert_eq!(first.query, "blue jays");
    assert_eq!(first.approx_traffic.as_deref(), Some("2000+"));
    assert_eq!(
        first.pub_date.as_deref(),
        Some("Wed, 8 Jul 2026 12:50:00 -0700")
    );
    assert_eq!(
        first.news_items.first().map(|item| item.source.as_str()),
        Some("Yahoo Sports"),
    );
}

#[test]
fn prompts_are_generated_from_the_seed_templates() {
    // The multilingual wording lives in data, not Rust (#386). Every generated
    // request must trace back to a seeded template: substituting the trending query
    // back with the `{query}` placeholder must reproduce a template that appears
    // verbatim in `data/seed/google-trends-prompts.lino`. If someone re-introduced a
    // hardcoded prompt in the converter, this test would fail.
    const PROMPT_SEED: &str = include_str!("../../../data/seed/google-trends-prompts.lino");
    let catalog = answer_google_trends_catalog(captured_feed());

    for topic in &catalog.topics {
        assert!(!topic.prompts.is_empty(), "topic {topic:?} has no prompts");
        for prompt in &topic.prompts {
            let templated = prompt.prompt.replace(&topic.query, "{query}");
            assert!(
                PROMPT_SEED.contains(&templated),
                "prompt {:?} (language {}) is not derived from a seeded template",
                prompt.prompt,
                prompt.language,
            );
        }
    }
}

#[test]
fn checked_in_google_trends_catalog_covers_top_ten_in_all_supported_languages() {
    let catalog = answer_google_trends_catalog(captured_feed());

    assert!(
        catalog
            .source_sha256
            .as_ref()
            .is_some_and(|digest| digest.len() == 64)
    );
    assert_eq!(catalog.topics[0].query, "julián andrés quiñones");
    assert_eq!(
        catalog.topics.len(),
        GOOGLE_TRENDS_TOP_LIMIT,
        "the committed snapshot should keep exactly the top 10 Trends topics",
    );
    assert_eq!(catalog.geo, "US");
    assert_eq!(catalog.locale, "en");
    assert_eq!(
        catalog.source_url,
        "https://trends.google.com/trending/rss?geo=US"
    );

    // Coverage is derived from the data, not hardcoded: every supported language
    // (from `supported_languages()`) must be present, and each language must
    // contribute the same number of request variations. Adding a language or a
    // variation to the prompt seed extends the catalog with no code or test edit.
    let supported: BTreeSet<String> = supported_languages().into_iter().collect();
    assert!(
        !supported.is_empty(),
        "the seed must declare at least one supported language",
    );

    let mut seen_ranks = BTreeSet::new();
    for topic in &catalog.topics {
        assert!(
            seen_ranks.insert(topic.rank),
            "rank should be unique: {topic:?}"
        );
        assert_ne!(topic.query.trim(), "");

        let languages: BTreeSet<String> = topic
            .prompts
            .iter()
            .map(|prompt| prompt.language.clone())
            .collect();
        assert_eq!(
            languages, supported,
            "each topic must cover exactly the supported languages: {topic:?}",
        );

        // Uniform, non-empty coverage per language proves the seed drives the shape.
        let per_language = topic.prompts.len() / languages.len();
        assert!(per_language >= 1, "each language needs a request variation");
        for language in &supported {
            let count = topic
                .prompts
                .iter()
                .filter(|prompt| &prompt.language == language)
                .count();
            assert_eq!(
                count, per_language,
                "language {language} should contribute {per_language} variations like every other",
            );
        }
        assert_eq!(
            topic.prompts.len(),
            per_language * languages.len(),
            "prompt count should be languages × variations",
        );
        assert_eq!(
            topic.answered.len(),
            topic.prompts.len(),
            "every prompt variation should be answered through the normal Formal AI path",
        );

        assert!(
            topic
                .prompts
                .iter()
                .any(GoogleTrendPromptVariant::is_trends_context_request),
            "topic should include a trend-specific request variation: {topic:?}",
        );

        for answered in &topic.answered {
            assert!(answered.prompt.ends_with('?'));
            assert_ne!(answered.answer.trim(), "");
            assert!(
                answered
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with("trace:")),
                "answers should preserve standard trace evidence: {answered:?}",
            );
        }
    }
}
