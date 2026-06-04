//! Dialog-aware helpers for the summarization pipeline.
//!
//! [`DialogTurn`] models a single user/assistant turn. The
//! `formalize_dialog` → `summarize_dialog` → `generate_chat_title` chain runs
//! the same formalize → summarize → deformalize pipeline as the rest of the
//! module, with a role-aware bias so user turns dominate the output when the
//! caller asks for a short summary or a chat title.

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
    if config.mode == SummarizationMode::Topic {
        let highest = statements.iter().max_by_key(|s| s.weight);
        return highest
            .map(|s| to_topic("", std::slice::from_ref(s)))
            .unwrap_or_default();
    }
    if statements.is_empty() {
        return String::new();
    }
    let summarized = summarize(&statements, config);
    deformalize(&summarized)
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
