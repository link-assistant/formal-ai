//! Multi-step web research for agentic clients (issue #687).
//!
//! Intent recognition is delegated to the same meaning-lexicon detector used by
//! the universal solver. This module only adds agentic sequencing (search, rank
//! a source, fetch, answer) and resolves a seed-declared contextual reference
//! against prior user turns. No natural-language vocabulary lives here.

use serde_json::json;

use super::planner::{fetch_arguments, plan_one, tool_for, AgenticPlan, Capability, Progress};
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
            if let Some(url) = progress.search_output.as_deref().and_then(preferred_url) {
                return Some(plan_one(tool, fetch_arguments(&url)));
            }
        }
        return Some(AgenticPlan::Final(final_answer(query, &progress)));
    }
    tool_for(tool_names, Capability::Search)
        .map(|tool| plan_one(tool, json!({ "query": query }).to_string()))
}

fn final_answer(query: &str, progress: &Progress) -> String {
    let evidence = progress
        .fetched_text
        .as_deref()
        .or(progress.search_output.as_deref())
        .unwrap_or_default()
        .trim();
    if evidence.is_empty() {
        return seed_text("web_research_no_content").replace("{query}", query);
    }
    let source = progress
        .search_output
        .as_deref()
        .and_then(preferred_url)
        .map_or_else(String::new, |url| {
            format!("\n\n{}: {url}", seed_text("web_research_source_label"))
        });
    format!("{evidence}{source}")
}

fn seed_text(key: &str) -> String {
    seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}

/// Rank URLs instead of blindly fetching the first search result. Government
/// and education hosts are authoritative for public facts; otherwise preserve
/// the search provider's ordering. The complete fetched URL is retained in the
/// final answer for auditability.
pub(super) fn preferred_url(text: &str) -> Option<String> {
    let urls = urls_in(text);
    urls.iter()
        .find(|url| authoritative_host(url))
        .cloned()
        .or_else(|| urls.into_iter().next())
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
    host.ends_with(".gov") || host.ends_with(".gov.uk") || host.ends_with(".edu")
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
        .map(|message| message.content.plain_text())
}
