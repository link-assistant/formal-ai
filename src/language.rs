//! Lightweight, deterministic language detection.
//!
//! The universal solver tags every impulse with a detected language so the
//! evidence trail carries `language:en`, `language:ru`, `language:hi`,
//! `language:zh`, or `language:unknown`. Detection is based on Unicode block
//! ranges of the dominant script in the prompt — no neural inference and no
//! external service. See `VISION.md` and `REQUIREMENTS.md` for the rules.

/// Detected language slug used inside `language:<slug>` evidence links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
    Hindi,
    Chinese,
    Unknown,
}

impl Language {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Russian => "ru",
            Self::Hindi => "hi",
            Self::Chinese => "zh",
            Self::Unknown => "unknown",
        }
    }
}

/// Detect the dominant language of a prompt.
///
/// Counts characters in known Unicode blocks. Latin-only text is treated as
/// English by default; pure scripts in unsupported blocks (Arabic, Hebrew,
/// etc.) are returned as `Unknown` so the loop can record an explicit
/// `language:unknown` event.
#[must_use]
pub fn detect(prompt: &str) -> Language {
    let mut latin = 0usize;
    let mut cyrillic = 0usize;
    let mut devanagari = 0usize;
    let mut cjk = 0usize;
    let mut other_script = 0usize;

    for character in prompt.chars() {
        let codepoint = u32::from(character);
        if character.is_ascii_alphabetic() {
            latin += 1;
        } else if (0x0400..=0x04FF).contains(&codepoint) {
            cyrillic += 1;
        } else if (0x0900..=0x097F).contains(&codepoint) {
            devanagari += 1;
        } else if (0x4E00..=0x9FFF).contains(&codepoint) {
            cjk += 1;
        } else if character.is_alphabetic() {
            other_script += 1;
        }
    }

    let total_script = latin + cyrillic + devanagari + cjk + other_script;
    if total_script == 0 {
        return Language::English;
    }

    if other_script > latin
        && other_script >= cyrillic
        && other_script >= devanagari
        && other_script >= cjk
    {
        return Language::Unknown;
    }
    if cyrillic >= latin.max(devanagari).max(cjk) && cyrillic > 0 {
        return Language::Russian;
    }
    if devanagari >= latin.max(cyrillic).max(cjk) && devanagari > 0 {
        return Language::Hindi;
    }
    if cjk >= latin.max(cyrillic).max(devanagari) && cjk > 0 {
        return Language::Chinese;
    }
    Language::English
}
