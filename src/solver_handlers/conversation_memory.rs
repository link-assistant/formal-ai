use std::fmt::Write as _;

use super::finalize_simple;

use crate::coding::contains_cjk;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, Slot, WordForm};
use crate::solver_helpers::{extract_introduced_name, last_user_turn, recall_name_from_history};
use crate::summarization::{
    generate_chat_title, summarize_dialog, DialogTurn, SummarizationConfig, SummarizationMode,
};

#[derive(Debug, Clone, Copy)]
enum RecallScope {
    Conversation,
    OtherConversations,
}

impl RecallScope {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::OtherConversations => "other_conversations",
        }
    }
}

#[derive(Debug)]
struct RecallQuery {
    term: String,
    scope: RecallScope,
}

#[derive(Debug)]
struct RecallMatch {
    turn_index: usize,
    role: &'static str,
    content: String,
}

pub fn try_conversation_memory(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if let Some(answer) = try_recall_name(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = try_recall_last_question(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = try_conversation_recall(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = try_summarize_conversation(prompt, normalized, log) {
        return Some(answer);
    }
    None
}

fn try_recall_name(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let asks_name = normalized.contains("what is my name")
        || normalized.contains("what's my name")
        || normalized.contains("do you know my name")
        || normalized.contains("who am i");
    if !asks_name {
        return None;
    }
    let name = recall_name_from_history(log, prompt).or_else(|| extract_introduced_name(prompt))?;
    log.append("filter:user", format!("name={name}"));
    let body = format!("Your name is {name}.");
    Some(finalize_simple(
        prompt,
        log,
        "recall_name",
        "response:recall_name",
        &body,
        0.9,
    ))
}

fn try_recall_last_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks = normalized.contains("what did i ask")
        || normalized.contains("what was my last question")
        || normalized.contains("what was my previous question")
        || normalized.contains("repeat my last message");
    if !asks {
        return None;
    }
    let previous = last_user_turn(log)?;
    let body = format!("Your previous message was: \"{previous}\"");
    log.append("filter:user", "previous_turn".to_owned());
    Some(finalize_simple(
        prompt,
        log,
        "recall_last_question",
        "response:recall_last_question",
        &body,
        0.9,
    ))
}

fn try_conversation_recall(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let query = recognize_recall_query(normalized)?;
    let matches = recall_matches(log, &query.term);
    log.append("filter:memory_query", query.term.clone());
    log.append("filter:memory_scope", query.scope.as_str());
    log.append("filter:memory_matches", matches.len().to_string());
    for matched in &matches {
        log.append(
            "memory_match",
            format!(
                "turn={} role={} content={}",
                matched.turn_index, matched.role, matched.content
            ),
        );
    }
    let language = detect_language(prompt).slug();
    let body = render_recall_report(&query, &matches, language);
    Some(finalize_simple(
        prompt,
        log,
        "conversation_recall",
        "response:conversation_recall",
        &body,
        0.9,
    ))
}

fn recognize_recall_query(normalized: &str) -> Option<RecallQuery> {
    recall_term_for_role(seed::ROLE_CONVERSATION_RECALL_QUERY, normalized)
        .map(|term| RecallQuery {
            term,
            scope: RecallScope::Conversation,
        })
        .or_else(|| {
            recall_term_for_role(seed::ROLE_CONVERSATION_RECALL_OTHER_QUERY, normalized).map(
                |term| RecallQuery {
                    term,
                    scope: RecallScope::OtherConversations,
                },
            )
        })
}

fn recall_term_for_role(role: &str, normalized: &str) -> Option<String> {
    seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter_map(|form| term_from_form(form, normalized))
        .find(|term| !term.is_empty())
}

fn term_from_form(form: &WordForm, normalized: &str) -> Option<String> {
    let raw = match form.slot() {
        Slot::Prefix => normalized.strip_prefix(form.before_slot())?,
        Slot::Suffix => normalized.strip_suffix(form.after_slot())?,
        Slot::Circumfix => normalized
            .strip_prefix(form.before_slot())?
            .strip_suffix(form.after_slot())?,
        Slot::Bare => return None,
    };
    clean_recall_term(raw)
}

fn clean_recall_term(raw: &str) -> Option<String> {
    let term = raw
        .trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '`' | '"' | '\'' | ':' | '-' | '_' | '.' | ',' | '?' | '!' | '(' | ')'
                )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!term.is_empty()).then_some(term)
}

fn recall_matches(log: &EventLog, term: &str) -> Vec<RecallMatch> {
    let needle = normalize_prompt(term);
    if needle.is_empty() {
        return Vec::new();
    }
    log.events()
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            let role = match event.kind {
                "prior_turn:user" => "user",
                "prior_turn:assistant" => "assistant",
                _ => return None,
            };
            let haystack = normalize_prompt(&event.payload);
            haystack.contains(&needle).then(|| RecallMatch {
                turn_index: index + 1,
                role,
                content: event.payload.clone(),
            })
        })
        .collect()
}

fn render_recall_report(query: &RecallQuery, matches: &[RecallMatch], language: &str) -> String {
    if matches.is_empty() {
        return match language {
            "ru" => format!(
                "Упоминаний \"{}\" в истории разговора не найдено.",
                query.term
            ),
            "zh" => format!("在对话历史中没有找到 \"{}\"。", query.term),
            "hi" => format!("बातचीत के इतिहास में \"{}\" नहीं मिला.", query.term),
            _ => format!(
                "No mentions of \"{}\" found in the conversation history.",
                query.term
            ),
        };
    }

    let mut body = match language {
        "ru" => format!(
            "Найдено упоминаний \"{}\" в истории разговора: {}\n",
            query.term,
            matches.len()
        ),
        "zh" => format!(
            "在对话历史中找到 \"{}\" 的记录: {}\n",
            query.term,
            matches.len()
        ),
        "hi" => format!(
            "बातचीत के इतिहास में \"{}\" के उल्लेख मिले: {}\n",
            query.term,
            matches.len()
        ),
        _ => format!(
            "Found {} mention(s) of \"{}\" in the conversation history.\n",
            matches.len(),
            query.term
        ),
    };
    for matched in matches {
        writeln!(
            body,
            "- turn {} {}: {}",
            matched.turn_index, matched.role, matched.content
        )
        .expect("string write is infallible");
    }
    body.trim_end().to_owned()
}

/// Recognise a request to summarize the running conversation by composing
/// meaning roles rather than matching raw per-language phrases (issue #386).
///
/// The universal algorithm is identical for every language: the prompt either
/// (a) carries a complete standalone conversation-summary phrasing, (b) carries
/// an objectless courtesy frame asking for a summary, (c) names a summary
/// directive *together with* a conversation reference, or (d) leads with a bare
/// summary directive (`summarize`, `резюме`, `总结`, …). The prompt is
/// re-normalised first so the boundary-aware matcher sees punctuation collapsed
/// to spaces. Mirror of `asksForConversationSummary` in the browser worker.
fn asks_for_conversation_summary(normalized: &str) -> bool {
    let cleaned = normalize_prompt(normalized);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_PHRASE, &cleaned)
        || lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_COURTESY, &cleaned)
        || (lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_DIRECTIVE, &cleaned)
            && lexicon.mentions_role(seed::ROLE_CONVERSATION_REFERENCE, &cleaned))
        || summary_directive_leads(&cleaned)
}

/// A bare summary directive standing alone is itself a request to summarize the
/// running conversation ("summarize", "резюме", "总结", …).
///
/// For whitespace-delimited scripts the directive must be the *whole* prompt, so
/// "summarize the article" is left for other handlers (a conversation object is
/// required via the directive∧reference arm instead). For CJK (no word spaces) a
/// leading substring suffices — mirroring the worker's historical `^总结` anchor
/// — which also keeps compounds like "工作总结" (a *work* summary) from being
/// mis-claimed. Surface words come from the `conversation_summary_directive`
/// role in the seed lexicon.
fn summary_directive_leads(cleaned: &str) -> bool {
    seed::lexicon()
        .words_for_role(seed::ROLE_CONVERSATION_SUMMARY_DIRECTIVE)
        .iter()
        .any(|word| {
            if contains_cjk(word) {
                cleaned.starts_with(word.as_str())
            } else {
                cleaned == word.as_str()
            }
        })
}

fn try_summarize_conversation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !asks_for_conversation_summary(normalized) {
        return None;
    }
    let turns: Vec<DialogTurn> = log
        .events()
        .iter()
        .filter_map(|event| match event.kind {
            "prior_turn:user" => Some(DialogTurn::user(event.payload.clone())),
            "prior_turn:assistant" => Some(DialogTurn::assistant(event.payload.clone())),
            _ => None,
        })
        .collect();
    let user_turn_count = turns.iter().filter(|t| t.role == "user").count();
    if user_turn_count == 0 {
        return None;
    }
    let language = detect_language(prompt).slug();
    // Standard mode keeps roughly 50% of the highest-weighted statements; with
    // the dialog bias (user +20, assistant -10) the user's questions dominate
    // the output while still keeping room for any assistant prose worth
    // remembering.
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language(language);
    let summary = summarize_dialog(&turns, &config);
    let title = generate_chat_title(&turns, language);
    let user_turns: Vec<&str> = turns
        .iter()
        .filter(|t| t.role == "user")
        .map(|t| t.text.as_str())
        .collect();
    let mut body = match language {
        "ru" => {
            format!("Резюме разговора: {summary}\n\nЗаголовок: {title}\n\nРеплики пользователя:\n")
        }
        "zh" => format!("对话摘要:{summary}\n\n标题:{title}\n\n用户发言:\n"),
        _ => format!("Conversation summary: {summary}\n\nTitle: {title}\n\nUser turns:\n"),
    };
    for (index, turn) in user_turns.iter().enumerate() {
        writeln!(body, "  {}. {turn}", index + 1).expect("string write is infallible");
    }
    log.append("filter:user", "conversation_summary".to_owned());
    log.append("summarization:mode", "standard".to_owned());
    log.append("summarization:language", language.to_owned());
    log.append("chat_title", title);
    Some(finalize_simple(
        prompt,
        log,
        "summarize_conversation",
        "response:summarize_conversation",
        body.trim_end(),
        0.9,
    ))
}
