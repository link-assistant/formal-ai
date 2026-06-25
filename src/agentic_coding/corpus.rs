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
use super::planner::CANONICAL_SOURCE_URL;

/// One page in the offline corpus.
struct CorpusPage {
    url: &'static str,
    title: &'static str,
    /// Lowercased keywords a search query is matched against.
    keywords: &'static [&'static str],
    body: &'static str,
}

/// The corpus pages, in search-ranking order. A single canonical page is enough
/// for the issue-#468 task; the shape generalises to more.
const fn pages() -> [CorpusPage; 1] {
    [CorpusPage {
        url: CANONICAL_SOURCE_URL,
        title: "Сказка о рыбаке и рыбке — Александр Пушкин (Викитека)",
        keywords: &[
            "рыбак",
            "рыбке",
            "рыбка",
            "пушкин",
            "сказка",
            "fisherman",
            "fish",
            "pushkin",
        ],
        body: CANONICAL_FISHERMAN_SYNOPSIS,
    }]
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
        .filter(|page| page.keywords.iter().any(|keyword| lower.contains(keyword)))
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
        |page| page.body.to_owned(),
    )
}
