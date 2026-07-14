//! Route factual and research questions to the client's web-search tool
//! (issue #687).
//!
//! Several reported `OpenCode` prompts — "When next elections in the USA?",
//! "What is the current population of Japan?", "Learn about it." — are questions
//! the deterministic engine cannot answer from its local knowledge base, so in
//! non-agentic mode they fall to the unknown-reasoning blurb. In agentic mode the
//! right move is to *act*: emit the client's web-search tool call, then fetch the
//! surfaced source and answer from it. Formal AI itself has no HTTP client, so the
//! harness (`OpenCode`, `@link-assistant/agent`, …) performs the actual network I/O.
//!
//! The decision of *whether* a prompt warrants web research is a pure
//! generalization, not a per-phrase list:
//!
//! 1. an explicit research imperative ("learn about …", "research …", "find out
//!    …"), with deixis ("it"/"this") resolved from the conversation;
//! 2. the seed-backed web-search recogniser the symbolic solver already uses
//!    ([`crate::solver_handlers::detect_web_search_query`]); or
//! 3. any answer-seeking question the deterministic engine cannot resolve locally
//!    — established by actually asking the engine and checking its intent.

use serde_json::json;

use super::planner::{fetch_arguments, plan_one, tool_for, AgenticPlan, Capability, Progress};
use crate::engine::FormalAiEngine;
use crate::protocol::ChatMessage;

/// The web-search query for a factual/research request, or [`None`] when the
/// latest user turn does not warrant web research.
pub(super) fn web_research_query_for(messages: &[ChatMessage]) -> Option<String> {
    let task = latest_user_text(messages)?;
    let lower = task.to_lowercase();

    // 1. An explicit research imperative: resolve deixis from history, then search.
    if is_research_imperative(&lower) {
        if let Some(topic) = research_topic(&task, messages) {
            return Some(topic);
        }
    }

    // 2. Reuse the seed-backed recogniser the symbolic solver uses, so the agentic
    //    path and the plain path classify web-search requests identically.
    if let Some(query) = crate::solver_handlers::detect_web_search_query(&task) {
        return Some(query);
    }

    // 3. An answer-seeking question the engine cannot resolve locally belongs on
    //    the web. Ask the deterministic engine; if it would fall to the unknown or
    //    web-search path, search for the question instead of dead-ending.
    if is_answer_seeking(&lower) && engine_cannot_resolve_locally(&task) {
        return Some(clean_query(&task));
    }

    None
}

/// The issue-#687 web-research recipe step: search the web for `query`, fetch the
/// first source the search surfaces, and answer from it. Returns [`None`] when the
/// client advertises no web-search/fetch tool and no research progress has been
/// made yet, so the caller falls through to the general-change plan instead of
/// dead-ending.
pub(super) fn plan_web_research_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    query: &str,
) -> Option<AgenticPlan> {
    let progress = Progress::scan(messages);

    // Once a source has been fetched, answer from it.
    if progress.done(Capability::Fetch) {
        return Some(AgenticPlan::Final(final_answer(query, &progress)));
    }

    // After searching, fetch the first source the results surfaced.
    if progress.done(Capability::Search) {
        if let Some(tool) = tool_for(tool_names, Capability::Fetch) {
            if let Some(url) = progress.search_output.as_deref().and_then(first_url) {
                return Some(plan_one(tool, fetch_arguments(&url)));
            }
        }
        // No fetch tool or no URL surfaced: answer from the search output directly.
        return Some(AgenticPlan::Final(final_answer(query, &progress)));
    }

    // Nothing done yet: emit the search tool call if the client advertises one.
    if let Some(tool) = tool_for(tool_names, Capability::Search) {
        return Some(plan_one(tool, json!({ "query": query }).to_string()));
    }

    None
}

/// Answer text for a completed web-research step: prefer the fetched source text,
/// falling back to the raw search output, and name the query that was researched.
fn final_answer(query: &str, progress: &Progress) -> String {
    let evidence = progress
        .fetched_text
        .as_deref()
        .or(progress.search_output.as_deref())
        .unwrap_or_default()
        .trim();
    if evidence.is_empty() {
        return format!("I researched \"{query}\" but the tools returned no usable content.");
    }
    format!("Here is what I found about \"{query}\":\n\n{evidence}")
}

/// The first `http(s)` URL in `text`, with trailing punctuation trimmed. Used to
/// pick the source the search results surfaced to fetch next.
pub(super) fn first_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|token| token.starts_with("http://") || token.starts_with("https://"))
        .map(|token| {
            token
                .trim_end_matches(['.', ',', ';', ')', ']', '"', '\''])
                .to_owned()
        })
}

/// Research/lookup imperatives (English + Russian). Matched as substrings so
/// "please learn about it" and "can you research …" both qualify.
const RESEARCH_LEADS: [&str; 14] = [
    "learn about",
    "learn more about",
    "research",
    "find out",
    "find information",
    "look up",
    "look into",
    "read about",
    "study ",
    "look it up",
    "search for",
    "изучи",
    "исследуй",
    "узнай",
];

/// Whether `lower` (a lowercased prompt) is an explicit research imperative.
fn is_research_imperative(lower: &str) -> bool {
    RESEARCH_LEADS.iter().any(|lead| lower.contains(lead))
}

/// Leading phrases stripped when reducing a request to its bare topic. Ordered
/// longest-first so the most specific opener wins.
const TOPIC_LEADS: [&str; 22] = [
    "tell me more about",
    "tell me about",
    "learn more about",
    "learn about",
    "find out about",
    "find information about",
    "read about",
    "look up",
    "look into",
    "research",
    "study",
    "what are the",
    "what is the",
    "what are",
    "what is",
    "who are",
    "who is",
    "where is",
    "where are",
    "when is",
    "when are",
    "about",
];

/// Deictic references that carry no topic of their own and must be resolved from
/// the conversation history.
const DEICTIC: [&str; 11] = [
    "it",
    "this",
    "that",
    "them",
    "these",
    "those",
    "more",
    "this topic",
    "that topic",
    "the topic",
    "about it",
];

/// Resolve a research imperative to the topic to search for: the text after the
/// imperative, or — when that is empty or a bare pronoun — the most recent
/// substantive topic from earlier in the conversation.
fn research_topic(task: &str, messages: &[ChatMessage]) -> Option<String> {
    let bare = strip_lead(&polite_trim(&task.to_lowercase()));
    if !bare.is_empty() && !is_deictic(&bare) {
        return Some(bare);
    }
    topic_from_history(messages)
}

/// The most recent earlier user turn reduced to its bare topic, skipping turns
/// that are themselves bare deictic research requests.
fn topic_from_history(messages: &[ChatMessage]) -> Option<String> {
    let mut users: Vec<String> = messages
        .iter()
        .filter(|m| m.role.eq_ignore_ascii_case("user"))
        .map(|m| m.content.plain_text().trim().to_owned())
        .filter(|text| !text.is_empty())
        .collect();
    users.pop(); // drop the current request
    users.into_iter().rev().find_map(|text| {
        let bare = strip_lead(&polite_trim(&text.to_lowercase()));
        (!bare.is_empty() && !is_deictic(&bare)).then_some(bare)
    })
}

/// Strip a leading politeness marker ("please", "can you", …).
fn polite_trim(lower: &str) -> String {
    let mut value = lower.trim();
    for prefix in ["please ", "can you ", "could you ", "would you ", "pls "] {
        if let Some(rest) = value.strip_prefix(prefix) {
            value = rest.trim();
        }
    }
    value.to_owned()
}

/// Remove a single leading topic-lead phrase and surrounding filler/punctuation.
fn strip_lead(lower: &str) -> String {
    let trimmed = lower.trim();
    for lead in TOPIC_LEADS {
        if let Some(rest) = trimmed.strip_prefix(lead) {
            return rest
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_owned();
        }
    }
    trimmed.trim_end_matches(['.', '!', '?']).trim().to_owned()
}

/// Whether `topic` is a bare deictic reference with no subject of its own.
fn is_deictic(topic: &str) -> bool {
    DEICTIC.contains(&topic.trim())
}

/// Sentence openers that mark an answer-seeking question.
const QUESTION_OPENERS: [&str; 18] = [
    "what ", "when ", "who ", "where ", "why ", "how ", "which ", "whose ", "whom ", "is ", "are ",
    "do ", "does ", "can ", "will ", "should ", "could ", "were ",
];

/// Question marks that end an interrogative sentence across the languages Formal
/// AI supports: ASCII `?`, the CJK full-width `？` (Chinese/Japanese), the Arabic
/// `؟`, and the inverted `¿`. Keeping these together lets a non-English question
/// reach the web the same way an English one does (issue #687 generalization).
const QUESTION_MARKS: [char; 4] = ['?', '？', '؟', '¿'];

/// Whether `lower` is an answer-seeking question (ends with a question mark in any
/// supported script, or opens with a question word).
fn is_answer_seeking(lower: &str) -> bool {
    let trimmed = lower.trim();
    trimmed.ends_with(QUESTION_MARKS)
        || QUESTION_OPENERS
            .iter()
            .any(|opener| trimmed.starts_with(opener))
}

/// Whether the deterministic engine would fail to answer `task` locally — i.e. it
/// resolves to the unknown-reasoning or web-search path. This is what makes the
/// routing a generalization: we web-search precisely what the engine cannot answer
/// from its own knowledge base, and nothing it can.
fn engine_cannot_resolve_locally(task: &str) -> bool {
    let intent = FormalAiEngine.answer(task).intent;
    matches!(intent.as_str(), "unknown" | "web_search")
}

/// Reduce a question to a search query: drop trailing punctuation (including the
/// non-ASCII question marks above), keep the words.
fn clean_query(task: &str) -> String {
    task.trim()
        .trim_end_matches(|c| QUESTION_MARKS.contains(&c) || matches!(c, '.' | '!' | '。'))
        .trim()
        .to_owned()
}

/// The text of the most recent `user` turn.
fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.plain_text())
}
