//! Multi-step web research for agentic clients (issue #687).
//!
//! Intent recognition is delegated to the same meaning-lexicon detector used by
//! the universal solver. This module only adds agentic sequencing (search, rank
//! a source, fetch, answer) and resolves a seed-declared contextual reference
//! against prior user turns. No natural-language vocabulary lives here.

use serde_json::json;

use super::planner::{
    fetch_arguments, plan_one, tool_for, AgenticPlan, Capability, PlannedToolCall, Progress,
};
use crate::engine::FormalAiEngine;
use crate::protocol::ChatMessage;
use crate::seed::{self, Slot};

pub(super) fn web_research_query_for(messages: &[ChatMessage]) -> Option<String> {
    let task = latest_user_text(messages)?;
    if let Some(query) = seed_research_subject(&task)
        .or_else(|| crate::solver_handlers::detect_web_search_query(&task))
    {
        return if is_context_reference(&query) {
            topic_from_history(messages)
        } else {
            Some(query)
        };
    }

    // A punctuation-marked question that the local engine cannot answer is an
    // open-world lookup. Question words themselves remain seed data in the
    // shared detector; punctuation is script structure, not language vocabulary.
    let trimmed = task.trim();
    let asks_question = trimmed.ends_with(['?', '？', '؟', '¿']);
    let unresolved = matches!(
        FormalAiEngine.answer(&task).intent.as_str(),
        "unknown" | "web_search"
    );
    (asks_question && unresolved).then(|| trim_question_punctuation(&task))
}

/// Extract the subject carried by a seed-declared research imperative. The
/// shared web detector deliberately rejects pronouns as standalone searches;
/// the agentic planner accepts them here because it can resolve them against
/// conversation history before creating a tool call.
fn seed_research_subject(task: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(task);
    seed::lexicon()
        .role_word_forms(seed::ROLE_WEB_SEARCH_IMPERATIVE_LEAD)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .find_map(|form| {
            let prefix = crate::engine::normalize_prompt(form.before_slot());
            normalized
                .strip_prefix(&prefix)
                .map(trim_question_punctuation)
                .filter(|subject| !subject.is_empty())
        })
}

pub(super) fn plan_web_research_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    query: &str,
) -> Option<AgenticPlan> {
    let progress = Progress::scan(messages);
    if progress.done(Capability::Fetch) {
        return Some(AgenticPlan::Final(final_answer(query, &progress)));
    }
    if progress.done(Capability::Search) {
        if let Some(tool) = tool_for(tool_names, Capability::Fetch) {
            if let Some(output) = progress.search_output.as_deref() {
                let calls = research_urls(output)
                    .into_iter()
                    .map(|url| PlannedToolCall {
                        tool: tool.to_owned(),
                        arguments: fetch_arguments(&url),
                    })
                    .collect::<Vec<_>>();
                if !calls.is_empty() {
                    return Some(AgenticPlan::ToolCalls(calls));
                }
            }
        }
        return Some(AgenticPlan::Final(final_answer(query, &progress)));
    }
    tool_for(tool_names, Capability::Search)
        .map(|tool| plan_one(tool, json!({ "query": query }).to_string()))
}

/// Evidence at or below this many characters is already an answer, so it is
/// reported verbatim. Above it the fetch result is a whole web page — site
/// chrome, navigation and unrelated articles around the part that answers the
/// question — and must be extracted from rather than dumped (issue #771).
const VERBATIM_EVIDENCE_LIMIT: usize = 600;

/// How many sentences an extract keeps. Enough for a claim plus its immediate
/// qualification, short enough to stay an answer rather than a transcript.
const EXTRACT_SENTENCES: usize = 3;

fn final_answer(query: &str, progress: &Progress) -> String {
    if !progress.fetched_pages.is_empty() {
        return progress
            .fetched_pages
            .iter()
            .map(|(url, evidence)| {
                format!(
                    "{}\n\n{}: {url}",
                    extract_answer(query, evidence.trim()),
                    seed_text("web_research_source_label")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
    }
    let evidence = progress
        .fetched_text
        .as_deref()
        .or(progress.search_output.as_deref())
        .unwrap_or_default()
        .trim();
    if evidence.is_empty() {
        return render_seed_text("web_research_no_content", "query", query);
    }
    let source = progress
        .search_output
        .as_deref()
        .and_then(preferred_url)
        .map_or_else(String::new, |url| {
            format!("\n\n{}: {url}", seed_text("web_research_source_label"))
        });
    format!("{}{source}", extract_answer(query, evidence))
}

/// Reduce fetched evidence to the sentences that actually bear on `query`.
///
/// A fetch tool returns the whole page; answering with it verbatim is what made
/// issue #771's session unreadable. Sentences are scored by symbolic token
/// overlap with the query — the same non-neural similarity the ranker uses — and
/// the best few are returned in document order so the extract still reads as
/// prose. Scoring is deterministic and carries no natural-language vocabulary,
/// so it works in every supported language — see [`relevance`] for how the
/// space-less scripts are handled.
fn extract_answer(query: &str, evidence: &str) -> String {
    if evidence.chars().count() <= VERBATIM_EVIDENCE_LIMIT {
        return evidence.to_owned();
    }
    let sentences = crate::summarization::formalize(evidence);
    let mut scored: Vec<(usize, f32, &str)> = sentences
        .iter()
        .enumerate()
        .map(|(position, statement)| {
            (
                position,
                relevance(query, &statement.text),
                statement.text.as_str(),
            )
        })
        .filter(|(_, score, _)| *score > 0.0)
        .collect();
    if scored.is_empty() {
        // Nothing overlaps the query: fall back to the head of the document
        // rather than the whole of it, so the answer stays bounded either way.
        return truncate_chars(evidence, VERBATIM_EVIDENCE_LIMIT);
    }
    // Rank by relevance, keep the best few, then restore document order.
    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    scored.truncate(EXTRACT_SENTENCES);
    scored.sort_by_key(|(position, _, _)| *position);
    scored
        .iter()
        .map(|(_, _, text)| *text)
        .collect::<Vec<_>>()
        .join(" ")
}

/// How much `sentence` bears on `query`, in `0.0..=1.0`.
///
/// Bag-of-words cosine is the primary measure. It tokenizes on non-alphanumeric
/// boundaries, which is exactly right for the space-separated scripts but scores
/// every Chinese sentence 0.0: a run of Han characters with no spaces is a
/// single token, so query and sentence never share one. The codebase's existing
/// answer for that (see `coding::catalog::contains_cjk` and its callers) is to
/// match on characters rather than words, so that is the fallback here.
fn relevance(query: &str, sentence: &str) -> f32 {
    let cosine = crate::probability::symbolic_cosine_similarity(query, sentence);
    if cosine > 0.0 || !crate::coding::contains_cjk(query) {
        return cosine;
    }
    character_overlap(query, sentence)
}

/// The fraction of the query's distinct ideographs that `sentence` also uses.
///
/// Punctuation and spacing are ignored, so the score reflects shared content
/// characters only. Common function characters inflate it slightly, which costs
/// nothing here because the score is only ever used to rank sentences of the
/// same document against each other.
fn character_overlap(query: &str, sentence: &str) -> f32 {
    let sentence: std::collections::BTreeSet<char> =
        sentence.chars().filter(|c| c.is_alphanumeric()).collect();
    let query: std::collections::BTreeSet<char> =
        query.chars().filter(|c| c.is_alphanumeric()).collect();
    if query.is_empty() {
        return 0.0;
    }
    let shared = query.iter().filter(|c| sentence.contains(c)).count();
    #[expect(
        clippy::cast_precision_loss,
        reason = "character counts are far below f32's exact-integer range"
    )]
    {
        shared as f32 / query.len() as f32
    }
}

/// Truncate to at most `max` characters on a char boundary, appending an
/// ellipsis when shortened.
fn truncate_chars(value: &str, max: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let head: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", head.trim_end())
}

fn seed_text(key: &str) -> String {
    seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}

fn render_seed_text(key: &str, name: &str, value: &str) -> String {
    let mut placeholder = String::with_capacity(name.len() + 2);
    placeholder.push('{');
    placeholder.push_str(name);
    placeholder.push('}');
    seed_text(key).replace(&placeholder, value)
}

/// Rank URLs instead of blindly fetching the first search result. Government
/// and education hosts are authoritative for public facts; otherwise preserve
/// the search provider's ordering. The complete fetched URL is retained in the
/// final answer for auditability.
pub(super) fn preferred_url(text: &str) -> Option<String> {
    research_urls(text).into_iter().next()
}

/// Bound the breadth of one research round while retaining independent sources.
/// The first authoritative host is moved to the front; the search provider's
/// ranking determines the remaining order. Three captures are enough to
/// triangulate a claim without turning a single question into an unbounded crawl.
const MAX_RESEARCH_SOURCES: usize = 3;

fn research_urls(text: &str) -> Vec<String> {
    let mut urls = urls_in(text);
    let mut seen = std::collections::BTreeSet::new();
    urls.retain(|url| seen.insert(url.clone()));
    if let Some(position) = urls.iter().position(|url| authoritative_host(url)) {
        urls.swap(0, position);
    }
    urls.truncate(MAX_RESEARCH_SOURCES);
    urls
}

fn urls_in(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|token| token.starts_with("http://") || token.starts_with("https://"))
        .map(|token| {
            token
                .trim_end_matches(['.', ',', ';', ')', ']', '"', '\''])
                .to_owned()
        })
        .collect()
}

fn authoritative_host(url: &str) -> bool {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut labels = host.rsplit('.');
    let terminal = labels.next().unwrap_or_default();
    terminal == "gov"
        || terminal == "edu"
        || (terminal == "uk" && labels.next().is_some_and(|label| label == "gov"))
}

fn is_context_reference(query: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(query);
    seed::lexicon()
        .role_word_forms(seed::ROLE_NON_REFERENTIAL_SUBJECT)
        .into_iter()
        .any(|form| match form.slot() {
            Slot::Bare => crate::engine::normalize_prompt(&form.text) == normalized,
            Slot::Prefix => {
                normalized.starts_with(&crate::engine::normalize_prompt(form.before_slot()))
            }
            Slot::Suffix | Slot::Circumfix => false,
        })
}

fn topic_from_history(messages: &[ChatMessage]) -> Option<String> {
    let latest = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    messages[..latest]
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            if !message.role.eq_ignore_ascii_case("user") {
                return None;
            }
            let text = message.content.plain_text();
            if super::report_issue::is_report_intent(&text)
                || is_conversation_meta_request(&text, &messages[..index])
            {
                return None;
            }
            let topic = crate::solver_handlers::detect_web_search_query(&text)
                .unwrap_or_else(|| trim_question_punctuation(&text));
            (!topic.trim().is_empty() && !is_context_reference(&topic)).then_some(topic)
        })
}

fn is_conversation_meta_request(prompt: &str, preceding: &[ChatMessage]) -> bool {
    let history = preceding
        .iter()
        .filter_map(|message| {
            let text = message.content.plain_text();
            if text.trim().is_empty() {
                None
            } else if message.role.eq_ignore_ascii_case("user") {
                Some(crate::ConversationTurn::user(text))
            } else if message.role.eq_ignore_ascii_case("assistant") {
                Some(crate::ConversationTurn::assistant(text))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    crate::solve_with_history(prompt, &history).intent == "summarize_conversation"
}

fn trim_question_punctuation(text: &str) -> String {
    text.trim()
        .trim_end_matches(['?', '？', '؟', '¿', '.', '!', '。'])
        .trim()
        .to_owned()
}

fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.user_request_text())
}
