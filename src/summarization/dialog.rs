//! Dialog-aware helpers for the summarization pipeline.
//!
//! [`DialogTurn`] models a single user/assistant turn. The
//! `formalize_dialog` → `summarize_dialog` → `generate_chat_title` chain runs
//! the same formalize → summarize → deformalize pipeline as the rest of the
//! module, with a role-aware bias so user turns dominate the output when the
//! caller asks for a short summary or a chat title.

use super::markdown::strip_markdown_noise;
use super::{
    deformalize, formalize, summarize, to_topic, Statement, SummarizationConfig, SummarizationMode,
};

/// A single dialog turn passed to [`summarize_dialog`] /
/// [`generate_chat_title`]. The role is informational only — the summarizer
/// uses the text content.
#[derive(Debug, Clone)]
pub struct DialogTurn {
    pub role: String,
    pub text: String,
}

impl DialogTurn {
    /// Build a turn from explicit role + text.
    #[must_use]
    pub fn new(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            text: text.into(),
        }
    }

    /// Convenience constructor for user turns.
    #[must_use]
    pub fn user(text: impl Into<String>) -> Self {
        Self::new("user", text)
    }

    /// Convenience constructor for assistant turns.
    #[must_use]
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new("assistant", text)
    }
}

/// Convert dialog turns into [`Statement`]s with role-aware weighting.
///
/// Each turn's text is formalized individually; user turns are weighted
/// higher than assistant turns so a `Short` summary keeps the user's
/// original questions when both sides are long.
#[must_use]
pub fn formalize_dialog(turns: &[DialogTurn]) -> Vec<Statement> {
    let mut out = Vec::new();
    for turn in turns {
        let bias: i16 = match turn.role.as_str() {
            "user" => 20,
            "assistant" => -10,
            _ => 0,
        };
        for mut stmt in formalize(&turn.text) {
            let bumped = i16::from(stmt.weight).saturating_add(bias).clamp(0, 100);
            stmt.weight = u8::try_from(bumped).unwrap_or(0);
            out.push(stmt);
        }
    }
    out
}

/// Summarize a dialog. The output preserves the order of the highest-weight
/// statements (user questions first when they tie with assistant prose) and
/// passes through [`deformalize`] for display.
#[must_use]
pub fn summarize_dialog(turns: &[DialogTurn], config: &SummarizationConfig) -> String {
    let statements = formalize_dialog(turns);
    if config.mode.is_label_only() {
        let highest = statements.iter().max_by_key(|s| s.weight);
        return highest
            .map(|s| super::label_for_mode(config.mode, &to_topic("", std::slice::from_ref(s))))
            .unwrap_or_default();
    }
    if statements.is_empty() {
        return String::new();
    }
    let summarized = summarize(&statements, config);
    deformalize(&summarized)
}

/// Summarize the current dialog task and status as bounded plain prose.
///
/// The most recent user turn is the current task. The most recent assistant
/// turn after it is the current status. Each contributes at most one sentence,
/// Markdown noise is removed, and the combined result is capped to the caller's
/// word and sentence budgets. This is intended for compact protocol recaps;
/// [`summarize_dialog`] remains the configurable general-purpose pipeline.
#[must_use]
pub fn summarize_dialog_plain(
    turns: &[DialogTurn],
    max_words: usize,
    max_sentences: usize,
) -> String {
    if max_words == 0 || max_sentences == 0 {
        return String::new();
    }
    let Some(user_index) = turns
        .iter()
        .rposition(|turn| turn.role.eq_ignore_ascii_case("user"))
    else {
        return String::new();
    };
    let mut sentences = Vec::with_capacity(max_sentences.min(2));
    if let Some(goal) = plain_first_sentence(&turns[user_index].text) {
        sentences.push(goal);
    }
    if max_sentences > 1 {
        let status = turns[user_index + 1..]
            .iter()
            .rev()
            .find(|turn| turn.role.eq_ignore_ascii_case("assistant"))
            .and_then(|turn| plain_first_sentence(&turn.text));
        if let Some(status) = status.filter(|status| {
            sentences
                .first()
                .is_none_or(|goal| !goal.eq_ignore_ascii_case(status))
        }) {
            sentences.push(status);
        }
    }
    bound_plain_words(&sentences.join(" "), max_words)
}

fn plain_first_sentence(text: &str) -> Option<String> {
    let cleaned = strip_markdown_noise(text);
    let without_markers: String = cleaned
        .chars()
        .filter(|character| !matches!(character, '#' | '`' | '*'))
        .collect();
    let mut plain = without_markers
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for separator in [" — ", " – "] {
        if let Some(index) = plain.find(separator) {
            plain.truncate(index);
        }
    }
    let mut boundary = None;
    let mut characters = plain.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        let next_is_boundary = characters
            .peek()
            .is_none_or(|(_, next)| next.is_whitespace());
        if matches!(character, '。' | '！' | '？')
            || (matches!(character, '.' | '!' | '?') && next_is_boundary)
        {
            boundary = Some(index + character.len_utf8());
            break;
        }
    }
    if let Some(boundary) = boundary {
        plain.truncate(boundary);
    }
    let plain = plain.trim().to_owned();
    if plain.is_empty() {
        None
    } else if plain.ends_with(['.', '!', '?', '。', '！', '？']) {
        Some(plain)
    } else {
        Some(format!("{plain}."))
    }
}

fn bound_plain_words(text: &str, max_words: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_words {
        return text.to_owned();
    }
    let mut bounded = words[..max_words].join(" ");
    while bounded.ends_with(['.', ',', ';', ':', '!', '?', '。', '！', '？']) {
        bounded.pop();
    }
    bounded.push('.');
    bounded
}

/// Generate a 1–5 word chat title from a dialog.
///
/// Equivalent to running [`summarize_dialog`] in `Topic` mode but spelled
/// out so the call site reads as `generate_chat_title(turns, "en")` instead
/// of building a config.
#[must_use]
pub fn generate_chat_title(turns: &[DialogTurn], language: &str) -> String {
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Topic)
        .with_language(language);
    summarize_dialog(turns, &config)
}
