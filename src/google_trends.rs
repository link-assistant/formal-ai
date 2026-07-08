//! Google Trends prompt-catalog generation for issue #499.
//!
//! The live Google Trends feed is intentionally kept out of normal test execution:
//! CI consumes a committed RSS snapshot and this deterministic converter turns it
//! into self-authored Formal AI request cases. The snapshot gives reviewers the
//! current trend topics and provenance; the generated prompts are local test data.

use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

/// The public Trending Now RSS endpoint used for the issue #499 capture.
pub const GOOGLE_TRENDS_US_RSS_URL: &str = "https://trends.google.com/trending/rss?geo=US";

/// The language set currently exercised by the multilingual prompt matrix.
pub const SUPPORTED_TREND_PROMPT_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

/// One news metadata row attached to a Google Trends RSS item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendNewsItem {
    pub title: String,
    pub url: String,
    pub source: String,
}

/// One Google Trends RSS item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendItem {
    pub rank: usize,
    pub title: String,
    pub approx_traffic: String,
    pub pub_date: String,
    pub link: String,
    pub news: Vec<TrendNewsItem>,
}

/// One Formal AI request derived from a trend topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendPromptCase {
    pub id: String,
    pub trend_rank: usize,
    pub language: String,
    pub topic: String,
    pub prompt: String,
    pub expected_use: String,
    pub expected_intent: String,
}

/// The full generated prompt suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendPromptSuite {
    pub geo: String,
    pub captured_at: String,
    pub source_url: String,
    pub source_snapshot: String,
    pub minimum_prompt_count: usize,
    pub trends: Vec<TrendItem>,
    pub prompt_cases: Vec<TrendPromptCase>,
}

/// Rendering configuration for a committed Google Trends prompt suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendPromptSuiteConfig {
    pub geo: String,
    pub captured_at: String,
    pub source_url: String,
    pub source_snapshot: String,
    pub top_n: usize,
}

impl Default for TrendPromptSuiteConfig {
    fn default() -> Self {
        Self {
            geo: "US".to_owned(),
            captured_at: "unknown".to_owned(),
            source_url: GOOGLE_TRENDS_US_RSS_URL.to_owned(),
            source_snapshot: "docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml"
                .to_owned(),
            top_n: 10,
        }
    }
}

/// Errors produced while parsing or rendering the Trends prompt suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoogleTrendsError {
    MissingItems,
    NotEnoughTrends { requested: usize, available: usize },
}

impl fmt::Display for GoogleTrendsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingItems => formatter.write_str("Google Trends RSS contained no items"),
            Self::NotEnoughTrends {
                requested,
                available,
            } => write!(
                formatter,
                "Google Trends RSS contained {available} item(s), but {requested} were requested",
            ),
        }
    }
}

impl Error for GoogleTrendsError {}

/// Parse a Google Trends RSS document into ranked trend items.
///
/// The feed shape is small and stable for this use case (`rss/channel/item` plus the
/// `ht:*` metadata namespace), so this parser extracts the known tag bodies without
/// pulling a new XML dependency into the crate. It still decodes XML entities and
/// handles CDATA sections.
pub fn parse_trending_rss(xml: &str) -> Result<Vec<TrendItem>, GoogleTrendsError> {
    let mut trends = Vec::new();
    for item in tag_blocks(xml, "item") {
        let title = tag_text(&item, "title").unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }
        let news = tag_blocks(&item, "ht:news_item")
            .into_iter()
            .filter_map(|block| {
                let title = tag_text(&block, "ht:news_item_title").unwrap_or_default();
                let url = tag_text(&block, "ht:news_item_url").unwrap_or_default();
                let source = tag_text(&block, "ht:news_item_source").unwrap_or_default();
                (!title.is_empty() || !url.is_empty() || !source.is_empty())
                    .then_some(TrendNewsItem { title, url, source })
            })
            .collect();
        trends.push(TrendItem {
            rank: trends.len() + 1,
            title,
            approx_traffic: tag_text(&item, "ht:approx_traffic").unwrap_or_default(),
            pub_date: tag_text(&item, "pubDate").unwrap_or_default(),
            link: tag_text(&item, "link").unwrap_or_default(),
            news,
        });
    }

    if trends.is_empty() {
        Err(GoogleTrendsError::MissingItems)
    } else {
        Ok(trends)
    }
}

/// Build the in-memory prompt suite from an RSS snapshot.
pub fn build_prompt_suite(
    xml: &str,
    config: &TrendPromptSuiteConfig,
) -> Result<TrendPromptSuite, GoogleTrendsError> {
    let mut trends = parse_trending_rss(xml)?;
    if trends.len() < config.top_n {
        return Err(GoogleTrendsError::NotEnoughTrends {
            requested: config.top_n,
            available: trends.len(),
        });
    }
    trends.truncate(config.top_n);
    for (index, trend) in trends.iter_mut().enumerate() {
        trend.rank = index + 1;
    }

    let mut prompt_cases =
        Vec::with_capacity(trends.len() * SUPPORTED_TREND_PROMPT_LANGUAGES.len());
    for trend in &trends {
        for language in SUPPORTED_TREND_PROMPT_LANGUAGES {
            let prompt = prompt_for_topic(language, &trend.title)
                .expect("SUPPORTED_TREND_PROMPT_LANGUAGES stays in sync with prompt templates");
            prompt_cases.push(TrendPromptCase {
                id: format!("issue_499_trend_{:02}_{language}", trend.rank),
                trend_rank: trend.rank,
                language: language.to_owned(),
                topic: trend.title.clone(),
                prompt,
                expected_use: "formal_ai_request".to_owned(),
                expected_intent: "current_topic_context".to_owned(),
            });
        }
    }

    Ok(TrendPromptSuite {
        geo: config.geo.clone(),
        captured_at: config.captured_at.clone(),
        source_url: config.source_url.clone(),
        source_snapshot: config.source_snapshot.clone(),
        minimum_prompt_count: prompt_cases.len(),
        trends,
        prompt_cases,
    })
}

/// Render a Google Trends RSS snapshot directly into the benchmark fixture.
pub fn render_prompt_suite_from_rss(
    xml: &str,
    config: &TrendPromptSuiteConfig,
) -> Result<String, GoogleTrendsError> {
    let suite = build_prompt_suite(xml, config)?;
    Ok(render_prompt_suite(&suite))
}

/// Render the prompt suite as Links Notation. Deterministic, with one trailing newline.
#[must_use]
pub fn render_prompt_suite(suite: &TrendPromptSuite) -> String {
    let mut out = String::new();
    out.push_str("google_trends_prompt_suite_issue_499\n");
    field(&mut out, 2, "record_type", "google_trends_prompt_suite");
    field(&mut out, 2, "id", "issue_499_google_trends_top10");
    field(&mut out, 2, "issue", "499");
    field(&mut out, 2, "title", "Google Trends top-ten prompt catalog");
    field(
        &mut out,
        2,
        "purpose",
        "Convert a captured Google Trends top-ten RSS snapshot into multilingual Formal AI request cases.",
    );
    field(&mut out, 2, "geo", &suite.geo);
    field(&mut out, 2, "captured_at", &suite.captured_at);
    field(&mut out, 2, "source_url", &suite.source_url);
    field(&mut out, 2, "source_snapshot", &suite.source_snapshot);
    number_field(&mut out, 2, "top_count", suite.trends.len());
    number_field(&mut out, 2, "prompt_count", suite.prompt_cases.len());
    number_field(
        &mut out,
        2,
        "minimum_prompt_count",
        suite.minimum_prompt_count,
    );
    field(
        &mut out,
        2,
        "runner",
        "cargo test --test unit issue_499_google_trends -- --nocapture",
    );
    field(
        &mut out,
        2,
        "ratchet_policy",
        "CI asserts that every captured top-ten topic has one generated prompt per supported language.",
    );
    for language in SUPPORTED_TREND_PROMPT_LANGUAGES {
        field(&mut out, 2, "supported_language", language);
    }

    for trend in &suite.trends {
        let _ = writeln!(out, "google_trends_topic_issue_499_{:02}", trend.rank);
        field(&mut out, 2, "record_type", "google_trends_topic");
        field(&mut out, 2, "suite", "issue_499_google_trends_top10");
        number_field(&mut out, 2, "rank", trend.rank);
        field(&mut out, 2, "title", &trend.title);
        field(&mut out, 2, "approx_traffic", &trend.approx_traffic);
        field(&mut out, 2, "pub_date", &trend.pub_date);
        field(&mut out, 2, "link", &trend.link);
        if let Some(news) = trend.news.first() {
            field(&mut out, 2, "first_news_source", &news.source);
            field(&mut out, 2, "first_news_title", &news.title);
            field(&mut out, 2, "first_news_url", &news.url);
        }
    }

    for case in &suite.prompt_cases {
        let _ = writeln!(out, "google_trends_prompt_case_{}", case.id);
        field(&mut out, 2, "record_type", "google_trends_prompt_case");
        field(&mut out, 2, "id", &case.id);
        field(&mut out, 2, "suite", "issue_499_google_trends_top10");
        number_field(&mut out, 2, "trend_rank", case.trend_rank);
        field(&mut out, 2, "language", &case.language);
        field(&mut out, 2, "topic", &case.topic);
        field(&mut out, 2, "prompt", &case.prompt);
        field(&mut out, 2, "expected_use", &case.expected_use);
        field(&mut out, 2, "expected_intent", &case.expected_intent);
    }

    format!("{}\n", out.trim_end())
}

/// The deterministic prompt template for one language/topic pair.
#[must_use]
pub fn prompt_for_topic(language: &str, topic: &str) -> Option<String> {
    match language {
        "en" => Some(format!("What is {topic}?")),
        "ru" => Some(format!("Что такое {topic}?")),
        "hi" => Some(format!("{topic} क्या है?")),
        "zh" => Some(format!("{topic} 是什么?")),
        _ => None,
    }
}

fn tag_text(xml: &str, tag: &str) -> Option<String> {
    tag_blocks(xml, tag)
        .into_iter()
        .next()
        .map(|text| decode_xml_text(strip_cdata(text.trim())).trim().to_owned())
}

fn tag_blocks(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut blocks = Vec::new();
    let mut offset = 0usize;

    while let Some(relative_start) = xml[offset..].find(&open) {
        let start = offset + relative_start;
        let after_name = xml[start + open.len()..].chars().next();
        if !matches!(after_name, Some('>' | ' ' | '\t' | '\n' | '\r')) {
            offset = start + open.len();
            continue;
        }
        let Some(open_end_relative) = xml[start..].find('>') else {
            break;
        };
        let body_start = start + open_end_relative + 1;
        let Some(close_relative) = xml[body_start..].find(&close) else {
            break;
        };
        let body_end = body_start + close_relative;
        blocks.push(xml[body_start..body_end].to_owned());
        offset = body_end + close.len();
    }

    blocks
}

fn strip_cdata(text: &str) -> &str {
    text.strip_prefix("<![CDATA[")
        .and_then(|value| value.strip_suffix("]]>"))
        .unwrap_or(text)
}

fn decode_xml_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '&' {
            out.push(ch);
            continue;
        }

        let mut entity = String::new();
        while let Some(&next) = chars.peek() {
            chars.next();
            if next == ';' {
                break;
            }
            entity.push(next);
        }
        if entity.is_empty() {
            out.push('&');
            continue;
        }
        if let Some(decoded) = decode_entity(&entity) {
            out.push(decoded);
        } else {
            out.push('&');
            out.push_str(&entity);
            out.push(';');
        }
    }
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => decode_numeric_entity(entity),
    }
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
        .or_else(|| {
            entity
                .strip_prefix('#')
                .and_then(|decimal| decimal.parse::<u32>().ok())
        })
        .and_then(char::from_u32)
}

fn field(out: &mut String, indent: usize, name: &str, value: &str) {
    let _ = writeln!(
        out,
        "{:indent$}{name} \"{}\"",
        "",
        escape_lino_value(value),
        indent = indent,
    );
}

fn number_field(out: &mut String, indent: usize, name: &str, value: usize) {
    let _ = writeln!(out, "{:indent$}{name} \"{value}\"", "", indent = indent);
}

fn escape_lino_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}
