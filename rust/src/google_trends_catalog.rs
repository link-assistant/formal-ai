//! Google Trends feed retrieval and multilingual prompt cataloging.
//!
//! Issue #498 asks for an automated path from Google Trends into reviewable test
//! cases: take the top searches, turn each search into Formal AI requests, include
//! variants in every supported language, and answer them through the normal symbolic
//! engine. The runtime reads the publisher through its source capture client.
//! Tests can pass recorded RSS bytes directly to the same conversion.
//!
//! The multilingual request wording is *data*, not code: the templates live in
//! `data/seed/google-trends-prompts.lino` and are expanded per topic (per the
//! no-hardcoded-language convention, #386). Adding a language or a request variation
//! is a change to that seed with no edit to this converter — the language set comes
//! from `supported_languages()`, so coverage generalizes automatically.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::sync::OnceLock;

use crate::seed::parser::parse_lino;
use crate::seed::supported_languages;
use crate::solver::{SolverConfig, UniversalSolver};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

/// How many Google Trends topics issue #498 requires the system to answer.
pub const GOOGLE_TRENDS_TOP_LIMIT: usize = 10;

const SOURCE_IDENTIFIER: &str = "google_trends_rss";
const DEFAULT_GEO: &str = "US";

const FEED_TTL_SECONDS: u64 = 900;

const GOOGLE_TRENDS_PROMPTS_LINO: &str =
    include_str!("../embedded/data/seed/google-trends-prompts.lino");

/// A parsed Google Trends RSS snapshot plus generated prompts and answers.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleTrendsCatalog {
    /// Snapshot source identifier.
    pub source: String,
    /// Original source URL.
    pub source_url: String,
    /// Google Trends geography code, e.g. `US`.
    pub geo: String,
    /// Google Trends locale used when collecting the feed.
    pub locale: String,
    /// Timestamp assigned when the source was captured.
    pub collected_at: String,
    /// Digest of the exact feed bytes, when captured by the source client.
    pub source_sha256: Option<String>,
    /// Ranked topics, capped at [`GOOGLE_TRENDS_TOP_LIMIT`] for issue #498.
    pub topics: Vec<GoogleTrendTopic>,
}

/// A ranked trend topic from Google Trends.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleTrendTopic {
    /// One-based rank in the feed.
    pub rank: usize,
    /// Search query/topic title.
    pub query: String,
    /// Google-provided approximate traffic label, e.g. `2000+`.
    pub approx_traffic: Option<String>,
    /// RSS publication date for the trend item.
    pub pub_date: Option<String>,
    /// News references attached by Google Trends.
    pub news_items: Vec<GoogleTrendNewsItem>,
    /// Prompt variants generated from this topic.
    pub prompts: Vec<GoogleTrendPromptVariant>,
    /// Answers for every generated prompt variant.
    pub answered: Vec<GoogleTrendPromptAnswer>,
}

/// One news item attached to a trend topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleTrendNewsItem {
    /// News headline/title.
    pub title: String,
    /// Source publication.
    pub source: String,
    /// Article URL.
    pub url: String,
}

/// A request variation generated from a trend topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleTrendPromptVariant {
    /// Language tag (`en`, `ru`, `hi`, `zh`).
    pub language: String,
    /// Stable variation key.
    pub variation_key: String,
    /// Natural-language request text.
    pub prompt: String,
}

impl GoogleTrendPromptVariant {
    /// Whether this request asks for Google Trends context.
    #[must_use]
    pub fn is_trends_context_request(&self) -> bool {
        self.variation_key == "trends_context"
    }
}

/// A prompt variant answered through the normal Formal AI engine.
#[derive(Debug, Clone, PartialEq)]
pub struct GoogleTrendPromptAnswer {
    /// Language tag for the prompt.
    pub language: String,
    /// Stable variation key for the prompt.
    pub variation_key: String,
    /// Prompt text sent to Formal AI.
    pub prompt: String,
    /// Engine intent classification.
    pub intent: String,
    /// Engine confidence.
    pub confidence: f32,
    /// Engine answer.
    pub answer: String,
    /// Standard trace/evidence links returned by the engine.
    pub evidence_links: Vec<String>,
}

/// Error returned when a Google Trends RSS feed cannot be converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoogleTrendsParseError {
    /// The feed contained no `<item>` records with titles.
    NoTopics,
}

impl fmt::Display for GoogleTrendsParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTopics => formatter.write_str("Google Trends RSS feed contained no topics"),
        }
    }
}

impl Error for GoogleTrendsParseError {}

/// Convert a Google Trends RSS XML string into ranked topics.
///
/// This parser targets the documented RSS feed shape used by
/// `https://trends.google.com/trending/rss?geo=US`: `<item>` records with
/// `<title>`, `<ht:approx_traffic>`, `<pubDate>`, and nested `<ht:news_item>`
/// records. Tests stay offline by feeding a small RSS fixture through this path.
///
/// # Errors
///
/// Returns [`GoogleTrendsParseError::NoTopics`] when no item title can be found.
pub fn parse_google_trends_rss(
    rss_xml: &str,
    geo: &str,
    locale: &str,
) -> Result<GoogleTrendsCatalog, GoogleTrendsParseError> {
    let channel = rss_xml
        .split_once("<rss")
        .and_then(|(_, tag)| {
            if !tag.starts_with(|ch: char| ch == '>' || ch.is_ascii_whitespace()) {
                return None;
            }
            let (_, content) = tag.split_once('>')?;
            let (feed, _) = content.split_once("</rss>")?;
            extract_blocks(feed, "channel").into_iter().next()
        })
        .ok_or(GoogleTrendsParseError::NoTopics)?;
    let mut topics = Vec::new();
    for block in extract_blocks(channel, "item") {
        if topics.len() == GOOGLE_TRENDS_TOP_LIMIT {
            break;
        }
        let Some(query) = extract_text(block, "title") else {
            continue;
        };
        if query.trim().is_empty() || query.encode_utf16().count() > 256 {
            continue;
        }
        let news_items = extract_blocks(block, "ht:news_item")
            .into_iter()
            .filter_map(parse_news_item)
            .collect();
        topics.push(GoogleTrendTopic {
            rank: topics.len() + 1,
            query,
            approx_traffic: extract_text(block, "ht:approx_traffic"),
            pub_date: extract_text(block, "pubDate"),
            news_items,
            prompts: Vec::new(),
            answered: Vec::new(),
        });
    }

    if topics.is_empty() {
        return Err(GoogleTrendsParseError::NoTopics);
    }

    Ok(GoogleTrendsCatalog {
        source: SOURCE_IDENTIFIER.to_owned(),
        source_url: feed_url(geo).unwrap_or_default(),
        geo: geo.to_string(),
        locale: locale.to_string(),
        collected_at: String::new(),
        source_sha256: None,
        topics,
    })
}

fn feed_url(geo: &str) -> Option<String> {
    if geo.len() != 2 || !geo.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return None;
    }
    crate::seed::source_record(SOURCE_IDENTIFIER).map(|source| source.api_url(&[("geo", geo)]))
}

fn default_locale() -> Option<String> {
    crate::seed::source_record(SOURCE_IDENTIFIER)?
        .api_language
        .into_iter()
        .next()
}

/// Read the ranked topics for one region through the caller's capture client.
pub fn fetch_google_trends_feed<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    geo: &str,
    locale: &str,
) -> Option<GoogleTrendsCatalog> {
    let url = feed_url(geo)?;
    let capture = client.fetch(&url).ok()?;
    let rss = std::str::from_utf8(capture.bytes()).ok()?;
    let mut snapshot = parse_google_trends_rss(rss, geo, locale).ok()?;
    snapshot.source_url = capture.source_url().into();
    snapshot.collected_at = capture.fetched_at().into();
    snapshot.source_sha256 = Some(capture.sha256().into());
    Some(snapshot)
}

/// Open the configured regional feed, respecting the runtime's network policy.
pub fn runtime_google_trends_feed(
    geo: &str,
    locale: &str,
    online: bool,
) -> Option<GoogleTrendsCatalog> {
    let client = CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        CurlSourceTransport,
    )
    .with_ttl_seconds(FEED_TTL_SECONDS)
    .with_online(online && crate::solver_config::read_network_allowed());
    fetch_google_trends_feed(&client, geo, locale)
}

/// Resolve the requested feed geography from a geo argument or the default feed.
pub fn requested_geo(task: &str) -> Option<String> {
    let lower = task.to_ascii_lowercase();
    let mut words = lower
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '?' | '&' | '#'))
        .filter(|word| !word.is_empty());
    let mut geo = DEFAULT_GEO;
    while let Some(word) = words.next() {
        if let Some(value) = word.strip_prefix("geo=") {
            geo = value;
            break;
        }
        if word == "geo" {
            geo = words.next()?;
            break;
        }
    }
    (geo.len() == 2 && geo.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then(|| geo.to_ascii_uppercase())
}

/// Load the current feed identified by a recipe request.
pub fn feed_for_task(task: &str) -> Option<GoogleTrendsCatalog> {
    runtime_google_trends_feed(&requested_geo(task)?, &default_locale()?, true)
}

/// Load the default feed used by the catalog API.
#[must_use]
pub fn google_trends_catalog() -> GoogleTrendsCatalog {
    feed_for_task("")
        .map(answer_google_trends_catalog)
        .unwrap_or_else(|| GoogleTrendsCatalog {
            source: SOURCE_IDENTIFIER.to_owned(),
            source_url: feed_url(DEFAULT_GEO).unwrap_or_default(),
            geo: DEFAULT_GEO.to_owned(),
            locale: default_locale().unwrap_or_default(),
            collected_at: String::new(),
            source_sha256: None,
            topics: Vec::new(),
        })
}

/// Render a parsed Trends snapshot as seed Links Notation.
///
/// The refresh flow is:
///
/// ```text
/// curl -sL 'https://trends.google.com/trending/rss?geo=US&hl=ru' \
///   | cargo run --example issue_498_parse_google_trends_rss \
///   > /tmp/google-trends-snapshot.lino
/// ```
#[must_use]
pub fn render_google_trends_snapshot_lino(snapshot: &GoogleTrendsCatalog) -> String {
    let mut out = String::from("google_trends_snapshot\n");
    out.push_str("  record_type \"google_trends_snapshot\"\n");
    field(&mut out, 2, "source", &snapshot.source);
    field(&mut out, 2, "source_url", &snapshot.source_url);
    field(&mut out, 2, "geo", &snapshot.geo);
    field(&mut out, 2, "locale", &snapshot.locale);
    if !snapshot.collected_at.trim().is_empty() {
        field(&mut out, 2, "collected_at", &snapshot.collected_at);
    }
    if let Some(digest) = &snapshot.source_sha256 {
        field(&mut out, 2, "source_sha256", digest);
    }
    for topic in snapshot.topics.iter().take(GOOGLE_TRENDS_TOP_LIMIT) {
        out.push_str("google_trend_topic\n");
        out.push_str("  record_type \"google_trend_topic\"\n");
        let _ = writeln!(out, "  rank \"{}\"", topic.rank);
        field(&mut out, 2, "query", &topic.query);
        if let Some(traffic) = &topic.approx_traffic {
            field(&mut out, 2, "approx_traffic", traffic);
        }
        if let Some(pub_date) = &topic.pub_date {
            field(&mut out, 2, "pub_date", pub_date);
        }
        for item in &topic.news_items {
            out.push_str("  news_item\n");
            out.push_str("    record_type \"google_trend_news_item\"\n");
            field(&mut out, 4, "title", &item.title);
            field(&mut out, 4, "source", &item.source);
            field(&mut out, 4, "url", &item.url);
        }
    }
    format!("{}\n", out.trim_end())
}

/// Expand and answer the ranked topics from a supplied feed.
#[must_use]
pub fn answer_google_trends_catalog(mut catalog: GoogleTrendsCatalog) -> GoogleTrendsCatalog {
    catalog.topics.sort_by_key(|topic| topic.rank);
    catalog.topics.truncate(GOOGLE_TRENDS_TOP_LIMIT);

    // A committed snapshot must not depend on the process-wide live-retrieval
    // default. Keep the normal solver/standing-requirement path, with network
    // access explicitly disabled for this deterministic catalog projection.
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    for topic in &mut catalog.topics {
        topic.prompts = prompt_variants_for_topic(&topic.query);
        topic.answered = topic
            .prompts
            .iter()
            .map(|prompt| {
                let answer = crate::dreaming_application::solve_with_standing_requirements(
                    &solver,
                    &prompt.prompt,
                    &[],
                    &[],
                );
                GoogleTrendPromptAnswer {
                    language: prompt.language.clone(),
                    variation_key: prompt.variation_key.clone(),
                    prompt: prompt.prompt.clone(),
                    intent: answer.intent,
                    confidence: answer.confidence,
                    answer: collapse_whitespace(&answer.answer),
                    evidence_links: answer.evidence_links,
                }
            })
            .collect();
    }

    catalog
}

/// A multilingual request template loaded from the prompt seed.
///
/// The natural-language pattern lives in `data/seed/google-trends-prompts.lino`
/// (per the no-hardcoded-language convention, #386); `{query}` is substituted with
/// the trending search term when the catalog is built.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GoogleTrendPromptTemplate {
    language: String,
    variation_key: String,
    text: String,
}

/// The placeholder in a seeded template replaced by the trending search term.
const QUERY_PLACEHOLDER: &str = "{query}";

/// Expand every seeded prompt template for `search`, in `supported_languages()`
/// order and, within each language, in seed order.
///
/// Adding a language or a request variation is a data-only change to the prompt
/// seed — this converter iterates whatever the seed declares, so no Rust edit is
/// needed to extend catalog coverage.
pub(crate) fn prompt_variants_for_topic(search: &str) -> Vec<GoogleTrendPromptVariant> {
    let templates = prompt_templates();
    supported_languages()
        .iter()
        .flat_map(|language| {
            templates
                .iter()
                .filter(|template| template.language == *language)
                .map(|template| GoogleTrendPromptVariant {
                    language: template.language.clone(),
                    variation_key: template.variation_key.clone(),
                    prompt: template.text.replace(QUERY_PLACEHOLDER, search),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The seeded prompt templates, parsed once from the prompt catalog seed.
fn prompt_templates() -> &'static [GoogleTrendPromptTemplate] {
    static TEMPLATES: OnceLock<Vec<GoogleTrendPromptTemplate>> = OnceLock::new();
    TEMPLATES.get_or_init(load_prompt_templates)
}

fn load_prompt_templates() -> Vec<GoogleTrendPromptTemplate> {
    let tree = parse_lino(GOOGLE_TRENDS_PROMPTS_LINO);
    tree.children
        .iter()
        .filter(|record| record.find_child_value("record_type") == "google_trend_prompt_template")
        .filter_map(|record| {
            let language = record.find_child_value("language").trim();
            let variation_key = record.find_child_value("variation").trim();
            let text = record.find_child_value("text");
            if language.is_empty() || variation_key.is_empty() || !text.contains(QUERY_PLACEHOLDER)
            {
                return None;
            }
            Some(GoogleTrendPromptTemplate {
                language: language.to_string(),
                variation_key: variation_key.to_string(),
                text: text.to_string(),
            })
        })
        .collect()
}

fn parse_news_item(block: &str) -> Option<GoogleTrendNewsItem> {
    let title = extract_text(block, "ht:news_item_title").filter(|value| !value.is_empty())?;
    let source = extract_text(block, "ht:news_item_source").unwrap_or_default();
    let url = extract_text(block, "ht:news_item_url").unwrap_or_default();
    Some(GoogleTrendNewsItem { title, source, url })
}

fn extract_blocks<'a>(text: &'a str, tag: &str) -> Vec<&'a str> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let mut blocks = Vec::new();
    let mut offset = 0usize;

    while let Some(start) = text[offset..].find(&start_tag) {
        let content_start = offset + start + start_tag.len();
        let Some(end) = text[content_start..].find(&end_tag) else {
            break;
        };
        let content_end = content_start + end;
        blocks.push(&text[content_start..content_end]);
        offset = content_end + end_tag.len();
    }

    blocks
}

fn extract_text(text: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let content_start = text.find(&start_tag)? + start_tag.len();
    let content_end = content_start + text[content_start..].find(&end_tag)?;
    let raw = text[content_start..content_end].trim();
    if raw.is_empty() {
        return None;
    }
    Some(collapse_whitespace(&decode_xml_text(raw)))
}

fn decode_xml_text(raw: &str) -> String {
    if let Some(text) = raw
        .strip_prefix("<![CDATA[")
        .and_then(|value| value.strip_suffix("]]>"))
    {
        return text.to_owned();
    }
    raw.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn field(out: &mut String, indent: usize, name: &str, value: &str) {
    let spaces = " ".repeat(indent);
    let _ = writeln!(out, "{spaces}{name} \"{}\"", escape_lino_value(value));
}

fn escape_lino_value(value: &str) -> String {
    collapse_whitespace(value)
        .replace('\\', "\\\\")
        .replace('"', "'")
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
