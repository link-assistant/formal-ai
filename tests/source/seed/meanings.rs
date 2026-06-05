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
    /// The Wikidata entity (`Q…`) or property (`P…`) id this meaning is rooted
    /// in, when it corresponds to one. Empty for meanings that have no Wikidata
    /// anchor. Lets the formalizer resolve language-independent ids from the
    /// seed instead of hardcoded tables (issue #386).
    pub wikidata: String,
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

    /// Does this meaning lexicalise `surface` as a whole surface form in
    /// `language` (exact, case-sensitive match)? The compositional translator
    /// resolves a normalized source word to the concept that lists it through
    /// this, so the per-word table stays in the data (issue #386).
    fn lexeme_lists(&self, language: &str, surface: &str) -> bool {
        self.lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language)
            .flat_map(|lexeme| lexeme.words.iter())
            .any(|word| word.text == surface)
    }

    /// Like [`Meaning::lexeme_lists`] but the matched form must also carry
    /// `action` — the per-form grammatical tag (e.g. a genitive inflection). Lets
    /// the compositional translator pick a single inflected form out of a meaning
    /// without naming it in code (issue #386).
    fn lexeme_lists_action(&self, language: &str, surface: &str, action: &str) -> bool {
        self.lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language)
            .flat_map(|lexeme| lexeme.words.iter())
            .any(|word| word.text == surface && word.action == action)
    }
}

/// A spelled-surface → value-surface rewrite table: each entry maps a spelled
/// surface (a word or, for [`WordValueTable`] phrases, a multi-word string) to
/// the value surface of its meaning — the numeral or operator symbol carrying no
/// alphabetic character. Both halves of the arithmetic normalization mapping
/// returned by [`Lexicon::arithmetic_normalization_tables`] share this shape.
pub type WordValueTable = Vec<(String, String)>;

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

    /// The meaning rooted in the Wikidata entity or property `id` (e.g. `Q89`,
    /// `P31`), if the seed carries one. Lets the formalizer resolve a
    /// language-independent id back to its canonical label and surfaces without
    /// a hardcoded table (issue #386).
    #[must_use]
    pub fn meaning_by_wikidata(&self, id: &str) -> Option<&Meaning> {
        self.meanings.iter().find(|m| m.wikidata == id)
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

    /// Translate `surface` from `source` to `target` through the meaning carrying
    /// `role` that lexicalises it.
    ///
    /// Finds the first meaning (declaration order) carrying `role` whose `source`
    /// lexeme lists `surface`, then returns its first `target`-language form. The
    /// compositional ru→en fallback resolves a lemma or fixed phrase to English
    /// through this — naming the semantic role and the language codes, never the
    /// surface words, which live in `data/seed/meanings-translation.lino` (#386).
    #[must_use]
    pub fn role_surface_translation<'a>(
        &'a self,
        role: &str,
        source: &str,
        target: &str,
        surface: &str,
    ) -> Option<&'a str> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .find(|meaning| meaning.lexeme_lists(source, surface))
            .and_then(|meaning| meaning.word_in(target))
    }

    /// Does any meaning carrying `role` lexicalise `surface` in `language`?
    ///
    /// Lets the compositional translator test a structural property of a source
    /// word — e.g. whether it is a genitive-governing head — by role rather than
    /// by naming the word in code (issue #386).
    #[must_use]
    pub fn role_lists_surface(&self, role: &str, language: &str, surface: &str) -> bool {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .any(|meaning| meaning.lexeme_lists(language, surface))
    }

    /// Like [`Lexicon::role_surface_translation`] but the `source` form must also
    /// carry `action`.
    ///
    /// The per-form grammatical tag selects one inflected surface out of a
    /// meaning's lexeme, so the compositional translator resolves a
    /// genitive-inflected complement to its English lemma while leaving the single
    /// tagged form in the data (issue #386).
    #[must_use]
    pub fn role_action_surface_translation<'a>(
        &'a self,
        role: &str,
        action: &str,
        source: &str,
        target: &str,
        surface: &str,
    ) -> Option<&'a str> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .find(|meaning| meaning.lexeme_lists_action(source, surface, action))
            .and_then(|meaning| meaning.word_in(target))
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

    /// Like [`mentions_role`](Self::mentions_role) but ignores a meaning's
    /// script-independent *value surfaces* — word forms that carry no alphabetic
    /// character, such as the operator symbol "+" or the numeral "10".
    ///
    /// Those forms exist so the arithmetic normalizer can read a meaning's
    /// machine value (see
    /// [`arithmetic_normalization_tables`](Self::arithmetic_normalization_tables));
    /// they are not spelled words. Operator-*word* detection must therefore skip
    /// them so a bare "+" is recognised as an operator *symbol* by the symbol
    /// scan, not double-counted as a spelled word operator. This mirrors the
    /// pure-numeral skip already applied to spelled-number detection.
    #[must_use]
    pub fn mentions_role_spelled(&self, role: &str, normalized: &str) -> bool {
        self.meanings_with_role(role).any(|meaning| {
            meaning
                .words()
                .filter(|word| word.chars().any(char::is_alphabetic))
                .any(|word| surface_present(normalized, word))
        })
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

    /// Build the word→value tables the arithmetic evaluator uses to rewrite a
    /// spelled expression into its symbolic form before tokenizing.
    ///
    /// "two plus three" becomes "2 + 3"; "пять умножить на два" becomes
    /// "5 * 2". Returns `(tokens, phrases)`. Each entry maps a spelled surface to
    /// the *value surface* of its meaning — the word form carrying no alphabetic
    /// character: the numeral "2" for the cardinal two, the symbol "+" for the
    /// addition operator. `tokens` are single words, applied after whitespace
    /// tokenization; `phrases` are multi-word surfaces, applied (and so replaced)
    /// before tokenization and ordered longest first so a phrase is rewritten
    /// before any shorter phrase it contains — "разделить на" before "делить на".
    /// Both lists are sorted deterministically so the generated mirror is stable.
    ///
    /// This is the single source of truth behind the generated `no_std` table in
    /// `src/arithmetic_word_tables.rs`: the evaluator is compiled into the wasm
    /// worker, which cannot reach the seed at runtime, so the table is
    /// materialized at build time by `examples/issue_386_gen_arith_table.rs` and
    /// checked against this builder by the `arithmetic_word_tables_match_seed`
    /// test in `src/calculation.rs`.
    #[must_use]
    pub fn arithmetic_normalization_tables(&self) -> (WordValueTable, WordValueTable) {
        let is_value_surface = |word: &str| !word.chars().any(char::is_alphabetic);
        let mut tokens: WordValueTable = Vec::new();
        let mut phrases: WordValueTable = Vec::new();
        for role in [
            super::roles::ROLE_CARDINAL_NUMBER_WORD,
            super::roles::ROLE_ARITHMETIC_OPERATOR_WORD,
        ] {
            for meaning in self.meanings_with_role(role) {
                // The value surface is the unique word form with no alphabetic
                // character: the numeral for a cardinal, the symbol for an
                // operator. Spelled surfaces in every language map onto it.
                let Some(value) = meaning.words().find(|&word| is_value_surface(word)) else {
                    continue;
                };
                for word in meaning.words() {
                    if word == value || is_value_surface(word) {
                        continue;
                    }
                    let entry = (word.to_string(), value.to_string());
                    if word.chars().any(char::is_whitespace) {
                        phrases.push(entry);
                    } else {
                        tokens.push(entry);
                    }
                }
            }
        }
        tokens.sort();
        tokens.dedup();
        phrases.sort_by(|a, b| {
            b.0.chars()
                .count()
                .cmp(&a.0.chars().count())
                .then_with(|| a.0.cmp(&b.0))
        });
        phrases.dedup();
        (tokens, phrases)
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
        wikidata: node.find_child_value("wikidata").to_string(),
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

#[path = "../source_tests/seed/meanings/tests.rs"]
mod tests;
