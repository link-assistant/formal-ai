//! Offline, deterministic web corpus for the in-repo agentic driver (issue #468).
//!
//! The maintainer asks our Formal AI to *"do web fetch and web search … to
//! actually complete the task"*. CI has no network, and the repository's
//! determinism stance forbids live calls anyway, so the driver resolves
//! `web_search` / `web_fetch` tool calls against this small fixed corpus instead
//! of the open web. It is a faithful stand-in: a search that surfaces the
//! canonical Викитека page for «Сказка о рыбаке и рыбке», and a fetch of that page
//! that returns the canonical synopsis — the very text the formalizer falls back
//! to (see [`CANONICAL_FISHERMAN_SYNOPSIS`]), so the loop produces a stable
//! knowledge base whether the fetch "succeeds" or not.

use std::fmt::Write as _;

use super::formalize::CANONICAL_FISHERMAN_SYNOPSIS;
use super::meaning_detail::{self, CONCEPTS};
use super::planner::CANONICAL_SOURCE_URL;

/// One page in the offline corpus. Fields are owned so pages can be built from the
/// [`CONCEPTS`] registry at call time (one Wikidata-lexeme page per concept).
struct CorpusPage {
    url: String,
    title: String,
    /// Lowercased keywords a search query is matched against.
    keywords: Vec<String>,
    body: String,
}

/// The corpus pages, in search-ranking order: the fisherman tale (issue #468)
/// first, then one Wikidata-lexeme page per registered concept (issue #538). Each
/// concept contributes its own page automatically, so registering a new concept in
/// [`CONCEPTS`] makes it web-searchable and web-fetchable with no change here.
fn pages() -> Vec<CorpusPage> {
    let mut pages = vec![CorpusPage {
        url: CANONICAL_SOURCE_URL.to_owned(),
        title: "Сказка о рыбаке и рыбке — Александр Пушкин (Викитека)".to_owned(),
        keywords: [
            "рыбак",
            "рыбке",
            "рыбка",
            "пушкин",
            "сказка",
            "fisherman",
            "fish",
            "pushkin",
        ]
        .iter()
        .map(|keyword| (*keyword).to_owned())
        .collect(),
        body: CANONICAL_FISHERMAN_SYNOPSIS.to_owned(),
    }];
    for concept in CONCEPTS {
        // The concept's own routing keywords plus the shared lexeme vocabulary, so
        // both a concept-specific query and a generic "wikidata lexeme …" query hit.
        let mut keywords: Vec<String> = concept.keywords.iter().map(|k| k.to_lowercase()).collect();
        keywords.extend(
            ["wikidata", "lexeme", "grammatical", "singular", "plural"]
                .iter()
                .map(|k| (*k).to_owned()),
        );
        pages.push(CorpusPage {
            url: concept.source_url.to_owned(),
            title: format!(
                "Wikidata lexemes for the {} concept ({}) — Wikidata",
                concept.name, concept.grounded_in
            ),
            keywords,
            body: meaning_detail::source_bundle(concept),
        });
    }
    pages
}

/// Resolve a `web_search` query into deterministic results text.
///
/// A query that matches a known page lists it (rank, title, url, snippet); an
/// unmatched query returns a "no results" line. Search results are never treated
/// as the source text — only [`web_fetch`] bodies are — so the wording here is
/// purely informational for the agent.
#[must_use]
pub fn web_search(query: &str) -> String {
    let lower = query.to_lowercase();
    let all = pages();
    let hits: Vec<&CorpusPage> = all
        .iter()
        .filter(|page| {
            page.keywords
                .iter()
                .any(|keyword| lower.contains(keyword.as_str()))
        })
        .collect();
    if hits.is_empty() {
        return format!("web_search: no results for {query:?}");
    }
    let mut out = String::new();
    for (rank, page) in hits.iter().enumerate() {
        let snippet: String = page.body.chars().take(80).collect();
        let _ = writeln!(
            out,
            "{}. {}\n   {}\n   {}…",
            rank + 1,
            page.title,
            page.url,
            snippet.trim()
        );
    }
    out.trim_end().to_owned()
}

/// Resolve a `web_fetch` url into deterministic page text.
///
/// An unknown url yields an error string the planner recognises via its error
/// heuristic and ignores (falling back to the canonical synopsis), exactly as a
/// real 404 would behave — which is how the driver exercises the *"understand
/// errors from tools"* requirement.
#[must_use]
pub fn web_fetch(url: &str) -> String {
    pages().iter().find(|page| page.url == url).map_or_else(
        || format!("web_fetch error: 404 not found for {url}"),
        |page| page.body.clone(),
    )
}
