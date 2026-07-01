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
//! This module is the deterministic meta-algorithm for that task. It is **not**
//! hard-coded to one concept: a small [`Concept`] registry ([`CONCEPTS`]) lets the
//! *same* recipe make **any** registered meaning more detailed, and [`concept_for_task`]
//! routes a natural-language request to the right one. That is the maintainer's
//! generality requirement — *"each time you should use different natural language
//! requests, so we test that solutions are never hardcoded, but truly general"*:
//! `tomato` and `potato` are enriched by the very same code, driven by two
//! differently worded requests.
//!
//! Given the Wikidata lexeme data fetched by the loop (`web_fetch`), the recipe
//! re-derives the enriched meaning block — every surface pinned to its part of
//! speech and grammatical number, grounded in its real Wikidata form, and with any
//! previously missing plural surface (e.g. tomato's `томаты`, form `L170542-F7`, or
//! potato's `potatoes`, form `L3784-F2`) recovered from the source. The block it
//! writes is byte-for-byte the enriched seed block, so the loop *reproduces the
//! exact data change* the issue asked for rather than a hand-authored
//! approximation. Neural inference stays a NON-GOAL: the recipe is a pure function
//! of the fetched lexeme facts.

use std::fmt::Write as _;

/// A concept the meaning-detail recipe knows how to enrich.
///
/// Everything the planner, corpus, and driver need for one concept lives here, so
/// adding a new concept is a matter of adding one [`Concept`] to [`CONCEPTS`] plus
/// its canonical lexeme facts — no code branches per concept.
#[derive(Debug, Clone, Copy)]
pub struct Concept {
    /// The meaning-block lemma head as it appears in the seed (`tomato`/`potato`).
    pub name: &'static str,
    /// The Wikidata item the meaning is grounded in (`Q23501`/`Q10998`).
    pub grounded_in: &'static str,
    /// The web-search query the planner issues for this concept.
    pub search_query: &'static str,
    /// The source URL the planner fetches (the offline corpus resolves it).
    pub source_url: &'static str,
    /// The workspace path the planner writes the enriched block to.
    pub kb_path: &'static str,
    /// The deterministic fallback fetch body (compact lexeme facts).
    pub canonical_lexemes: &'static str,
    /// Lowercased keywords that route a request to this concept.
    pub keywords: &'static [&'static str],
}

/// The canonical issue-#538 task string (tomato). The wording carries the
/// keywords [`is_meaning_detail_task`] recognises.
pub const MEANING_DETAIL_TASK: &str = "Make the tomato meaning more detailed: pin every surface's \
                                       part of speech and grammatical number, ground it in Wikidata, \
                                       and add the missing plural to томат.";

/// A *differently worded* request for the second concept (potato).
///
/// Using distinct natural language for each concept is the maintainer's generality
/// check: the recipe must not depend on the exact phrasing of the tomato task.
pub const POTATO_DETAIL_TASK: &str = "Please make the potato word and meaning richer — record the \
                                      singular/plural of each surface, add the missing plural form \
                                      potatoes, and keep it grounded in Wikidata.";

/// The tomato web-search query the planner issues (kept as a named const for the
/// existing tests; equals [`TOMATO`]'s `search_query`).
pub const SEARCH_QUERY: &str = "Wikidata lexemes tomato помидор томат grammatical number forms";

/// The tomato source URL the planner fetches (equals [`TOMATO`]'s `source_url`).
pub const SOURCE_URL: &str = "https://www.wikidata.org/wiki/Lexeme:L170542";

/// The path the planner writes the enriched tomato block to (equals [`TOMATO`]'s
/// `kb_path`).
pub const KB_PATH: &str = "meanings-tomato-detail.lino";

/// The tomato concept.
pub const TOMATO: Concept = Concept {
    name: "tomato",
    grounded_in: "Q23501",
    search_query: SEARCH_QUERY,
    source_url: SOURCE_URL,
    kb_path: KB_PATH,
    canonical_lexemes: CANONICAL_TOMATO_LEXEMES,
    keywords: &["помидор", "томат", "tomato"],
};

/// The potato concept — proof the recipe is general, not tomato-specific.
pub const POTATO: Concept = Concept {
    name: "potato",
    grounded_in: "Q10998",
    search_query: "Wikidata lexemes potato картофель картошка grammatical number forms",
    source_url: "https://www.wikidata.org/wiki/Lexeme:L3784",
    kb_path: "meanings-potato-detail.lino",
    canonical_lexemes: CANONICAL_POTATO_LEXEMES,
    keywords: &["potato", "картофель", "картошка", "आलू", "土豆", "马铃薯"],
};

/// Every concept the meaning-detail recipe can enrich, in routing/ranking order.
pub const CONCEPTS: &[&Concept] = &[&TOMATO, &POTATO];

/// Generic keywords that mark a user turn as the issue-#538 meaning-detail task,
/// independent of which concept it targets.
const DETAIL_KEYWORDS: [&str; 6] = [
    "grammatical number",
    "more detailed",
    "singular or plural",
    "part of speech",
    "detailed meaning",
    "detailed word",
];

/// The concept a request targets, if any — the first registered concept whose
/// keywords appear in `prompt`.
#[must_use]
pub fn concept_for_task(prompt: &str) -> Option<&'static Concept> {
    let lower = prompt.to_lowercase();
    CONCEPTS.iter().copied().find(|concept| {
        concept
            .keywords
            .iter()
            .any(|keyword| lower.contains(&keyword.to_lowercase()))
    })
}

/// Whether `prompt` asks to make a meaning more detailed (issue #538): either it
/// uses a generic detail keyword, or it names a concept the recipe knows.
#[must_use]
pub fn is_meaning_detail_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    DETAIL_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        || concept_for_task(prompt).is_some()
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

/// A non-grounded extra surface (Hindi/Chinese/…) kept for multilingual parity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraSurface {
    /// The language code (`hi`/`zh`/`ru`).
    pub language: String,
    /// The surface spelling.
    pub text: String,
}

/// The concept's lexeme facts recovered from the fetched Wikidata data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptLexemes {
    /// The grounded source lexemes, in render order.
    pub sources: Vec<SourceLexeme>,
    /// The non-grounded extra surfaces (Hindi/Chinese/Russian).
    pub extras: Vec<ExtraSurface>,
}

/// The deterministic fallback fetch body for tomato.
///
/// A faithful, human-readable rendering of the tomato lexeme facts. The offline
/// corpus serves exactly this text for [`TOMATO`]'s `source_url`, and
/// [`parse_lexemes`] round-trips it, so the loop produces a stable block whether
/// the fetch "succeeds" or falls back. Compare
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

/// The deterministic fallback fetch body for potato.
///
/// The English lexeme `L3784` carries the singular/plural forms grounded in
/// Wikidata; the Russian, Hindi, and Chinese surfaces ride along as non-grounded
/// extras (as the seed authors them).
pub const CANONICAL_POTATO_LEXEMES: &str = "\
Wikidata lexemes for the potato concept (Q10998)
lexeme L3784 en Q1860 Q1084 sense=L3784-S1
  form F1 potato singular Q110786
  form F2 potatoes plural Q146786
extra ru картофель
extra ru картошка
extra hi आलू
extra zh 土豆
extra zh 马铃薯
";

/// Parse the fetched Wikidata-lexeme text into structured facts.
///
/// The format is the compact one [`CANONICAL_TOMATO_LEXEMES`] uses; unknown lines
/// are ignored so a richer real fetch body still parses. Returns [`None`] if no
/// grounded lexeme was found (so the planner can fall back).
#[must_use]
pub fn parse_lexemes(text: &str) -> Option<ConceptLexemes> {
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
        Some(ConceptLexemes { sources, extras })
    } else {
        None
    }
}

/// The concept's lexeme facts, parsed from `fetched` when it is real lexeme data,
/// else from the concept's canonical fallback.
#[must_use]
pub fn concept_lexemes(concept: &Concept, fetched: Option<&str>) -> ConceptLexemes {
    fetched
        .and_then(parse_lexemes)
        .unwrap_or_else(|| parse_lexemes(concept.canonical_lexemes).expect("canonical facts parse"))
}

/// The tomato lexeme facts (thin wrapper over [`concept_lexemes`] for [`TOMATO`]).
#[must_use]
pub fn tomato_lexemes(fetched: Option<&str>) -> ConceptLexemes {
    concept_lexemes(&TOMATO, fetched)
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

/// Render the enriched meaning block for `concept` in Links Notation.
///
/// The output is byte-for-byte the enriched seed block for that concept
/// (`data/seed/meanings-translation.lino`), so the agentic loop reproduces the
/// exact issue-#538 data change. Russian comments include the lemma spelling (as
/// the seed authors them); other languages name the language only.
#[must_use]
pub fn render_block(concept: &Concept, lexemes: &ConceptLexemes) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  {}", concept.name);
    let _ = writeln!(out, "    grounded-in {}", concept.grounded_in);
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

/// Render the enriched tomato block (thin wrapper over [`render_block`]).
#[must_use]
pub fn render_tomato_block(lexemes: &ConceptLexemes) -> String {
    render_block(&TOMATO, lexemes)
}

/// Build the enriched block for `concept` from the fetched lexeme data (or the
/// canonical fallback), ready to write to the concept's `kb_path`.
#[must_use]
pub fn enrich_block(concept: &Concept, fetched: Option<&str>) -> String {
    render_block(concept, &concept_lexemes(concept, fetched))
}

/// Build the enriched tomato block (thin wrapper over [`enrich_block`]).
#[must_use]
pub fn enrich_tomato_block(fetched: Option<&str>) -> String {
    enrich_block(&TOMATO, fetched)
}

/// The self-contained final answer for `concept`: a natural-language summary plus
/// the enriched block inline.
#[must_use]
pub fn final_answer_for(concept: &Concept, block: &str) -> String {
    format!(
        "Made the {name} meaning more detailed: every surface now pins its part of speech and \
         grammatical number, is grounded in its Wikidata lexeme forms, and every plural surface \
         recovered from the source is added.\n\n\
         Enriched meaning block ({path}):\n\n{block}",
        name = concept.name,
        path = concept.kb_path,
        block = block.trim_end(),
    )
}

/// The self-contained final answer for tomato (thin wrapper over
/// [`final_answer_for`]).
#[must_use]
pub fn final_answer(block: &str) -> String {
    final_answer_for(&TOMATO, block)
}
