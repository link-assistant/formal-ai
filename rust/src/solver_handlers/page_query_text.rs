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
//!
//! A query line that evidences the seed meaning
//! `code_example_decomposition_request` asks instead for the parts of the
//! page's code examples: each code block in a registered grammar goes through
//! the code-example decomposer (issue #1164 R1164-11).

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

/// The seed meaning a query line evidences when it asks for the parts of the
/// supplied page's code examples (issue #1164 R1164-11).
const CODE_EXAMPLE_QUERY_MEANING: &str = "code_example_decomposition_request";
/// The intent a code-example decomposition answers under.
const CODE_EXAMPLE_INTENT: &str = "code_example_decomposition";
/// The response link of that intent.
const CODE_EXAMPLE_RESPONSE: &str = "response:code_example_decomposition";
/// The intent a supplied-page query answers under.
const PAGE_QUERY_INTENT: &str = "page_query";
/// The response link of that intent.
const PAGE_QUERY_RESPONSE: &str = "response:page_query";

/// Whether the prompt is a page query over a page it supplies.
///
/// The capability table reads the file names such a page mentions as a batch
/// read; the page the prompt carries is the source instead, so the
/// capability-gap refusal yields to this handler.
#[must_use]
pub fn page_query_text_claims(prompt: &str) -> bool {
    crate::web_formalize::split_supplied_page(prompt).is_some_and(|(query, _)| {
        crate::memory_query_language::parse_page_query(&query).is_some()
            || asks_for_code_example_parts(&query)
    })
}

/// Answer a page query over a page the prompt supplies, or decline.
#[must_use]
pub fn handle_page_query_text(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (query, page_text) = crate::web_formalize::split_supplied_page(prompt)?;
    if asks_for_code_example_parts(&query) {
        return code_example_page_answer(prompt, &query, &page_text, log);
    }
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
    log.append("page_query", query);
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

/// Whether the query line asks for the parts of the page's code examples.
fn asks_for_code_example_parts(query: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(query);
    crate::seed::lexicon()
        .meaning(CODE_EXAMPLE_QUERY_MEANING)
        .is_some_and(|meaning| meaning.evidenced_in(&normalized))
}

/// The supplied page's code examples, decomposed (issue #1164 R1164-11).
///
/// Every code block in a language with a registered grammar goes through
/// [`crate::code_example_knowledge::decompose_code_node`], and the answer
/// lists each part as its kind and source text, one per line. A block in an
/// unregistered language is skipped, never guessed; a page none of whose
/// blocks decomposes is declined. The browser twin is
/// `tryCodeExamplePageQuery` in `js/worker/formal_ai_worker_code_examples.js`.
fn code_example_page_answer(
    prompt: &str,
    query: &str,
    page_text: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let page = crate::web_formalize::FormalizedPage::from_supplied_text(page_text, query);
    let mut lines = Vec::new();
    for block in page.code_blocks() {
        let Ok(node) =
            crate::code_example_knowledge::decompose_code_node(&block.text, &block.language, &[])
        else {
            continue;
        };
        log.append(
            "code_example_decomposition",
            format!("{} parts={}", node.language_slug, node.parts.len()),
        );
        for part in &node.parts {
            let line = format!("{} {}", part.kind.as_str(), part.source_text);
            log.append(
                "code_example_part",
                format!("{} {line}", node.language_slug),
            );
            lines.push(line);
        }
    }
    if lines.is_empty() {
        return None;
    }
    Some(finalize_simple(
        prompt,
        log,
        CODE_EXAMPLE_INTENT,
        CODE_EXAMPLE_RESPONSE,
        &lines.join("\n"),
        0.85,
    ))
}
