use std::fmt::Write as _;

mod memory_write;
use memory_write::try_memory_write;

use super::finalize_simple;

use crate::coding::contains_cjk;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::link_store::memory_events_to_link_records;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::seed::{self, Slot, WordForm};
use crate::solver_helpers::{
    extract_introduced_name, last_turn, last_user_turn, recall_name_from_history,
};
use crate::summarization::{
    generate_chat_title, summarize_dialog, DialogTurn, SummarizationConfig, SummarizationMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
struct MemoryRecallMatch {
    event_index: usize,
    role: String,
    conversation_id: String,
    conversation_title: String,
    sent_at: String,
    detail: MemoryRecallDetail,
}

#[derive(Debug)]
enum MemoryRecallDetail {
    Field { name: &'static str, value: String },
    Link { from: String, to: String },
}

/// Result of executing a natural-language memory query against a mutable store.
#[derive(Debug, Clone)]
pub struct MemoryQueryExecution {
    pub answer: SymbolicAnswer,
    pub changed: bool,
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
    if let Some(answer) = try_recall_previous_message(prompt, normalized, log) {
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

#[must_use]
pub fn answer_memory_recall(
    prompt: &str,
    events: &[MemoryEvent],
    current_conversation_id: Option<&str>,
) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    log.append("impulse", prompt.to_owned());
    try_memory_recall(
        prompt,
        &normalized,
        events,
        current_conversation_id,
        &mut log,
    )
}

#[must_use]
pub fn execute_memory_query(
    prompt: &str,
    store: &mut MemoryStore,
    current_conversation_id: Option<&str>,
) -> Option<MemoryQueryExecution> {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    log.append("impulse", prompt.to_owned());
    if let Some(answer) = try_memory_write(
        prompt,
        &normalized,
        store,
        current_conversation_id,
        &mut log,
    ) {
        return Some(MemoryQueryExecution {
            answer,
            changed: true,
        });
    }
    answer_memory_recall(prompt, store.events(), current_conversation_id).map(|answer| {
        // Issue #494: usage is counted on read access — every store event the
        // recall actually read gets its access count bumped, and the caller
        // persists the store so dreaming sees frequently-read data as used.
        let accessed =
            recalled_event_indices(&normalized, store.events(), current_conversation_id, prompt);
        let changed = store.record_access(&accessed) > 0;
        MemoryQueryExecution { answer, changed }
    })
}

/// Indices of the store events a recall for this prompt reads.
fn recalled_event_indices(
    normalized: &str,
    events: &[MemoryEvent],
    current_conversation_id: Option<&str>,
    prompt: &str,
) -> Vec<usize> {
    let Some(query) = recognize_recall_query(normalized) else {
        return Vec::new();
    };
    let mut indices: Vec<usize> =
        memory_recall_matches(events, &query, current_conversation_id, prompt)
            .iter()
            // `event_index` is 1-based for display; the store slice is 0-based.
            .map(|matched| matched.event_index.saturating_sub(1))
            .collect();
    indices.sort_unstable();
    indices.dedup();
    indices
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
    let cleaned = normalize_prompt(normalized);
    let asks = seed::lexicon().mentions_role(
        seed::ROLE_CONVERSATION_RECALL_PREVIOUS_USER_MESSAGE,
        &cleaned,
    );
    if !asks {
        return None;
    }
    let previous = previous_user_request_turn(log)?;
    let language = detect_language(prompt).slug();
    let body = render_previous_user_message(&previous, language);
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

fn previous_user_request_turn(log: &EventLog) -> Option<String> {
    let latest_user = last_user_turn(log).map(ToOwned::to_owned);
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:user")
        .filter_map(|event| {
            let content = event.payload.trim();
            (!content.is_empty()).then_some(content)
        })
        .find(|content| !is_recall_meta_prompt(content))
        .map(ToOwned::to_owned)
        .or(latest_user)
}

fn is_recall_meta_prompt(prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    seed::lexicon().mentions_role(
        seed::ROLE_CONVERSATION_RECALL_PREVIOUS_USER_MESSAGE,
        &normalized,
    ) || seed::lexicon().mentions_role(seed::ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE, &normalized)
        || recognize_recall_query(&normalized).is_some()
}

fn render_previous_user_message(content: &str, language: &str) -> String {
    match language {
        "ru" => format!("Вы спрашивали: \"{content}\""),
        "zh" => format!("你之前问的是:\"{content}\""),
        "hi" => format!("आपने पूछा था: \"{content}\""),
        _ => format!("Your previous question was: {content}"),
    }
}

/// Recall the content of the immediately preceding message (issue #529).
///
/// Recognizes language-agnostic "what was written in the previous message?"
/// phrasings via the `conversation_recall_previous_message` seed role and
/// replays the last prior turn regardless of role. Unlike
/// [`try_recall_last_question`], which returns the user's own last question,
/// this returns whatever message came immediately before the current prompt —
/// for the issue's scenario, the assistant's previous reply.
fn try_recall_previous_message(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !seed::lexicon().mentions_role(
        seed::ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE,
        &normalize_prompt(normalized),
    ) {
        return None;
    }
    let language = detect_language(prompt).slug();
    let previous = last_turn(log).map(|(role, content)| (role, content.to_owned()));
    let body = if let Some((role, content)) = previous {
        log.append("filter:user", format!("previous_message role={role}"));
        render_previous_message(role, &content, language)
    } else {
        log.append("filter:user", "previous_message:none".to_owned());
        render_no_previous_message(language)
    };
    Some(finalize_simple(
        prompt,
        log,
        "recall_previous_message",
        "response:recall_previous_message",
        &body,
        0.9,
    ))
}

/// Localize the role of a recalled message ("user"/"assistant").
fn localized_role(role: &str, language: &str) -> &'static str {
    match (role, language) {
        ("assistant", "ru") => "ассистент",
        ("assistant", "hi") => "सहायक",
        ("assistant", "zh") => "助手",
        ("assistant", _) => "assistant",
        (_, "ru") => "пользователь",
        (_, "hi") => "उपयोगकर्ता",
        (_, "zh") => "用户",
        (_, _) => "user",
    }
}

fn render_previous_message(role: &str, content: &str, language: &str) -> String {
    let role_label = localized_role(role, language);
    match language {
        "ru" => format!("В прошлом сообщении ({role_label}) было написано: \"{content}\""),
        "zh" => format!("上一条消息（{role_label}）写道:\"{content}\""),
        "hi" => format!("पिछले संदेश ({role_label}) में लिखा था: \"{content}\""),
        _ => format!("The previous message ({role_label}) was: \"{content}\""),
    }
}

fn render_no_previous_message(language: &str) -> String {
    match language {
        "ru" => String::from("Прошлого сообщения пока нет."),
        "zh" => String::from("还没有上一条消息。"),
        "hi" => String::from("अभी तक कोई पिछला संदेश नहीं है."),
        _ => String::from("There is no previous message yet."),
    }
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

fn try_memory_recall(
    prompt: &str,
    normalized: &str,
    events: &[MemoryEvent],
    current_conversation_id: Option<&str>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let query = recognize_recall_query(normalized)?;
    let matches = memory_recall_matches(events, &query, current_conversation_id, prompt);
    let conversation_count = memory_conversation_count(&matches);
    log.append("filter:memory_query", query.term.clone());
    log.append("filter:memory_scope", query.scope.as_str());
    log.append("filter:memory_matches", matches.len().to_string());
    log.append(
        "filter:memory_conversations",
        conversation_count.to_string(),
    );
    for matched in &matches {
        log.append(
            "memory_match",
            format!(
                "event={} conversation={} title={} role={} {}",
                matched.event_index,
                matched.conversation_id,
                matched.conversation_title,
                matched.role,
                matched.log_fragment()
            ),
        );
    }
    let language = detect_language(prompt).slug();
    let body = render_memory_recall_report(&query, &matches, language);
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

fn memory_recall_matches(
    events: &[MemoryEvent],
    query: &RecallQuery,
    current_conversation_id: Option<&str>,
    trigger_text: &str,
) -> Vec<MemoryRecallMatch> {
    let needle = normalize_prompt(&query.term);
    if needle.is_empty() {
        return Vec::new();
    }
    let trigger = normalize_prompt(trigger_text);
    let mut matches = Vec::new();
    for (index, event) in events.iter().enumerate() {
        if !event_in_recall_scope(event, query, current_conversation_id) {
            continue;
        }
        for (name, value) in memory_event_field_values(event) {
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            let haystack = normalize_prompt(value);
            if !haystack.contains(&needle) {
                continue;
            }
            if name == "content" && !trigger.is_empty() && haystack == trigger {
                continue;
            }
            matches.push(memory_field_match(index, event, name, value));
        }
    }

    for (index, (event, record)) in events
        .iter()
        .zip(memory_events_to_link_records(events))
        .enumerate()
    {
        if !event_in_recall_scope(event, query, current_conversation_id) {
            continue;
        }
        for link in record.links {
            let haystack = normalize_prompt(&format!("{} {}", link.from, link.to));
            if !haystack.contains(&needle) {
                continue;
            }
            matches.push(memory_link_match(index, event, &link.from, &link.to));
        }
    }

    matches
}

fn event_in_recall_scope(
    event: &MemoryEvent,
    query: &RecallQuery,
    current_conversation_id: Option<&str>,
) -> bool {
    let conversation_id = event.conversation_id.as_deref().unwrap_or("legacy");
    query.scope != RecallScope::OtherConversations
        || current_conversation_id.is_none_or(|current| current != conversation_id)
}

fn memory_event_field_values(event: &MemoryEvent) -> Vec<(&'static str, &str)> {
    let mut fields = Vec::new();
    push_memory_field(&mut fields, "id", Some(event.id.as_str()));
    push_memory_field(&mut fields, "kind", event.kind.as_deref());
    push_memory_field(&mut fields, "role", event.role.as_deref());
    push_memory_field(&mut fields, "intent", event.intent.as_deref());
    push_memory_field(&mut fields, "tool", event.tool.as_deref());
    push_memory_field(&mut fields, "inputs", event.inputs.as_deref());
    push_memory_field(&mut fields, "outputs", event.outputs.as_deref());
    push_memory_field(&mut fields, "content", event.content.as_deref());
    push_memory_field(&mut fields, "sentAt", event.sent_at.as_deref());
    push_memory_field(&mut fields, "demoLabel", event.demo_label.as_deref());
    push_memory_field(
        &mut fields,
        "conversationId",
        event.conversation_id.as_deref(),
    );
    push_memory_field(
        &mut fields,
        "conversationTitle",
        event.conversation_title.as_deref(),
    );
    for evidence in &event.evidence {
        push_memory_field(&mut fields, "evidence", Some(evidence.as_str()));
    }
    fields
}

fn push_memory_field<'a>(
    fields: &mut Vec<(&'static str, &'a str)>,
    name: &'static str,
    value: Option<&'a str>,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        fields.push((name, value));
    }
}

fn memory_field_match(
    index: usize,
    event: &MemoryEvent,
    name: &'static str,
    value: &str,
) -> MemoryRecallMatch {
    memory_match(
        index,
        event,
        MemoryRecallDetail::Field {
            name,
            value: value.to_owned(),
        },
    )
}

fn memory_link_match(index: usize, event: &MemoryEvent, from: &str, to: &str) -> MemoryRecallMatch {
    memory_match(
        index,
        event,
        MemoryRecallDetail::Link {
            from: from.to_owned(),
            to: to.to_owned(),
        },
    )
}

fn memory_match(
    index: usize,
    event: &MemoryEvent,
    detail: MemoryRecallDetail,
) -> MemoryRecallMatch {
    let role = event
        .role
        .as_deref()
        .or(event.kind.as_deref())
        .or(event.intent.as_deref())
        .unwrap_or("event");
    MemoryRecallMatch {
        event_index: index + 1,
        role: role.to_ascii_lowercase(),
        conversation_id: event
            .conversation_id
            .as_deref()
            .unwrap_or("legacy")
            .to_owned(),
        conversation_title: event
            .conversation_title
            .as_deref()
            .unwrap_or_default()
            .to_owned(),
        sent_at: event.sent_at.as_deref().unwrap_or_default().to_owned(),
        detail,
    }
}

impl MemoryRecallMatch {
    fn log_fragment(&self) -> String {
        match &self.detail {
            MemoryRecallDetail::Field { name, value } => format!("field={name} value={value}"),
            MemoryRecallDetail::Link { from, to } => format!("link={from}->{to}"),
        }
    }

    fn render_line(&self) -> String {
        let stamp = if self.sent_at.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.sent_at)
        };
        match &self.detail {
            MemoryRecallDetail::Field { name, value } if *name == "content" => {
                format!("{}{}: {}", self.role, stamp, value)
            }
            MemoryRecallDetail::Field { name, value } => format!("{name}{stamp}: {value}"),
            MemoryRecallDetail::Link { from, to } => format!("link{stamp}: {from} -> {to}"),
        }
    }
}

fn memory_conversation_count(matches: &[MemoryRecallMatch]) -> usize {
    let mut ids: Vec<&str> = Vec::new();
    for matched in matches {
        if !ids.contains(&matched.conversation_id.as_str()) {
            ids.push(matched.conversation_id.as_str());
        }
    }
    ids.len()
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

fn render_memory_recall_report(
    query: &RecallQuery,
    matches: &[MemoryRecallMatch],
    language: &str,
) -> String {
    if matches.is_empty() {
        return match language {
            "ru" => format!("Упоминаний \"{}\" в памяти не найдено.", query.term),
            "zh" => format!("在记忆中没有找到 \"{}\"。", query.term),
            "hi" => format!("स्मृति में \"{}\" नहीं मिला.", query.term),
            _ => format!("No mentions of \"{}\" found in memory.", query.term),
        };
    }

    let conversation_count = memory_conversation_count(matches);
    let mut body = match language {
        "ru" => format!(
            "Найдено упоминаний \"{}\" в памяти: {} (бесед: {}).\n",
            query.term,
            matches.len(),
            conversation_count
        ),
        "zh" => format!(
            "在记忆中找到 \"{}\" 的记录: {} (对话: {})。\n",
            query.term,
            matches.len(),
            conversation_count
        ),
        "hi" => format!(
            "स्मृति में \"{}\" के उल्लेख मिले: {} (बातचीत: {}).\n",
            query.term,
            matches.len(),
            conversation_count
        ),
        _ => format!(
            "Found {} mention(s) of \"{}\" across {} conversation(s) in memory.\n",
            matches.len(),
            query.term,
            conversation_count
        ),
    };

    let mut conversation_ids: Vec<&str> = Vec::new();
    for matched in matches {
        if !conversation_ids.contains(&matched.conversation_id.as_str()) {
            conversation_ids.push(matched.conversation_id.as_str());
        }
    }
    for conversation_id in conversation_ids {
        let title = matches
            .iter()
            .find(|matched| {
                matched.conversation_id == conversation_id && !matched.conversation_title.is_empty()
            })
            .map_or("", |matched| matched.conversation_title.as_str());
        let label = if title.is_empty() || title == conversation_id {
            conversation_id.to_owned()
        } else {
            format!("{title} ({conversation_id})")
        };
        writeln!(body, "- conversation {label}").expect("string write is infallible");
        for matched in matches
            .iter()
            .filter(|matched| matched.conversation_id == conversation_id)
        {
            writeln!(body, "  - {}", matched.render_line()).expect("string write is infallible");
        }
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
