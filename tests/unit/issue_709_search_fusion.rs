//! Acceptance regressions for issue #709: statement-level search fusion.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use formal_ai::{
    execute_search_fusion, telegram_html_from_markdown, try_web_search_with_client,
    CachedSourceClient, EventLog, FetchError, SearchSourceClassification, SourceTier,
    SourceTransport,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                r#"{"AbstractURL":"https://facts.invalid/original","Heading":"Original handbook","AbstractText":"Apple is a fruit.","RelatedTopics":[{"FirstURL":"https://facts.invalid/report","Text":"Independent report - Яблоко это фрукт."},{"FirstURL":"https://facts.invalid/repost","Text":"Copied post - Apple is a fruit."}]}"#
                    .as_bytes()
                    .to_vec(),
            );
        }
        match url {
            "https://facts.invalid/original" => Ok(b"Apple is a fruit.\n".to_vec()),
            "https://facts.invalid/report" => Ok("Яблоко это фрукт.\n".as_bytes().to_vec()),
            "https://facts.invalid/repost" => Ok(b"Apple is a fruit.\n".to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

#[derive(Clone, Default)]
struct ConflictTransport;

impl SourceTransport for ConflictTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                br#"{"AbstractURL":"https://speed.invalid/official","Heading":"Official benchmark","AbstractText":"The parser is fast.","RelatedTopics":[{"FirstURL":"https://speed.invalid/lab","Text":"Independent lab - The parser is not fast."}]}"#
                    .to_vec(),
            );
        }
        match url {
            "https://speed.invalid/official" => Ok(b"The parser is fast.\n".to_vec()),
            "https://speed.invalid/lab" => Ok(b"The parser is not fast.\n".to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

#[derive(Clone, Default)]
struct ForeignOnlyTransport;

impl SourceTransport for ForeignOnlyTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                r#"{"AbstractURL":"https://facts.invalid/report","Heading":"Russian handbook","AbstractText":"Яблоко это фрукт.","RelatedTopics":[]}"#
                    .as_bytes()
                    .to_vec(),
            );
        }
        match url {
            "https://facts.invalid/report" => Ok("Яблоко это фрукт.\n".as_bytes().to_vec()),
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

const fn fixed_time() -> u64 {
    1_753_444_800
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-709-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn classify(url: &str) -> SearchSourceClassification {
    if url.ends_with("/original") {
        SearchSourceClassification::new(SourceTier::OriginalFirstParty, "en")
    } else if url.ends_with("/report") {
        SearchSourceClassification::new(SourceTier::IndependentCorroboration, "ru")
    } else {
        SearchSourceClassification::new(SourceTier::Unoriginal, "en")
    }
}

#[test]
fn cached_sources_are_formalized_merged_ranked_and_replayed_deterministically() {
    let cache = temp_cache("merge");
    let transport = FixtureTransport::default();
    let requests = Arc::clone(&transport.requests);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);

    let first = execute_search_fusion(&online, "apple taxonomy", "en", 3, classify)
        .expect("captured search fusion");

    assert_eq!(first.research.search.fused.len(), 3);
    assert_eq!(first.research.pages.len(), 3);
    assert!(first
        .observations
        .iter()
        .any(|item| item.origin.as_str() == "search_hit"));
    assert!(first
        .observations
        .iter()
        .any(|item| item.origin.as_str() == "fetched_source"));
    assert!(first
        .observations
        .iter()
        .all(|item| !item.source_url.is_empty() && !item.formalization.is_empty()));

    let statement = first
        .answer
        .statements
        .iter()
        .find(|item| item.text == "Apple is a fruit.")
        .expect("cross-language fact");
    assert_eq!(statement.source_count, 2, "the repost is ignored");
    assert_eq!(statement.sources[0].tier, SourceTier::OriginalFirstParty);
    assert_eq!(
        statement.sources[1].tier,
        SourceTier::IndependentCorroboration
    );
    assert_eq!(first.ignored_sources, vec!["https://facts.invalid/repost"]);
    assert_eq!(first.answer.statements[0].id, statement.id);

    let proposal = first.learning_proposal();
    for page in &first.research.pages {
        assert!(proposal.contains(page.capture.sha256()));
    }
    assert!(proposal.contains("search_statement_formalization"));
    assert!(proposal.contains("search_statement_merge"));
    assert!(proposal.contains("search_statement_rank"));

    let offline = CachedSourceClient::new(&cache, transport);
    let replay = execute_search_fusion(&offline, "apple taxonomy", "en", 3, classify)
        .expect("offline replay");
    assert!(replay
        .research
        .pages
        .iter()
        .all(|page| page.capture.cached()));
    assert_eq!(replay.render_markdown(), first.render_markdown());
    assert_eq!(replay.trace(), first.trace());
    assert_eq!(replay.learning_proposal(), proposal);
    assert_eq!(requests.load(Ordering::SeqCst), 4);

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn decisive_foreign_language_fact_is_deformalized_in_the_query_language() {
    let cache = temp_cache("cross-language");
    let client = CachedSourceClient::new(&cache, ForeignOnlyTransport)
        .with_online(true)
        .with_clock(fixed_time);
    let execution = execute_search_fusion(&client, "apple taxonomy", "en", 1, |_| {
        SearchSourceClassification::new(SourceTier::OriginalFirstParty, "ru")
    })
    .expect("captured search fusion");
    let statement = execution
        .answer
        .statements
        .iter()
        .find(|item| {
            item.sources
                .iter()
                .any(|source| source.url.ends_with("/report"))
        })
        .expect("foreign source contributes the decisive fact");

    assert_eq!(statement.text, "Apple is a fruit.");
    assert!(statement.semantic_links.contains("wikidata:Q89"));
    assert!(statement.semantic_links.contains("wikidata:P31"));
    assert!(statement.semantic_links.contains("wikidata:Q3314483"));
    assert_eq!(statement.sources.len(), 1);
    assert_eq!(statement.sources[0].language, "ru");
    assert!(statement.sources[0].quote.contains("Яблоко"));

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn contradictory_sources_keep_both_sides_with_tiers_and_posteriors() {
    let cache = temp_cache("conflict");
    let client = CachedSourceClient::new(&cache, ConflictTransport)
        .with_online(true)
        .with_clock(fixed_time);
    let execution = execute_search_fusion(&client, "parser speed", "en", 2, |url| {
        if url.ends_with("/official") {
            SearchSourceClassification::new(SourceTier::OriginalFirstParty, "en")
        } else {
            SearchSourceClassification::new(SourceTier::IndependentCorroboration, "en")
        }
    })
    .expect("conflicting search fusion");

    let contested: Vec<_> = execution
        .answer
        .statements
        .iter()
        .filter(|statement| statement.conflict == Some("source_disagreement"))
        .collect();
    assert_eq!(contested.len(), 2, "neither side may be silently dropped");
    assert!(contested.iter().any(|item| item.text.contains("not fast")));
    assert!(contested.iter().any(|item| !item.text.contains("not fast")));
    assert!(contested
        .iter()
        .all(|item| (0.0..=1.0).contains(&item.posterior.get())));
    assert_ne!(contested[0].posterior, contested[1].posterior);
    assert!(contested
        .iter()
        .flat_map(|item| &item.sources)
        .any(|source| source.tier == SourceTier::OriginalFirstParty));
    assert!(execution.trace().contains("conflict:source_disagreement"));

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn presentation_normalizes_source_url_title_quote_and_read_more() {
    let cache = temp_cache("presentation");
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let execution = execute_search_fusion(&client, "apple taxonomy", "en", 3, classify)
        .expect("captured search fusion");
    let rendered = execution.render_markdown();

    assert!(rendered.contains("Original handbook"));
    assert!(rendered.contains("https://facts.invalid/original"));
    assert!(rendered.contains("> Apple is a fruit."));
    assert!(rendered.contains("[Read more](https://facts.invalid/original)"));
    assert!(rendered.contains("posterior="));
    assert!(rendered.contains("source_tier=original_first_party"));
    assert_eq!(rendered.matches("Original handbook").count(), 1);

    let mut log = EventLog::new();
    execution.record(&mut log);
    assert_eq!(
        log.events()
            .iter()
            .filter(|event| event.kind == "search_fusion:formalization")
            .count(),
        execution.observations.len()
    );

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn cli_http_and_telegram_use_the_same_ranked_source_contract() {
    let cache = temp_cache("surfaces");
    let client = CachedSourceClient::new(&cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(fixed_time);
    let mut log = EventLog::new();
    let answer = try_web_search_with_client(
        "Search the web for apple taxonomy",
        "search the web for apple taxonomy",
        &mut log,
        &client,
    )
    .expect("web search route");

    assert!(answer.answer.contains("Apple is a fruit."));
    assert!(answer.answer.contains("posterior="));
    assert!(answer.answer.contains("Original handbook"));
    assert!(answer
        .evidence_links
        .iter()
        .any(|link| link.starts_with("search_fusion:formalization:")));

    let telegram = telegram_html_from_markdown(&answer.answer);
    assert!(telegram.contains("<a href=\"https://facts.invalid/original\">Original handbook</a>"));
    assert!(telegram.contains("<blockquote>Apple is a fruit.</blockquote>"));
    assert!(telegram.contains("<a href=\"https://facts.invalid/original\">Read more</a>"));
    assert!(!telegram.contains("[Read more]"));

    fs::remove_dir_all(cache).expect("remove fixture cache");
}
