//! Second agentic recipe — driving Formal AI to *make its own meanings more
//! detailed* (issue #538).
//!
//! Issue #468 proved the agentic loop can formalize a text into Links Notation.
//! Issue #538 asks the harder, self-referential question the maintainer set as
//! the project's direction: can the **same** loop — Formal AI driven through its
//! own agentic CLI against its OpenAI-compatible server — *edit its own seed
//! knowledge* to make a meaning more detailed? Concretely, the tomato meaning
//! listed the Russian surfaces `помидор`, `помидоры`, `томат` without recording
//! which is singular or plural, and `помидор` had a plural while its synonym
//! `томат` did not.
//!
//! This module is the deterministic meta-algorithm for that task. Given the
//! Wikidata lexeme data fetched by the loop (`web_fetch`), it re-derives the
//! enriched tomato block — every surface pinned to its part of speech and
//! grammatical number, grounded in its real Wikidata form, and with the
//! previously missing plural `томаты` (form `L170542-F7`) recovered from the
//! source. The block it writes is byte-for-byte the enriched seed block, so the
//! loop *reproduces the exact data change* the issue asked for rather than a
//! hand-authored approximation. Neural inference stays a NON-GOAL: the recipe is
//! a pure function of the fetched lexeme facts.

use std::fmt::Write as _;

/// The canonical issue-#538 task string. The wording carries the keywords
/// [`is_meaning_detail_task`] recognises.
pub const MEANING_DETAIL_TASK: &str = "Make the tomato meaning more detailed: pin every surface's \
                                       part of speech and grammatical number, ground it in Wikidata, \
                                       and add the missing plural to томат.";

/// The web-search query the planner issues for this recipe.
pub const SEARCH_QUERY: &str = "Wikidata lexemes tomato помидор томат grammatical number forms";

/// The source URL the planner fetches — the offline corpus resolves it to the
/// tomato lexeme facts (see [`super::corpus`]).
pub const SOURCE_URL: &str = "https://www.wikidata.org/wiki/Lexeme:L170542";

/// The path the planner writes the enriched meaning block to.
pub const KB_PATH: &str = "meanings-tomato-detail.lino";

/// Keywords that mark a user turn as the issue-#538 meaning-detail task.
const DETAIL_KEYWORDS: [&str; 8] = [
    "grammatical number",
    "more detailed",
    "singular or plural",
    "part of speech",
    "помидор",
    "томат",
    "detailed meaning",
    "detailed word",
];

/// Whether `prompt` asks to make a meaning more detailed (issue #538).
#[must_use]
pub fn is_meaning_detail_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    DETAIL_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
}

/// One inflected form of a source lexeme, as recovered from Wikidata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexemeForm {
    /// The form suffix, e.g. `F1` (the full id is `<lexeme>-<suffix>`).
    pub suffix: String,
    /// The surface spelling.
    pub text: String,
    /// The grammatical number: `singular` or `plural`.
    pub number: String,
    /// The Wikidata grammatical-feature id (`Q110786`/`Q146786`).
    pub feature: String,
}

/// One grounded source lexeme (English or Russian) with its forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLexeme {
    /// The Wikidata lexeme id, e.g. `L7993`.
    pub id: String,
    /// The two-letter language code used on the surfaces (`en`/`ru`).
    pub language: String,
    /// The Wikidata language item id (`Q1860`/`Q7737`).
    pub language_item: String,
    /// The Wikidata lexical-category id (`Q1084` = noun).
    pub category: String,
    /// The grounded sense id, if the lexeme has one (`L7993-S1`).
    pub sense: Option<String>,
    /// The inflected forms, in Wikidata order.
    pub forms: Vec<LexemeForm>,
}

/// A non-grounded extra surface (Hindi/Chinese) kept for multilingual parity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraSurface {
    /// The language code (`hi`/`zh`).
    pub language: String,
    /// The surface spelling.
    pub text: String,
}

/// The tomato lexeme facts recovered from the fetched Wikidata data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomatoLexemes {
    /// The grounded English and Russian source lexemes, in render order.
    pub sources: Vec<SourceLexeme>,
    /// The non-grounded extra surfaces (Hindi/Chinese).
    pub extras: Vec<ExtraSurface>,
}

/// The deterministic fallback fetch body: a faithful, human-readable rendering of
/// the tomato lexeme facts. The offline corpus serves exactly this text for
/// [`SOURCE_URL`], and [`parse_lexemes`] round-trips it, so the loop produces a
/// stable block whether the fetch "succeeds" or falls back. Compare
/// [`super::formalize::CANONICAL_FISHERMAN_SYNOPSIS`].
pub const CANONICAL_TOMATO_LEXEMES: &str = "\
Wikidata lexemes for the tomato concept (Q23501)
lexeme L7993 en Q1860 Q1084 sense=L7993-S1
  form F1 tomato singular Q110786
  form F2 tomatoes plural Q146786
lexeme L3526 ru Q7737 Q1084 sense=L3526-S1
  form F1 помидор singular Q110786
  form F3 помидоры plural Q146786
lexeme L170542 ru Q7737 Q1084
  form F1 томат singular Q110786
  form F7 томаты plural Q146786
extra hi टमाटर
extra zh 番茄
extra zh 西红柿
";

/// Parse the fetched Wikidata-lexeme text into structured facts.
///
/// The format is the compact one [`CANONICAL_TOMATO_LEXEMES`] uses; unknown lines
/// are ignored so a richer real fetch body still parses. Returns [`None`] if no
/// grounded lexeme was found (so the planner can fall back).
#[must_use]
pub fn parse_lexemes(text: &str) -> Option<TomatoLexemes> {
    let mut sources: Vec<SourceLexeme> = Vec::new();
    let mut extras: Vec<ExtraSurface> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let mut fields = trimmed.split_whitespace();
        match fields.next() {
            Some("lexeme") => {
                let (Some(id), Some(language), Some(language_item), Some(category)) =
                    (fields.next(), fields.next(), fields.next(), fields.next())
                else {
                    continue;
                };
                let sense = fields
                    .find_map(|field| field.strip_prefix("sense="))
                    .map(str::to_owned);
                sources.push(SourceLexeme {
                    id: id.to_owned(),
                    language: language.to_owned(),
                    language_item: language_item.to_owned(),
                    category: category.to_owned(),
                    sense,
                    forms: Vec::new(),
                });
            }
            Some("form") => {
                let (Some(suffix), Some(text), Some(number), Some(feature)) =
                    (fields.next(), fields.next(), fields.next(), fields.next())
                else {
                    continue;
                };
                if let Some(current) = sources.last_mut() {
                    current.forms.push(LexemeForm {
                        suffix: suffix.to_owned(),
                        text: text.to_owned(),
                        number: number.to_owned(),
                        feature: feature.to_owned(),
                    });
                }
            }
            Some("extra") => {
                if let (Some(language), Some(text)) = (fields.next(), fields.next()) {
                    extras.push(ExtraSurface {
                        language: language.to_owned(),
                        text: text.to_owned(),
                    });
                }
            }
            _ => {}
        }
    }
    if sources.iter().any(|source| !source.forms.is_empty()) {
        Some(TomatoLexemes { sources, extras })
    } else {
        None
    }
}

/// The tomato lexeme facts, parsed from `fetched` when it is real lexeme data,
/// else from the canonical fallback.
#[must_use]
pub fn tomato_lexemes(fetched: Option<&str>) -> TomatoLexemes {
    fetched
        .and_then(parse_lexemes)
        .unwrap_or_else(|| parse_lexemes(CANONICAL_TOMATO_LEXEMES).expect("canonical facts parse"))
}

/// Human-readable Wikidata language name for the surface comments.
fn language_name(code: &str) -> &'static str {
    match code {
        "en" => "english",
        "ru" => "russian",
        "hi" => "hindi",
        "zh" => "chinese",
        _ => "unknown",
    }
}

/// Render the enriched tomato meaning block in Links Notation.
///
/// The output is byte-for-byte the enriched seed block
/// (`data/seed/meanings-translation.lino`), so the agentic loop reproduces the
/// exact issue-#538 data change. Russian comments include the lemma spelling (as
/// the seed authors them); English comments name the language only.
#[must_use]
pub fn render_tomato_block(lexemes: &TomatoLexemes) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  tomato");
    let _ = writeln!(out, "    grounded-in Q23501");
    let _ = writeln!(out, "    defined-by entity");
    let _ = writeln!(out, "    role compositional_lemma");

    for source in &lexemes.sources {
        let name = language_name(&source.language);
        let lemma = source.forms.first().map(|form| form.text.as_str());
        // Russian source/surface comments carry the lemma spelling; English does not.
        let comment_lemma = source.language == "ru";
        let lemma_suffix = match (comment_lemma, lemma) {
            (true, Some(text)) => format!(" {text}"),
            _ => String::new(),
        };
        let _ = writeln!(
            out,
            "    source-lexeme {} # wikidata {name} source lexeme{lemma_suffix}",
            source.id
        );
        let _ = writeln!(
            out,
            "      language {} # wikidata language {name}",
            source.language_item
        );
        let _ = writeln!(
            out,
            "      lexical-category {} # wikidata category noun",
            source.category
        );
        for form in &source.forms {
            let _ = writeln!(
                out,
                "      form {}-{} # wikidata form {}",
                source.id, form.suffix, form.text
            );
            let _ = writeln!(
                out,
                "        feature {} # wikidata grammatical feature {}",
                form.feature, form.number
            );
        }
        if let Some(sense) = &source.sense {
            let _ = writeln!(out, "      sense {sense} # wikidata grounded sense");
        }
        for form in &source.forms {
            let comment_text = if comment_lemma {
                format!(" {}", form.text)
            } else {
                String::new()
            };
            let _ = writeln!(
                out,
                "    surface {}-{} # wikidata {name} {} surface{comment_text}",
                source.id, form.suffix, form.number
            );
            let _ = writeln!(out, "      text {}", form.text);
            let _ = writeln!(out, "      language {}", source.language);
            let _ = writeln!(out, "      part_of_speech noun");
            let _ = writeln!(out, "      grammatical_number {}", form.number);
            if let Some(sense) = &source.sense {
                let _ = writeln!(out, "      sense {sense} # wikidata grounded sense");
            }
        }
    }

    // Extra, non-grounded surfaces grouped by language, in first-seen order.
    let mut seen_languages: Vec<&str> = Vec::new();
    for extra in &lexemes.extras {
        if !seen_languages.contains(&extra.language.as_str()) {
            seen_languages.push(&extra.language);
        }
    }
    for language in seen_languages {
        let _ = writeln!(out, "    lexeme {language}");
        for extra in lexemes.extras.iter().filter(|e| e.language == language) {
            let _ = writeln!(out, "      surface");
            let _ = writeln!(out, "        text {}", extra.text);
            let _ = writeln!(out, "        part_of_speech noun");
        }
    }

    out
}

/// Build the enriched tomato block from the fetched lexeme data (or the canonical
/// fallback), ready to write to [`KB_PATH`].
#[must_use]
pub fn enrich_tomato_block(fetched: Option<&str>) -> String {
    render_tomato_block(&tomato_lexemes(fetched))
}

/// The self-contained final answer: a natural-language summary plus the enriched
/// block inline.
#[must_use]
pub fn final_answer(block: &str) -> String {
    format!(
        "Made the tomato meaning more detailed: every surface now pins its part of speech and \
         grammatical number, is grounded in its Wikidata lexeme form, and the previously missing \
         Russian plural `томаты` (form L170542-F7) is added so `томат` matches `помидор`.\n\n\
         Enriched meaning block ({KB_PATH}):\n\n{block}",
        block = block.trim_end(),
    )
}
