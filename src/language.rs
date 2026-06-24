//! Lightweight, deterministic language detection.
//!
//! The universal solver tags every impulse with a detected language so the
//! evidence trail carries `language:en`, `language:ru`, `language:hi`,
//! `language:zh`, or `language:unknown`. Detection is based on Unicode block
//! ranges in the prompt — no neural inference and no external service. See
//! `VISION.md` and `REQUIREMENTS.md` for the rules.

/// Detected language slug used inside `language:<slug>` evidence links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
    Hindi,
    Chinese,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Script {
    Latin,
    Cyrillic,
    Devanagari,
    Cjk,
    Other,
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
/// English by default; when a supported non-Latin prompt starts the command or
/// contains a local question marker alongside Latin identifiers, that prompt
/// language is preserved.
/// Pure scripts in unsupported blocks (Arabic, Hebrew, etc.) are returned as
/// `Unknown` so the loop can record an explicit `language:unknown` event.
#[must_use]
pub fn detect(prompt: &str) -> Language {
    let mut latin = 0usize;
    let mut cyrillic = 0usize;
    let mut devanagari = 0usize;
    let mut cjk = 0usize;
    let mut other_script = 0usize;
    let mut first_script = None;

    for character in prompt.chars() {
        let codepoint = u32::from(character);
        if character.is_ascii_alphabetic() {
            latin += 1;
            first_script.get_or_insert(Script::Latin);
        } else if (0x0400..=0x04FF).contains(&codepoint) {
            cyrillic += 1;
            first_script.get_or_insert(Script::Cyrillic);
        } else if (0x0900..=0x097F).contains(&codepoint) {
            devanagari += 1;
            first_script.get_or_insert(Script::Devanagari);
        } else if (0x4E00..=0x9FFF).contains(&codepoint) {
            cjk += 1;
            first_script.get_or_insert(Script::Cjk);
        } else if character.is_alphabetic() {
            other_script += 1;
            first_script.get_or_insert(Script::Other);
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
    if latin > 0 {
        if let Some(language) = marker_language(prompt, cyrillic, devanagari, cjk) {
            return language;
        }
        match first_script {
            Some(Script::Cyrillic) if cyrillic >= devanagari.max(cjk) => {
                return Language::Russian;
            }
            Some(Script::Devanagari) if devanagari >= cyrillic.max(cjk) => {
                return Language::Hindi;
            }
            Some(Script::Cjk) if cjk >= cyrillic.max(devanagari) => return Language::Chinese,
            _ => {}
        }
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

fn marker_language(
    prompt: &str,
    cyrillic: usize,
    devanagari: usize,
    cjk: usize,
) -> Option<Language> {
    let normalized = prompt.to_lowercase();
    let mut best = None;
    for (script, count, language) in [
        (Script::Cyrillic, cyrillic, Language::Russian),
        (Script::Devanagari, devanagari, Language::Hindi),
        (Script::Cjk, cjk, Language::Chinese),
    ] {
        if count == 0 || !contains_question_marker(&normalized, script) {
            continue;
        }
        match best {
            Some((best_count, _)) if count <= best_count => {}
            _ => best = Some((count, language)),
        }
    }
    best.map(|(_, language)| language)
}

fn contains_question_marker(prompt: &str, script: Script) -> bool {
    let markers: &[&str] = match script {
        Script::Cyrillic => &[
            "\u{0447}\u{0442}\u{043e}",
            "\u{043a}\u{0430}\u{043a}",
            "\u{043a}\u{0442}\u{043e}",
            "\u{0433}\u{0434}\u{0435}",
            "\u{043a}\u{043e}\u{0433}\u{0434}\u{0430}",
            "\u{043f}\u{043e}\u{0447}\u{0435}\u{043c}\u{0443}",
        ],
        Script::Devanagari => &[
            "\u{0915}\u{094d}\u{092f}\u{093e}",
            "\u{0915}\u{094c}\u{0928}",
            "\u{0915}\u{0939}\u{093e}\u{0901}",
            "\u{0915}\u{092c}",
            "\u{0915}\u{0948}\u{0938}\u{0947}",
            "\u{0915}\u{094d}\u{092f}\u{094b}\u{0902}",
        ],
        Script::Cjk => &[
            "\u{4ec0}\u{4e48}",
            "\u{5417}",
            "\u{600e}\u{4e48}",
            "\u{8c01}",
            "\u{54ea}",
        ],
        Script::Latin | Script::Other => &[],
    };
    markers.iter().any(|marker| prompt.contains(marker))
}
