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
use super::roles::{ROLE_ONTOLOGY_CATEGORY, ROLE_ONTOLOGY_ROOT, ROLE_ONTOLOGY_TYPE};
use super::MEANING_FILES;

/// Where a surface form positions the variable subject of a templated prompt.
///
/// A meaning's surface text may be a fixed phrase or a template with one open
/// slot — the position a user fills with the concrete subject ("how does *X*
/// work"). The slot is marked in the data with a single ellipsis `…` (U+2026,
/// serializer-safe: not a quote or backslash), and its position classifies the
/// form:
///
/// * [`Slot::Bare`] — no `…`: a fixed phrase carrying no subject ("how it works").
/// * [`Slot::Prefix`] — trailing `…`: the literal precedes the subject, which
///   follows ("how does …" → subject after).
/// * [`Slot::Suffix`] — leading `…`: the subject precedes the literal ("… как
///   работает" → subject before).
/// * [`Slot::Circumfix`] — middle `…`: the subject sits between two literals
///   ("how … works").
///
/// This lets recognition code derive an affix-matching strategy from the data
/// rather than from a hardcoded per-language list (issue #386).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// A fixed phrase with no open subject slot.
    Bare,
    /// The literal precedes the subject (trailing `…`).
    Prefix,
    /// The subject precedes the literal (leading `…`).
    Suffix,
    /// The subject sits between two literals (middle `…`).
    Circumfix,
}

/// A single surface form together with a self-describing note.
///
/// The data must "not just list" the word (issue #386): each form carries a
/// `description` — a concise human note of *that* form's sense and shape (its
/// citation form, grammatical form, script, or romanisation). Recognition code
/// still matches on [`WordForm::text`]; the description makes the form usable
/// for any purpose (diagnostics, learning, lexical grounding), not only the
/// one parsing path baked into a handler.
///
/// A form may also carry an `action` — the canonical, language-independent
/// operation it names when it stands in for a verb (e.g. the procedural surface
/// "как сделать …" names the `do` action). Empty when the form does not fix an
/// action (the operation is then read from the matched subject instead).
#[derive(Debug, Clone)]
pub struct WordForm {
    pub text: String,
    pub description: String,
    pub action: String,
}

impl WordForm {
    /// How this form positions its subject slot — derived from the position of
    /// the `…` (U+2026) marker in [`WordForm::text`]. A form with no marker is
    /// [`Slot::Bare`]; see [`Slot`] for the full classification.
    #[must_use]
    pub fn slot(&self) -> Slot {
        match self.text.split_once('…') {
            None => Slot::Bare,
            Some((before, after)) => match (!before.is_empty(), !after.is_empty()) {
                (true, true) => Slot::Circumfix,
                (true, false) => Slot::Prefix,
                (false, true) => Slot::Suffix,
                (false, false) => Slot::Bare,
            },
        }
    }

    /// The literal text before the `…` slot marker (the whole text when there is
    /// no marker). For a [`Slot::Prefix`] form this is the matchable prefix.
    #[must_use]
    pub fn before_slot(&self) -> &str {
        match self.text.split_once('…') {
            Some((before, _)) => before,
            None => &self.text,
        }
    }

    /// The literal text after the `…` slot marker (empty when there is no
    /// marker). For a [`Slot::Suffix`] form this is the matchable suffix.
    #[must_use]
    pub fn after_slot(&self) -> &str {
        match self.text.split_once('…') {
            Some((_, after)) => after,
            None => "",
        }
    }
}

/// Surface forms that evidence a meaning in one language.
#[derive(Debug, Clone)]
pub struct Lexeme {
    pub language: String,
    pub words: Vec<WordForm>,
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
            .flat_map(|lexeme| lexeme.words.iter().map(|w| w.text.as_str()))
    }

    /// Every surface form (text plus its self-describing note) across every
    /// language this meaning lexicalises, in declaration order.
    pub fn word_forms(&self) -> impl Iterator<Item = &WordForm> {
        self.lexemes.iter().flat_map(|lexeme| lexeme.words.iter())
    }

    /// Is this meaning evidenced in `normalized` — does any of its surface
    /// words (in any language) appear as a whole token or phrase? Matching is
    /// not language-gated: an English proper noun (e.g. `python`) is evidence
    /// in a prompt written in any language.
    #[must_use]
    pub fn evidenced_in(&self, normalized: &str) -> bool {
        self.words().any(|word| surface_present(normalized, word))
    }

    /// The first surface word this meaning lexicalises in `language`, if any.
    /// Used to render a concept in a chosen language (e.g. a dimension label).
    #[must_use]
    pub fn word_in(&self, language: &str) -> Option<&str> {
        self.lexemes
            .iter()
            .find(|lexeme| lexeme.language == language)
            .and_then(|lexeme| lexeme.words.first().map(|w| w.text.as_str()))
    }

    /// The self-describing note for `word` (matched case-insensitively against
    /// the stored surface text) in any language, if recorded. This is the live
    /// reader that makes [`WordForm::description`] usable — the data describes
    /// each form rather than merely listing it (issue #386).
    #[must_use]
    pub fn describe_word(&self, word: &str) -> Option<&str> {
        self.word_forms()
            .find(|form| form.text.eq_ignore_ascii_case(word))
            .map(|form| form.description.as_str())
    }

    /// Languages this meaning is lexicalised in (used by coverage tests).
    #[must_use]
    pub fn languages(&self) -> BTreeSet<String> {
        self.lexemes.iter().map(|l| l.language.clone()).collect()
    }

    /// Does any surface form this meaning lexicalises in one of `languages`
    /// appear in `normalized` as a raw substring (`str::contains`)?
    ///
    /// The language-restricted, raw-substring sibling of [`Meaning::evidenced_in`].
    /// Feature-capability recognition matches a feature's multilingual aliases by
    /// raw substring — punctuation is preserved, so whole-token boundaries do not
    /// hold — and only in the prompt's own language plus English, so it queries
    /// this rather than the token-bounded [`evidenced_in`](Self::evidenced_in).
    /// The surface words stay in the data; only the language codes (the legitimate
    /// code-resident bridge) and the raw-substring contract live in the caller.
    #[must_use]
    pub fn mentions_in_languages_raw(&self, normalized: &str, languages: &[&str]) -> bool {
        self.lexemes
            .iter()
            .filter(|lexeme| languages.contains(&lexeme.language.as_str()))
            .flat_map(|lexeme| lexeme.words.iter())
            .any(|word| !word.text.is_empty() && normalized.contains(word.text.as_str()))
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

    /// Every surface *form* (text, description, action, slot) contributed by
    /// every meaning carrying `role`, in declaration order. Unlike
    /// [`Lexicon::words_for_role`] this preserves each form's slot marker and
    /// action, so a handler can derive an affix-matching strategy from the data:
    /// it walks the forms, buckets them by [`WordForm::slot`], and matches each
    /// against the prompt — never naming a surface word itself (issue #386).
    #[must_use]
    pub fn role_word_forms<'a>(&'a self, role: &str) -> Vec<&'a WordForm> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .flat_map(Meaning::word_forms)
            .collect()
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
    /// whitespace token or phrase (see [`crate::coding::contains_cjk`]).
    #[must_use]
    pub fn mentions_role(&self, role: &str, normalized: &str) -> bool {
        self.meanings_with_role(role)
            .any(|meaning| meaning.evidenced_in(normalized))
    }

    /// Does `normalized` contain any surface word of any meaning in `role` as a
    /// raw substring (`str::contains`), ignoring whitespace-token boundaries?
    ///
    /// This is the deliberately *looser* sibling of [`mentions_role`]. Many
    /// legacy recognisers matched an inflectable stem — `правил` to catch
    /// `правила`/`правило`/`правил`, `расчёт` to catch `при расчёте` — by raw
    /// substring. Those stems are not whole tokens, so [`mentions_role`]'s
    /// token-bounded contract would miss them. A meaning whose surface forms are
    /// such stems (recorded as [`Slot::Bare`] words) is queried through this
    /// method instead, preserving the original byte-faithful substring match
    /// while still keeping the surface words in the data, not the code. Slot
    /// markers are not stripped, so author stem roles as bare forms.
    #[must_use]
    pub fn mentions_role_raw(&self, role: &str, normalized: &str) -> bool {
        self.meanings_with_role(role)
            .any(|meaning| meaning.words().any(|word| normalized.contains(word)))
    }

    /// Distinct surface words contributed by every meaning carrying `role`,
    /// limited to the given `languages`, in declaration order.
    ///
    /// Lets a handler partition a role's vocabulary by linguistic typology — for
    /// the translation request-gate, the head-initial English/Russian command
    /// stems (matched clause-initially) versus the head-final Hindi/Chinese stems
    /// (matched anywhere, gated by a target marker) — while keeping every surface
    /// word in the data. Language codes are the legitimate code-resident bridge
    /// (see [`crate::translation::language_markers`]); the words stay in the seed.
    #[must_use]
    pub fn words_for_role_in_languages(&self, role: &str, languages: &[&str]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for meaning in self.meanings_with_role(role) {
            for lexeme in &meaning.lexemes {
                if !languages.contains(&lexeme.language.as_str()) {
                    continue;
                }
                for word in &lexeme.words {
                    if !out.iter().any(|existing| existing == &word.text) {
                        out.push(word.text.clone());
                    }
                }
            }
        }
        out
    }

    /// The first language in `priority` whose surface word for `role` appears in
    /// `normalized` (raw substring), or `None` when none is present.
    ///
    /// Answers "which language did the user issue this command in?" — the
    /// source-inferencer reads a translation command's verb language as the
    /// language of the prompt itself. Priority order resolves ties (a prompt that
    /// happens to carry stems from several languages takes the first listed).
    /// Language codes are the legitimate code-resident bridge; the surface words
    /// stay in the data.
    #[must_use]
    pub fn first_role_language(
        &self,
        role: &str,
        normalized: &str,
        priority: &[&'static str],
    ) -> Option<&'static str> {
        priority.iter().copied().find(|&lang| {
            self.meanings_with_role(role).any(|meaning| {
                meaning
                    .lexemes
                    .iter()
                    .filter(|lexeme| lexeme.language == lang)
                    .any(|lexeme| {
                        lexeme
                            .words
                            .iter()
                            .any(|word| normalized.contains(word.text.as_str()))
                    })
            })
        })
    }

    /// The first meaning carrying `role`, in declaration order, that is
    /// evidenced in `normalized` — or `None`. Declaration order therefore
    /// encodes priority (e.g. the first matching delivery mode wins).
    #[must_use]
    pub fn first_role_match(&self, role: &str, normalized: &str) -> Option<&Meaning> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .find(|meaning| meaning.evidenced_in(normalized))
    }

    /// The first meaning carrying `role`, in declaration order, that mentions one
    /// of its `languages` surface forms in `normalized` as a raw substring — or
    /// `None`.
    ///
    /// The raw-substring, language-restricted sibling of [`first_role_match`](Self::first_role_match).
    /// Declaration order encodes priority, so the feature-capability recogniser
    /// lists its alias meanings in the legacy table order and takes the first hit,
    /// querying the prompt's own language plus English without ever naming a
    /// surface word in code.
    #[must_use]
    pub fn first_role_match_in_languages_raw(
        &self,
        role: &str,
        normalized: &str,
        languages: &[&str],
    ) -> Option<&Meaning> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .find(|meaning| meaning.mentions_in_languages_raw(normalized, languages))
    }

    /// Does any meaning carrying `role` mention one of its `languages` surface
    /// forms in `normalized` as a raw substring?
    ///
    /// The boolean, language-restricted sibling of [`mentions_role_raw`](Self::mentions_role_raw).
    /// The feature-capability question gate uses it to check each language's
    /// interrogative cues only against prompts detected in that language.
    #[must_use]
    pub fn mentions_role_in_languages_raw(
        &self,
        role: &str,
        normalized: &str,
        languages: &[&str],
    ) -> bool {
        self.meanings_with_role(role)
            .any(|meaning| meaning.mentions_in_languages_raw(normalized, languages))
    }

    /// The single meaning that roots the merged ontology — the one carrying
    /// [`ROLE_ONTOLOGY_ROOT`] (the `link` meaning), or `None` if absent.
    #[must_use]
    pub fn ontology_root(&self) -> Option<&Meaning> {
        self.meanings
            .iter()
            .find(|m| m.has_role(ROLE_ONTOLOGY_ROOT))
    }

    /// Does `slug` reach the ontology root by following `defined_by` edges?
    ///
    /// A breadth-first walk of the `defined_by` graph that visits each meaning
    /// at most once (cycles are expected). Every meaning must reach the root, so
    /// the data forms one connected ontology rather than disjoint islands of
    /// vocabulary — the universal "everything reduces to a link" stance.
    #[must_use]
    pub fn reaches_root(&self, slug: &str) -> bool {
        let Some(root) = self.ontology_root() else {
            return false;
        };
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut stack: Vec<&str> = vec![slug];
        while let Some(current) = stack.pop() {
            if current == root.slug {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            if let Some(meaning) = self.meaning(current) {
                for target in &meaning.defined_by {
                    stack.push(target.as_str());
                }
            }
        }
        false
    }

    /// The type-system sub-root of the ontology — the meaning carrying
    /// [`ROLE_ONTOLOGY_TYPE`] (the `type` meaning), or `None` if absent.
    ///
    /// A distinguished node directly under the [`ontology_root`](Self::ontology_root):
    /// the broadest classifications descend from it, so a reasoner can ask "what
    /// kind of thing is this?" by walking up to the type sub-root.
    #[must_use]
    pub fn ontology_type_root(&self) -> Option<&Meaning> {
        self.meanings
            .iter()
            .find(|m| m.has_role(ROLE_ONTOLOGY_TYPE))
    }

    /// The top-level ontological categories — every meaning carrying
    /// [`ROLE_ONTOLOGY_CATEGORY`] (entity, concept, relation, action, property).
    ///
    /// These are the genera each domain cluster roots in, so generic reasoning
    /// can classify any meaning into a small, fixed set of categories rather
    /// than special-casing each domain.
    pub fn ontology_categories(&self) -> impl Iterator<Item = &Meaning> {
        self.meanings_with_role(ROLE_ONTOLOGY_CATEGORY)
    }
}

/// Does the surface word/phrase `expected` appear in `normalized`?
///
/// CJK surfaces have no inter-word spaces, so they match as substrings.
/// Space-delimited scripts match on whole-token boundaries — equal to the
/// whole string, or bounded by spaces — so a multi-word phrase ("each step")
/// matches as a unit and a short word ("api") never matches inside a longer
/// one ("напиши"). An empty surface never matches.
fn surface_present(normalized: &str, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized == expected
        || normalized.starts_with(&format!("{expected} "))
        || normalized.ends_with(&format!(" {expected}"))
        || normalized.contains(&format!(" {expected} "))
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
                    .map(|w| WordForm {
                        text: w.id.clone(),
                        description: w.find_child_value("description").to_string(),
                        action: w.find_child_value("action").to_string(),
                    })
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
    use super::super::roles::{
        ROLE_MECHANISM_INQUIRY, ROLE_PROCEDURAL_REQUEST, ROLE_PROGRAM_ARTIFACT,
        ROLE_PROGRAM_MODIFICATION, ROLE_TRANSLATION_ACTION,
    };
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
    fn word_forms_round_trip_and_describe_word_resolves() {
        // The surface text iterated by `words()` and the text carried by each
        // `WordForm` must be the same set, and `describe_word` must resolve a
        // recorded form (case-insensitively) while rejecting an unknown one.
        let lex = lexicon();
        let meaning = &lex.meanings[0];
        let from_words: Vec<&str> = meaning.words().collect();
        let from_forms: Vec<&str> = meaning.word_forms().map(|f| f.text.as_str()).collect();
        assert_eq!(
            from_words, from_forms,
            "words() and word_forms() must enumerate the same surfaces in order"
        );
        let first = from_words[0];
        assert!(
            meaning.describe_word(first).is_some(),
            "describe_word must resolve a recorded surface form"
        );
        assert!(
            meaning
                .describe_word("\u{0}-definitely-not-a-recorded-surface")
                .is_none(),
            "describe_word must return None for an unknown surface"
        );
    }

    #[test]
    fn descriptions_are_parsed_from_the_seed() {
        // Proves the `description` child is actually read off the wire:
        // at least one surface form must carry a non-empty self-describing note.
        // (The whole-lexicon enforcement invariant lives in
        // `every_word_form_is_described`.)
        let lex = lexicon();
        let described = lex
            .meanings
            .iter()
            .flat_map(Meaning::word_forms)
            .filter(|f| !f.description.trim().is_empty())
            .count();
        assert!(
            described > 0,
            "the parser must read `description` children off the seed"
        );
    }

    #[test]
    fn every_word_form_is_described() {
        // The self-describing-data contract (issue #386): a meaning may not just
        // *list* a surface form, it must *describe* it. Every `word` in every
        // lexeme of every meaning carries a non-empty `description`, so the seed
        // can be consumed for any purpose — not just the parsing order the code
        // happens to use today. This is the permanent ratchet behind the
        // per-language backfill; a new word with no description fails the build.
        let lex = lexicon();
        let mut missing: Vec<String> = Vec::new();
        for meaning in &lex.meanings {
            for lexeme in &meaning.lexemes {
                for form in &lexeme.words {
                    if form.description.trim().is_empty() {
                        missing.push(format!(
                            "{} / {} / {}",
                            meaning.slug, lexeme.language, form.text
                        ));
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "{} word form(s) lack a description, e.g. {}",
            missing.len(),
            missing
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        );
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
    fn word_form_slot_is_derived_from_the_ellipsis_marker() {
        // The slot classification is purely a function of where the `…` (U+2026)
        // marker sits in the surface text (issue #386): no marker is Bare, a
        // trailing marker is Prefix, a leading marker is Suffix, and a middle
        // marker is Circumfix. before_slot/after_slot expose the literals around
        // the slot so a handler can match them without naming the surface.
        let form = |text: &str| WordForm {
            text: text.to_string(),
            description: String::new(),
            action: String::new(),
        };

        let bare = form("how it works");
        assert_eq!(bare.slot(), Slot::Bare);
        assert_eq!(bare.before_slot(), "how it works");
        assert_eq!(bare.after_slot(), "");

        let prefix = form("how does …");
        assert_eq!(prefix.slot(), Slot::Prefix);
        assert_eq!(prefix.before_slot(), "how does ");
        assert_eq!(prefix.after_slot(), "");

        let suffix = form("… как работает");
        assert_eq!(suffix.slot(), Slot::Suffix);
        assert_eq!(suffix.before_slot(), "");
        assert_eq!(suffix.after_slot(), " как работает");

        let circumfix = form("how … works");
        assert_eq!(circumfix.slot(), Slot::Circumfix);
        assert_eq!(circumfix.before_slot(), "how ");
        assert_eq!(circumfix.after_slot(), " works");
    }

    #[test]
    fn how_cluster_roles_populate_and_classify() {
        let lex = lexicon();

        // mechanism_inquiry surfaces span all four slot kinds; representative
        // surfaces must land in the expected bucket with the expected literals.
        let mech = lex.role_word_forms(ROLE_MECHANISM_INQUIRY);
        assert!(
            !mech.is_empty(),
            "mechanism_inquiry must contribute surface forms"
        );
        assert!(mech.iter().any(|f| f.slot() == Slot::Bare));
        assert!(mech.iter().any(|f| f.slot() == Slot::Prefix));
        assert!(mech.iter().any(|f| f.slot() == Slot::Suffix));
        assert!(mech.iter().any(|f| f.slot() == Slot::Circumfix));
        assert!(
            mech.iter()
                .any(|f| f.slot() == Slot::Bare && f.text == "how it works"),
            "the bare English how-it-works phrase must be present"
        );
        assert!(
            mech.iter()
                .any(|f| f.slot() == Slot::Prefix && f.before_slot() == "how does "),
            "the `how does …` prefix surface must be present"
        );
        assert!(
            mech.iter().any(|f| f.slot() == Slot::Circumfix
                && f.before_slot() == "how "
                && f.after_slot() == " works"),
            "the `how … works` circumfix surface must be present"
        );
        assert!(
            mech.iter()
                .any(|f| f.slot() == Slot::Suffix && f.after_slot() == " как работает"),
            "the `… как работает` suffix surface must be present"
        );

        // procedural_request surfaces are all prefixes; some name a canonical
        // action, others leave the operation to the matched task's first word.
        let proc = lex.role_word_forms(ROLE_PROCEDURAL_REQUEST);
        assert!(
            !proc.is_empty(),
            "procedural_request must contribute surface forms"
        );
        assert!(
            proc.iter().all(|f| f.slot() == Slot::Prefix),
            "every procedural surface positions the task after the slot"
        );
        assert!(
            proc.iter()
                .any(|f| f.before_slot() == "how to do " && f.action == "do"),
            "`how to do …` must name the do action explicitly"
        );
        assert!(
            proc.iter()
                .any(|f| f.before_slot() == "how to " && f.action.is_empty()),
            "`how to …` must leave the action to the task"
        );
        assert!(
            proc.iter()
                .any(|f| f.before_slot() == "如何做 " && f.action == "do"),
            "the Chinese `如何做 …` surface must carry its trailing space and do action"
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

    #[test]
    fn mentions_role_raw_matches_inflected_stems() {
        // The raw sibling is the looser match a legacy stem recogniser needs: the
        // bare imperative `отмени` is a substring of the longer word
        // `отменительный`, so the raw query fires where the token-bounded one
        // deliberately does not. This is the byte-faithful behaviour the old
        // `normalized.contains("…")` disjunctions relied on.
        let lex = lexicon();
        assert!(lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
        assert!(!lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
        // A whole-token surface still matches under the raw query.
        assert!(lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "отмени сортировку"));
        // A prompt with no modification word matches under neither query.
        assert!(!lex.mentions_role_raw(ROLE_PROGRAM_MODIFICATION, "привет мир"));
    }

    #[test]
    fn words_for_role_partition_by_language() {
        // The translation-action stems split by linguistic typology: the
        // clause-initial English/Russian command verbs versus the head-final
        // Hindi/Chinese ones. Each partition draws only its own languages' words.
        let lex = lexicon();
        let head_initial = lex.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["en", "ru"]);
        assert!(head_initial.iter().any(|w| w == "translate"));
        assert!(head_initial.iter().any(|w| w == "переведи"));
        assert!(head_initial.iter().any(|w| w == "опиши"));
        assert!(!head_initial.iter().any(|w| w == "翻译"));
        let head_final = lex.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["hi", "zh"]);
        assert!(head_final.iter().any(|w| w == "翻译"));
        assert!(head_final.iter().any(|w| w == "अनुवाद"));
        assert!(!head_final.iter().any(|w| w == "translate"));
    }

    #[test]
    fn first_role_language_reads_the_command_language() {
        // The source-inferencer asks which language's translation verb a prompt
        // carries; that language is the language the user wrote the command in.
        let lex = lexicon();
        let priority = ["ru", "hi", "zh"];
        assert_eq!(
            lex.first_role_language(ROLE_TRANSLATION_ACTION, "переведи apple", &priority),
            Some("ru")
        );
        assert_eq!(
            lex.first_role_language(ROLE_TRANSLATION_ACTION, "apple का अनुवाद करो", &priority),
            Some("hi")
        );
        assert_eq!(
            lex.first_role_language(ROLE_TRANSLATION_ACTION, "把 apple 翻译成中文", &priority),
            Some("zh")
        );
        // No command verb present → no language inferred (caller defaults to en).
        assert_eq!(
            lex.first_role_language(ROLE_TRANSLATION_ACTION, "what is apple", &priority),
            None
        );
    }

    #[test]
    fn the_ontology_has_a_single_link_root() {
        // The merged ontology has exactly one root — the `link` meaning, which
        // is defined_by itself. A type-system sub-root (`type`) sits under it,
        // realising "Link should be the root of ontology, we can also have Type
        // link, for type system ontology" (issue #386).
        let lex = lexicon();
        let roots: Vec<&Meaning> = lex.meanings_with_role(ROLE_ONTOLOGY_ROOT).collect();
        assert_eq!(
            roots.len(),
            1,
            "the merged ontology must have exactly one root, found {}",
            roots.len()
        );
        let root = roots[0];
        assert_eq!(root.slug, "link", "the ontology root must be `link`");
        assert!(
            root.defined_by.iter().any(|t| t == "link"),
            "the root `link` must be defined by itself (self-rooted)"
        );
        let type_root = lex
            .ontology_type_root()
            .expect("a type-system sub-root (role ontology_type) must exist");
        assert!(
            lex.reaches_root(&type_root.slug),
            "the type sub-root must reduce to the link root"
        );
        // The bridge categories (entity, concept, relation, action, property)
        // each sit under the root too, so every domain genus has a category to
        // root in.
        let categories: Vec<&Meaning> = lex.ontology_categories().collect();
        assert!(
            categories.len() >= 2,
            "the ontology must define top-level categories under the root, found {}",
            categories.len()
        );
        for category in categories {
            assert!(
                lex.reaches_root(&category.slug),
                "ontology category {} must reduce to the link root",
                category.slug
            );
        }
    }

    #[test]
    fn every_meaning_reaches_the_link_root() {
        // The whole lexicon is one connected ontology: following `defined_by`
        // from any meaning eventually arrives at the single `link` root. No
        // meaning is an island of vocabulary disconnected from the root concept.
        let lex = lexicon();
        assert!(lex.ontology_root().is_some(), "an ontology root must exist");
        for meaning in &lex.meanings {
            assert!(
                lex.reaches_root(&meaning.slug),
                "{} does not reach the `link` ontology root via defined_by",
                meaning.slug
            );
        }
    }
}
