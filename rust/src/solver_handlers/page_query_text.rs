//! Page queries over a page the prompt supplies (issue #1163 R10).
//!
//! A prompt whose first line is one of the memory query language's page
//! queries (the `page_query` templates of
//! `data/seed/page-formalization-rules.lino`, for example "the command in the
//! paragraph that mentions compile") and whose remaining lines are a page
//! (HTML, Markdown or plain text, sniffed by the seed's rules) is answered by
//! formalizing that page with the generic page formalizer and running the
//! query against it. The answer is the text of every block the query's links
//! point at, so it is what the page itself says -- nothing is fetched and
//! nothing is composed. A prompt that carries no page, or a query that no
//! block answers, is declined so the handlers below keep it.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

/// The intent a supplied-page query answers under.
const PAGE_QUERY_INTENT: &str = "page_query";
/// The response link of that intent.
const PAGE_QUERY_RESPONSE: &str = "response:page_query";

/// Answer a page query over a page the prompt supplies, or decline.
#[must_use]
pub fn handle_page_query_text(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (query, page_text) = crate::web_formalize::split_supplied_page(prompt)?;
    let parsed = crate::memory_query_language::parse_page_query(&query)?;
    let page = crate::web_formalize::FormalizedPage::from_supplied_text(&page_text, &query);
    let blocks = page.code_blocks().len() + page.paragraphs().len();
    let mut store = crate::web_formalize::FormalizedPageStore::new();
    let key = store.insert(page);
    let links = parsed.execute(&store);
    let stored = store.get(&key)?;
    let texts: Vec<&str> = links
        .iter()
        .filter_map(|link| stored.block_text(link))
        .collect();
    if texts.is_empty() {
        return None;
    }
    log.append("page_query", query.clone());
    log.append("page_query_page", format!("{key} blocks={blocks}"));
    for text in &texts {
        log.append("page_query_answer", (*text).to_owned());
    }
    Some(finalize_simple(
        prompt,
        log,
        PAGE_QUERY_INTENT,
        PAGE_QUERY_RESPONSE,
        &texts.join("\n"),
        0.85,
    ))
}
