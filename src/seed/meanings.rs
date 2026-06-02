//! Language-independent *meaning* lexicon (issue #386).
//!
//! The solver must never recognise a user prompt by matching a hardcoded list
//! of words in one language. Instead it references **meanings** — concepts that
//! exist independently of any language and are *self-describing*: every meaning
//! is `defined_by` other meanings (a closed, mutually-referential graph in the
//! spirit of <https://github.com/link-foundation/relative-meta-logic>), carries
//! a human `gloss`, and is anchored to real lexical data via `wiktionary`.
//!
//! A meaning declares the semantic `role`s it can play when read in a prompt
//! (e.g. a sort is both a `program_artifact` a follow-up can refer to and a
//! `program_modification` a follow-up can request). Its `lexeme` blocks list
//! the surface words that *evidence* it, per language. Recognition code asks
//! the lexicon "which words evidence role X?" and stays free of hardcoded
//! natural-language text — the words live once, here in the data.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::parser::{parse_lino, LinoNode};
use super::MEANING_FILES;

/// Semantic role: a thing a program produces that a later turn can refer back
/// to (a result, an output, the program/script/code itself, an ordering).
pub const ROLE_PROGRAM_ARTIFACT: &str = "program_artifact";
/// Semantic role: an operation a follow-up turn can request against the active
/// program (sort, reverse, cancel, change, …) — additive or subtractive.
pub const ROLE_PROGRAM_MODIFICATION: &str = "program_modification";
/// Semantic role: a kind of program artifact a user can ask to be authored
/// (a program, a script, code, a function). The noun side of "write a <kind>".
pub const ROLE_PROGRAM_KIND: &str = "program_kind";
/// Semantic role: a verb that requests a program artifact be produced (write,
/// create, show, generate, make, build). The verb side of "write a <kind>".
pub const ROLE_PROGRAM_REQUEST: &str = "program_request";
/// Semantic role: a concrete unit of measurement (metre, byte, kilogram, …).
/// Each such meaning is `defined_by` the [`ROLE_PHYSICAL_DIMENSION`] it measures.
pub const ROLE_MEASUREMENT_UNIT: &str = "measurement_unit";
/// Semantic role: a physical dimension (length, mass, time, …). Units that
/// belong to different dimensions cannot be converted into one another.
pub const ROLE_PHYSICAL_DIMENSION: &str = "physical_dimension";
/// Semantic role: a named day of the week (Monday … Sunday). The meaning slug
/// is the English weekday name so a handler can resolve a matched lexeme back
/// to a position in the seven-day cycle.
pub const ROLE_CALENDAR_WEEKDAY: &str = "calendar_weekday";
/// Semantic role: the "comes after" relation between weekdays — a +1 step in
/// the seven-day cycle (after, next day, после, के बाद, 之后, …).
pub const ROLE_CALENDAR_DIRECTION_NEXT: &str = "calendar_direction_next";
/// Semantic role: the "comes before" relation between weekdays — a -1 step in
/// the seven-day cycle (before, previous day, перед, से पहले, 之前, …).
pub const ROLE_CALENDAR_DIRECTION_PREVIOUS: &str = "calendar_direction_previous";
/// Semantic role: the present day relative to the system clock (today,
/// сегодня, आज, 今天). Distinguishes a "what day is it now?" query.
pub const ROLE_CALENDAR_TODAY: &str = "calendar_today";
/// Semantic role: a reference to a day, date, or week — the noun a calendar
/// question is about (day, weekday, date, week, день, неделя, 星期, …).
pub const ROLE_CALENDAR_DAY_REFERENCE: &str = "calendar_day_reference";
/// Semantic role: an interrogative or imperative asking which day (what,
/// which, какой, कौन, 什么, …). The question side of a calendar query.
pub const ROLE_CALENDAR_QUESTION: &str = "calendar_question";
/// Semantic role: a relation in the knowledge base that maps a subject to a
/// value (capital, population, author, …).
///
/// A fact query detects which relation a prompt asks about by walking every
/// meaning carrying this role, in declaration order; each is `defined_by` the
/// `knowledge_relation` concept.
pub const ROLE_FACT_RELATION: &str = "fact_relation";

/// Surface words that evidence a meaning in one language.
#[derive(Debug, Clone)]
pub struct Lexeme {
    pub language: String,
    pub words: Vec<String>,
}

/// A language-independent meaning grounded in real lexical data.
#[derive(Debug, Clone)]
pub struct Meaning {
    pub slug: String,
    pub gloss: String,
    pub wiktionary: String,
    pub defined_by: Vec<String>,
    pub roles: Vec<String>,
    pub lexemes: Vec<Lexeme>,
}

impl Meaning {
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Every surface word across every language this meaning lexicalises.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        self.lexemes
            .iter()
            .flat_map(|lexeme| lexeme.words.iter().map(String::as_str))
    }

    /// The first surface word this meaning lexicalises in `language`, if any.
    /// Used to render a concept in a chosen language (e.g. a dimension label).
    #[must_use]
    pub fn word_in(&self, language: &str) -> Option<&str> {
        self.lexemes
            .iter()
            .find(|lexeme| lexeme.language == language)
            .and_then(|lexeme| lexeme.words.first().map(String::as_str))
    }

    /// Languages this meaning is lexicalised in (used by coverage tests).
    #[must_use]
    pub fn languages(&self) -> BTreeSet<String> {
        self.lexemes.iter().map(|l| l.language.clone()).collect()
    }
}

/// The parsed set of meanings.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    pub meanings: Vec<Meaning>,
}

impl Lexicon {
    #[must_use]
    pub fn meaning(&self, slug: &str) -> Option<&Meaning> {
        self.meanings.iter().find(|m| m.slug == slug)
    }

    /// Every meaning carrying `role`, in declaration order. Lets recognition
    /// code walk a semantic category (e.g. every measurement unit) without ever
    /// naming the surface words — those live in the data.
    pub fn meanings_with_role<'a>(&'a self, role: &'a str) -> impl Iterator<Item = &'a Meaning> {
        self.meanings.iter().filter(move |m| m.has_role(role))
    }

    /// Distinct surface words contributed by every meaning carrying `role`,
    /// in declaration order. Useful for diagnostics and tests.
    #[must_use]
    pub fn words_for_role(&self, role: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for meaning in self.meanings.iter().filter(|m| m.has_role(role)) {
            for word in meaning.words() {
                if !out.iter().any(|existing| existing == word) {
                    out.push(word.to_string());
                }
            }
        }
        out
    }

    /// Does `normalized` mention any surface word of any meaning in `role`?
    ///
    /// Mirrors the CJK-substring vs. whitespace-token contract used across the
    /// solver: CJK scripts have no inter-word spaces, so a CJK surface word is
    /// matched as a substring, while space-delimited scripts match a whole
    /// whitespace token (see [`crate::coding::contains_cjk`]).
    #[must_use]
    pub fn mentions_role(&self, role: &str, normalized: &str) -> bool {
        self.meanings
            .iter()
            .filter(|m| m.has_role(role))
            .flat_map(Meaning::words)
            .any(|word| token_present(normalized, word))
    }
}

fn token_present(normalized: &str, expected: &str) -> bool {
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized.split_whitespace().any(|token| token == expected)
}

fn parse_lexicon(text: &str) -> Lexicon {
    let root = parse_lino(text);
    // The lexicon is split across several files (program, units, …), each
    // wrapping its records under a top-level `meanings` node. When the files
    // are concatenated the document therefore holds one-or-more `meanings`
    // containers; collect the records from every one. If none is present the
    // records sit at the document root (kept for robustness).
    let mut meanings = Vec::new();
    let containers: Vec<&LinoNode> = root
        .children
        .iter()
        .filter(|c| c.name == "meanings")
        .collect();
    let sources: Vec<&LinoNode> = if containers.is_empty() {
        vec![&root]
    } else {
        containers
    };
    for container in sources {
        for node in container.children.iter().filter(|c| c.name == "meaning") {
            meanings.push(parse_meaning(node));
        }
    }
    Lexicon { meanings }
}

fn parse_meaning(node: &LinoNode) -> Meaning {
    let mut defined_by = Vec::new();
    let mut roles = Vec::new();
    let mut lexemes = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "defined_by" => defined_by.push(child.id.clone()),
            "role" => roles.push(child.id.clone()),
            "lexeme" => {
                let words = child
                    .children
                    .iter()
                    .filter(|w| w.name == "word")
                    .map(|w| w.id.clone())
                    .collect();
                lexemes.push(Lexeme {
                    language: child.id.clone(),
                    words,
                });
            }
            _ => {}
        }
    }
    Meaning {
        slug: node.id.clone(),
        gloss: node.find_child_value("gloss").to_string(),
        wiktionary: node.find_child_value("wiktionary").to_string(),
        defined_by,
        roles,
        lexemes,
    }
}

/// The parsed meaning lexicon. Cached — the embedded data is immutable at
/// runtime, so parsing happens at most once per process.
#[must_use]
pub fn lexicon() -> &'static Lexicon {
    static CACHE: OnceLock<Lexicon> = OnceLock::new();
    CACHE.get_or_init(|| parse_lexicon(&MEANING_FILES.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The supported languages every meaning must lexicalise so that the
    /// concept truly is "translatable to any language" (issue #386).
    const SUPPORTED_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

    #[test]
    fn lexicon_is_non_empty_and_well_formed() {
        let lex = lexicon();
        assert!(lex.meanings.len() >= 10, "expected a real lexicon");
        for meaning in &lex.meanings {
            assert!(!meaning.slug.is_empty(), "meaning needs a slug");
            assert!(
                !meaning.gloss.trim().is_empty(),
                "{} needs a conceptual gloss",
                meaning.slug
            );
            assert!(
                !meaning.wiktionary.trim().is_empty(),
                "{} must be grounded in real lexical data (wiktionary)",
                meaning.slug
            );
            assert!(
                !meaning.roles.is_empty(),
                "{} must declare at least one semantic role",
                meaning.slug
            );
        }
    }

    #[test]
    fn every_meaning_is_self_describing() {
        // relative-meta-logic: each term is defined using other terms. The
        // `defined_by` graph must therefore be closed — every reference
        // resolves to another defined meaning (cycles are allowed and
        // expected; there are no undefined primitives).
        let lex = lexicon();
        let slugs: BTreeSet<&str> = lex.meanings.iter().map(|m| m.slug.as_str()).collect();
        for meaning in &lex.meanings {
            assert!(
                !meaning.defined_by.is_empty(),
                "{} must be defined by other meanings",
                meaning.slug
            );
            for target in &meaning.defined_by {
                assert!(
                    slugs.contains(target.as_str()),
                    "{} is defined_by `{target}`, which is not itself a defined meaning",
                    meaning.slug
                );
            }
        }
    }

    #[test]
    fn every_meaning_covers_all_supported_languages() {
        let lex = lexicon();
        for meaning in &lex.meanings {
            let languages = meaning.languages();
            for language in SUPPORTED_LANGUAGES {
                assert!(
                    languages.contains(language),
                    "{} is missing a `{language}` lexeme — meanings must translate to every supported language",
                    meaning.slug
                );
            }
            for lexeme in &meaning.lexemes {
                assert!(
                    !lexeme.words.is_empty(),
                    "{} / {} lexeme must list at least one surface word",
                    meaning.slug,
                    lexeme.language
                );
            }
        }
    }

    #[test]
    fn program_roles_are_populated() {
        let lex = lexicon();
        assert!(
            !lex.words_for_role(ROLE_PROGRAM_ARTIFACT).is_empty(),
            "program_artifact role must have surface words"
        );
        assert!(
            !lex.words_for_role(ROLE_PROGRAM_MODIFICATION).is_empty(),
            "program_modification role must have surface words"
        );
    }

    #[test]
    fn mentions_role_honours_cjk_and_token_boundaries() {
        let lex = lexicon();
        // Whitespace token (Russian): a substring of a longer token must NOT
        // match, but the standalone token must.
        assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отмени сортировку"));
        assert!(!lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
        // CJK substring: matches inside a space-free run.
        assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "取消排序"));
        assert!(lex.mentions_role(ROLE_PROGRAM_ARTIFACT, "取消排序"));
    }
}
