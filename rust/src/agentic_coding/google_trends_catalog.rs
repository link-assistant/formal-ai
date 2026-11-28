//! Agentic recipe for issue #498: turn Google Trends into multilingual test prompts.
//!
//! The recipe renders requests and answers from the regional publisher feed.

use std::fmt::Write as _;

use crate::google_trends_catalog::{
    GoogleTrendsCatalog, answer_google_trends_catalog, feed_for_task,
};
use crate::seed::{agentic_message, parser::parse_lino};

/// The workspace path the planner writes.
pub const GOOGLE_TRENDS_CATALOG_PATH: &str = "google-trends-catalog.lino";

/// A differently worded task for the Google Trends catalog recipe.
pub const GOOGLE_TRENDS_CATALOG_TASK: &str = "Convert the Google Trends top 10 searches into multilingual Formal AI test prompts, \
     include two request variations in every supported language, answer each request, \
     and record the Google Trends catalog in Links Notation.";

const GOOGLE_TRENDS_KEYWORDS: [&str; 4] = [
    "google trends",
    "trending searches",
    "top searches",
    "trends catalog",
];

/// Whether `prompt` asks for the Google Trends catalog recipe.
#[must_use]
pub fn is_google_trends_catalog_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    GOOGLE_TRENDS_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        && (lower.contains("prompt")
            || lower.contains("answer")
            || lower.contains("catalog")
            || lower.contains("test"))
}

/// Render the requested regional catalog from the publisher capture.
pub(super) fn document_for_task(task: &str) -> Option<String> {
    let catalog = answer_google_trends_catalog(feed_for_task(task)?);
    Some(render_catalog(&catalog))
}

/// Render the default regional report, or an empty document if the feed is unavailable.
#[must_use]
pub fn render_document() -> String {
    document_for_task("").unwrap_or_default()
}

/// The self-contained final answer for the agentic loop.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let parsed = parse_lino(document);
    let Some(catalog) = parsed.children.first() else {
        return agentic_message("google_trends_source_unavailable", &[]);
    };
    agentic_message(
        "google_trends_catalog_final_answer",
        &[
            ("topic_count", catalog.find_child_value("topic_count")),
            ("geo", catalog.find_child_value("geo")),
            ("prompt_count", catalog.find_child_value("prompt_count")),
            ("answer_count", catalog.find_child_value("answered_count")),
            ("source_url", catalog.find_child_value("source_url")),
            ("path", GOOGLE_TRENDS_CATALOG_PATH),
            ("document", document.trim_end()),
        ],
    )
}

/// Shell command used by the agentic recipe to verify the written catalog exists.
#[must_use]
pub fn verification_command() -> String {
    let program = format!(
        "p='{GOOGLE_TRENDS_CATALOG_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:12]))"
    );
    format!(
        "python3 -c {}",
        super::general_planner::shell_quote(&program)
    )
}

/// Render answers together with their source identity.
pub fn render_catalog(catalog: &GoogleTrendsCatalog) -> String {
    let prompt_count: usize = catalog.topics.iter().map(|topic| topic.prompts.len()).sum();
    let answer_count: usize = catalog
        .topics
        .iter()
        .map(|topic| topic.answered.len())
        .sum();

    let mut out = String::from("google_trends_catalog\n");
    out.push_str("  record_type \"google_trends_catalog\"\n");
    field(&mut out, 2, "source", &catalog.source);
    field(&mut out, 2, "source_url", &catalog.source_url);
    field(&mut out, 2, "geo", &catalog.geo);
    field(&mut out, 2, "locale", &catalog.locale);
    field(&mut out, 2, "collected_at", &catalog.collected_at);
    if let Some(digest) = &catalog.source_sha256 {
        field(&mut out, 2, "source_sha256", digest);
    }
    let _ = writeln!(out, "  topic_count \"{}\"", catalog.topics.len());
    let _ = writeln!(out, "  prompt_count \"{prompt_count}\"");
    let _ = writeln!(out, "  answered_count \"{answer_count}\"");

    for topic in &catalog.topics {
        out.push_str("  topic\n");
        let _ = writeln!(out, "    rank \"{}\"", topic.rank);
        field(&mut out, 4, "query", &topic.query);
        if let Some(traffic) = &topic.approx_traffic {
            field(&mut out, 4, "approx_traffic", traffic);
        }
        if let Some(pub_date) = &topic.pub_date {
            field(&mut out, 4, "pub_date", pub_date);
        }
        if let Some(news) = topic.news_items.first() {
            out.push_str("    source_news\n");
            field(&mut out, 6, "title", &news.title);
            field(&mut out, 6, "source", &news.source);
            field(&mut out, 6, "url", &news.url);
        }
        for prompt in &topic.prompts {
            out.push_str("    prompt\n");
            field(&mut out, 6, "prompt_language", &prompt.language);
            field(&mut out, 6, "variation", &prompt.variation_key);
            field(&mut out, 6, "text", &prompt.prompt);
        }
        for answered in &topic.answered {
            out.push_str("    answered\n");
            field(&mut out, 6, "prompt_language", &answered.language);
            field(&mut out, 6, "variation", &answered.variation_key);
            field(&mut out, 6, "prompt", &answered.prompt);
            field(&mut out, 6, "intent", &answered.intent);
            let _ = writeln!(
                out,
                "      confidence \"{:.3}\"",
                round_confidence(answered.confidence)
            );
            field(&mut out, 6, "answer", &answered.answer);
        }
    }

    format!("{}\n", out.trim_end())
}

fn field(out: &mut String, indent: usize, name: &str, value: &str) {
    let spaces = " ".repeat(indent);
    let _ = writeln!(out, "{spaces}{name} \"{}\"", escape_value(value));
}

fn escape_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\\', "\\\\")
        .replace('"', "'")
}

fn round_confidence(confidence: f32) -> f32 {
    (confidence * 1000.0).round() / 1000.0
}
