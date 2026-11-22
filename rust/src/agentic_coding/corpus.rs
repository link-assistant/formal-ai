//! Discover registered publication links and acquire CLI source evidence.

use std::fmt::Write as _;

use super::formalization_recipe::publication_url;
use super::lexicon::Lexicon;
use super::meaning_detail;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};

struct SourcePage {
    url: String,
    title: String,
    search_terms: String,
}

fn pages() -> Vec<SourcePage> {
    let lexicon = Lexicon::standard();
    let mut pages: Vec<_> = lexicon
        .works()
        .iter()
        .filter_map(|work| {
            Some(SourcePage {
                url: publication_url(work)?,
                title: work.publisher_title.clone()?,
                search_terms: work.publisher_title.as_ref()?.to_lowercase(),
            })
        })
        .collect();
    pages.extend(meaning_detail::concepts().iter().map(|concept| SourcePage {
        url: concept.source_url.clone(),
        title: crate::seed::agentic_message(
            "meaning-detail-source-title",
            &[("name", &concept.name), ("entity", &concept.grounded_in)],
        ),
        search_terms: concept.search_query.to_lowercase(),
    }));
    pages
}

/// List declared publisher endpoints that match the search terms.
#[must_use]
pub fn web_search(query: &str) -> String {
    let lower = query.to_lowercase();
    let pages = pages();
    let lexicon = Lexicon::standard();
    let title_match = lexicon.best_work_for(query).and_then(publication_url);
    let mut hits: Vec<_> = pages
        .iter()
        .filter_map(|page| {
            let count = lower
                .split_whitespace()
                .filter(|word| crate::seed::surface_present(&page.search_terms, word))
                .count();
            (count > 0 || title_match.as_deref() == Some(page.url.as_str()))
                .then_some((count, page))
        })
        .collect();
    hits.sort_by_key(|(count, _)| std::cmp::Reverse(*count));
    if hits.is_empty() {
        return format!("web_search: no results for {query:?}");
    }
    let mut out = String::new();
    for (rank, (_, page)) in hits.iter().enumerate() {
        let _ = writeln!(out, "{}. {}\n   {}", rank + 1, page.title, page.url);
    }
    out.trim_end().to_owned()
}

/// Fetch a registered publication, or return the tool's error text.
#[must_use]
pub fn web_fetch(url: &str) -> String {
    web_fetch_result(url).unwrap_or_else(|detail| detail)
}

/// Fetch a registered publication using the caller's network policy.
/// # Errors
/// Reports unknown sources, denied or missing captures, and invalid UTF-8.
pub fn web_fetch_result(url: &str) -> Result<String, String> {
    if pages().iter().any(|page| page.url == url) {
        return fetch_source_bytes(url)
            .and_then(|bytes| String::from_utf8(bytes).map_err(|error| error.to_string()));
    }
    Err(missing_source(url))
}

fn missing_source(url: &str) -> String {
    format!("web_fetch error: 404 not found for {url}")
}

fn fetch_source_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        CurlSourceTransport,
    )
    .with_online(crate::solver_config::read_network_allowed());
    client
        .fetch(url)
        .map(|capture| capture.bytes().to_vec())
        .map_err(|error| error.to_string())
}

/// Read a registered publication or a public work item explicitly named by URL.
/// # Errors
/// Reports denied or missing captures, retrieval failures, and invalid bodies.
pub fn fetch_cli_source(url: &str) -> Result<String, String> {
    let Some(path) = url.strip_prefix("https://github.com/") else {
        return web_fetch_result(url);
    };
    let missing = || missing_source(url);
    let fields: Vec<_> = path.split('/').collect();
    let [owner, repo, "issues" | "pull", number] = fields.as_slice() else {
        return Err(missing());
    };
    if [owner, repo].iter().any(|name| {
        matches!(**name, "" | "." | "..")
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._".contains(&byte))
    }) || !number.bytes().all(|byte| byte.is_ascii_digit())
        || number.parse::<u64>().ok().is_none_or(|number| number == 0)
    {
        return Err(missing());
    }

    let bytes = fetch_source_bytes(&format!(
        "https://api.github.com/repos/{owner}/{repo}/issues/{number}"
    ))?;
    let item: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    item.get("body")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(missing)
}
