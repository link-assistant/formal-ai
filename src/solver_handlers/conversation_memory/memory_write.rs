//! Natural-language memory writes: append and Turing-complete substitution.
//!
//! These recognisers turn natural-language messages into read+write control over
//! the entire associative memory (issue #529). Both paths are driven by the seed
//! lexicon rather than hardcoded per-language phrases, so they extend to every
//! supported language (English, Russian, Hindi, Chinese) automatically.

use super::super::finalize_simple;

use crate::coding::contains_cjk;
use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::seed::{self, Slot};

pub(super) fn try_memory_write(
    prompt: &str,
    normalized: &str,
    store: &mut MemoryStore,
    current_conversation_id: Option<&str>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let request = recognize_memory_write(prompt, normalized)?;
    let language = detect_language(prompt).slug();
    // A substitution is a genuine read+write transformation: rewrite every
    // matching value already stored *before* the audit event is appended, so the
    // audit record itself is never rewritten (issue #529).
    let applied = match &request {
        MemoryWriteRequest::Substitute(old_value, new_value) => {
            let count = store.apply_substitution(old_value, new_value);
            log.append("memory_substitution_applied", count.to_string());
            count
        }
        MemoryWriteRequest::Append(_) => 0,
    };
    let event = request.memory_event(store.len(), current_conversation_id, applied);
    let body = request.answer(language, applied);
    log.append("memory_write", request.log_value());
    log.append("memory_write_event", event.id.clone());
    store.append(event);
    Some(finalize_simple(
        prompt,
        log,
        request.intent(),
        request.response_link(),
        &body,
        0.9,
    ))
}

#[derive(Debug, Clone)]
enum MemoryWriteRequest {
    Append(String),
    Substitute(String, String),
}

impl MemoryWriteRequest {
    fn memory_event(
        &self,
        sequence: usize,
        current_conversation_id: Option<&str>,
        applied: usize,
    ) -> MemoryEvent {
        let conversation_id = current_conversation_id.unwrap_or("memory_query");
        match self {
            Self::Append(statement) => MemoryEvent {
                id: stable_id("memory_write", &format!("{sequence}:{statement}")),
                kind: Some(String::from("message")),
                role: Some(String::from("user")),
                intent: Some(String::from("memory_write")),
                content: Some(statement.clone()),
                conversation_id: Some(conversation_id.to_owned()),
                conversation_title: Some(String::from("Memory query")),
                evidence: vec![String::from("memory_write:natural_language")],
                ..MemoryEvent::default()
            },
            Self::Substitute(old_value, new_value) => MemoryEvent {
                id: stable_id(
                    "memory_substitution",
                    &format!("{sequence}:{old_value}->{new_value}"),
                ),
                kind: Some(String::from("memory_substitution")),
                role: Some(String::from("user")),
                intent: Some(String::from("memory_substitution")),
                inputs: Some(format!("replace:{old_value}")),
                outputs: Some(format!("with:{new_value}")),
                content: Some(format!("replace {old_value} with {new_value} in memory")),
                conversation_id: Some(conversation_id.to_owned()),
                conversation_title: Some(String::from("Memory query")),
                evidence: vec![
                    String::from("substitution_event:update"),
                    format!("substitution:applied={applied}"),
                ],
                ..MemoryEvent::default()
            },
        }
    }

    fn answer(&self, language: &str, applied: usize) -> String {
        match self {
            Self::Append(statement) => match language {
                "ru" => format!("Запомнил: {statement}"),
                "zh" => format!("已记住:{statement}"),
                "hi" => format!("स्मृति में सहेजा गया: {statement}"),
                _ => format!("Recorded memory: {statement}"),
            },
            Self::Substitute(old_value, new_value) => match language {
                "ru" => format!(
                    "Заменил \"{old_value}\" на \"{new_value}\" в памяти (обновлено вхождений: {applied})."
                ),
                "zh" => format!(
                    "已在记忆中将\"{old_value}\"替换为\"{new_value}\"(更新 {applied} 处)。"
                ),
                "hi" => format!(
                    "स्मृति में \"{old_value}\" को \"{new_value}\" से बदला ({applied})।"
                ),
                _ => format!(
                    "Replaced \"{old_value}\" with \"{new_value}\" in memory ({applied} occurrence(s) updated)."
                ),
            },
        }
    }

    const fn intent(&self) -> &'static str {
        match self {
            Self::Append(_) => "memory_write",
            Self::Substitute(_, _) => "memory_substitution",
        }
    }

    const fn response_link(&self) -> &'static str {
        match self {
            Self::Append(_) => "response:memory_write",
            Self::Substitute(_, _) => "response:memory_substitution",
        }
    }

    fn log_value(&self) -> String {
        match self {
            Self::Append(statement) => format!("append content={statement}"),
            Self::Substitute(old_value, new_value) => {
                format!("substitute old={old_value} new={new_value}")
            }
        }
    }
}

fn recognize_memory_write(prompt: &str, normalized: &str) -> Option<MemoryWriteRequest> {
    if let Some((old_value, new_value)) = recognize_memory_substitution(normalized) {
        return Some(MemoryWriteRequest::Substitute(old_value, new_value));
    }
    recognize_memory_append(prompt).map(MemoryWriteRequest::Append)
}

/// Recognise a natural-language "remember …" directive in any supported
/// language from the `memory_append_directive` seed role (issue #529).
///
/// The surfaces are [`Slot::Prefix`] forms (trailing `…`), so each form's
/// literal-before-the-slot is the matchable prefix. Matching runs on a
/// lowercased copy of the *raw* prompt — lowercasing is byte-length preserving
/// for en/ru/hi/zh, so the same byte offset slices the original prompt and the
/// recorded statement keeps its original case and punctuation. Longer prefixes
/// are tried first so "remember that X" wins over "remember X".
fn recognize_memory_append(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim_start();
    let lowered = trimmed.to_lowercase();
    let mut prefixes: Vec<String> = seed::lexicon()
        .role_word_forms(seed::ROLE_MEMORY_APPEND_DIRECTIVE)
        .iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(|form| form.before_slot().to_owned())
        .filter(|prefix| !prefix.is_empty())
        .collect();
    prefixes.sort_by_key(|prefix| std::cmp::Reverse(prefix.len()));
    for prefix in prefixes {
        if lowered.starts_with(&prefix) {
            if let Some(statement) = clean_memory_write_text(&trimmed[prefix.len()..]) {
                return Some(statement);
            }
        }
    }
    None
}

/// Recognise a natural-language memory substitution (a read+write transform) in
/// any supported language (issue #529).
///
/// A bare "replace X with Y" is an ordinary coding request, so a memory *scope*
/// phrase (`memory_scope`: "in memory", "в памяти", "स्मृति में", "在记忆中", …)
/// must be present to claim the prompt. We then strip the scope phrase and the
/// substitution directive (`memory_substitution_directive`: "replace", "замени",
/// "रखो", "把", …) — directives lead in SVO languages and trail in Hindi, so a
/// position-independent strip handles both — and split the remaining operand
/// span on the longest matching connector (`memory_substitution_connector`:
/// "with", "на", "की जगह", "换成", …) to recover the (old, new) pair.
fn recognize_memory_substitution(normalized: &str) -> Option<(String, String)> {
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role(seed::ROLE_MEMORY_SCOPE, normalized) {
        return None;
    }
    let without_scope = strip_first_surface(normalized, seed::ROLE_MEMORY_SCOPE)?;
    let operands = strip_first_surface(&without_scope, seed::ROLE_MEMORY_SUBSTITUTION_DIRECTIVE)?;
    let (old_raw, new_raw) =
        split_once_surface(&operands, seed::ROLE_MEMORY_SUBSTITUTION_CONNECTOR)?;
    Some((
        clean_memory_write_text(&old_raw)?,
        clean_memory_write_text(&new_raw)?,
    ))
}

/// Leftmost boundary-aware byte span of `surface` within `haystack`.
///
/// Mirrors [`surface_present`](crate::seed): a CJK surface matches as a
/// substring (no inter-word spaces), while a space-delimited surface must be a
/// whole whitespace token or phrase bounded by the string ends or by spaces.
fn surface_span(haystack: &str, surface: &str) -> Option<(usize, usize)> {
    if surface.is_empty() {
        return None;
    }
    if contains_cjk(surface) {
        let start = haystack.find(surface)?;
        return Some((start, start + surface.len()));
    }
    let mut search = 0;
    while let Some(rel) = haystack[search..].find(surface) {
        let start = search + rel;
        let end = start + surface.len();
        let left_ok = start == 0 || haystack[..start].ends_with(' ');
        let right_ok = end == haystack.len() || haystack[end..].starts_with(' ');
        if left_ok && right_ok {
            return Some((start, end));
        }
        search = start + 1;
    }
    None
}

/// Leftmost boundary-aware span of any surface word of `role` in `haystack`.
///
/// On a tie at the same start offset the longest surface wins, so multi-word
/// phrases ("की जगह") are preferred over any shorter prefix of themselves.
fn best_surface_span(haystack: &str, role: &str) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for surface in seed::lexicon().words_for_role(role) {
        let Some((start, end)) = surface_span(haystack, &surface) else {
            continue;
        };
        best = Some(match best {
            Some(current) if start > current.0 || (start == current.0 && end <= current.1) => {
                current
            }
            _ => (start, end),
        });
    }
    best
}

/// Remove the leftmost occurrence of any surface word of `role`, returning the
/// remaining text with surrounding whitespace collapsed.
fn strip_first_surface(haystack: &str, role: &str) -> Option<String> {
    let (start, end) = best_surface_span(haystack, role)?;
    Some(collapse_ws(&format!(
        "{} {}",
        &haystack[..start],
        &haystack[end..]
    )))
}

/// Split `span` once on the leftmost surface word of `role`, returning the
/// (before, after) operands around the connector.
fn split_once_surface(span: &str, role: &str) -> Option<(String, String)> {
    let (start, end) = best_surface_span(span, role)?;
    Some((span[..start].to_owned(), span[end..].to_owned()))
}

fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_memory_write_text(raw: &str) -> Option<String> {
    let text = raw
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
    (!text.is_empty()).then_some(text)
}
