//! The shared text formalizer (R1188-U18, U19 and U21).
//!
//! Any text becomes formal statements whose terms are meanings of the seed
//! lexicon, and statements become text again (the rendering half is
//! [`super::statement_rendering`]). Page formalization
//! ([`super::page`]), round-trip translation
//! ([`crate::translation::round_trip`]) and dependency summarization
//! ([`crate::summarization::dependency`]) all read text through this one
//! module. It mirrors `js/agentic/crate/text_formalization.mjs`, the
//! JavaScript root.
//!
//! A text is cut into sentences and clauses at the seeded punctuation
//! ([`super::segment`]). Each clause is read word by word: a word or phrase
//! the lexicon lists in the text's language becomes that meaning (an inflected
//! word is folded to its dictionary form by the seeded endings first); a
//! numeral becomes a number; a capitalized word the lexicon does not know
//! becomes a name, except the word that opens a sentence unless the text
//! writes it as a name elsewhere; function words and negation cues carry no
//! term; anything else is an unknown term, kept with its surface (its identity
//! is the word without its seeded ending). A clause is one statement whose
//! subject is its first term, and a clause that opens with an anaphor or a
//! clause continuation keeps the subject of the statement before it. The words
//! of those kinds live in `data/seed/text-formalization.lino`.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use unicode_general_category::{GeneralCategory, get_general_category};

use super::segment::{Script, Segment, clauses, script_of, sentences};
use crate::seed::parser::LinoNode;
use crate::seed::{TEXT_FORMALIZATION_LINO, lexicon, parse_lino};

/// The lexicon roles whose words carry no term.
pub const FUNCTION_WORD_ROLES: [&str; 2] = ["translation_stop_word", "statement_function_word"];

/// The lexicon role whose words deny a statement.
pub const NEGATION_ROLE: &str = "statement_negation_cue";

/// The most words one lexicon surface may span.
pub const LONGEST_PHRASE: usize = 4;

/// The most characters one Han lexicon surface may span.
pub const LONGEST_HAN_WORD: usize = 8;

/// An inflected word keeps at least this many characters.
pub const SHORTEST_STEM: usize = 3;

const APOSTROPHES: [char; 2] = ['\'', '\u{2019}'];
const NUMBER_SEPARATORS: [char; 2] = ['.', ','];
const PLACEHOLDERS: [char; 6] = ['{', '}', '<', '>', '\u{2026}', '|'];

/// What one word of a clause is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    /// A meaning of the seed lexicon.
    Meaning,
    /// A capitalized word the lexicon does not know.
    Name,
    /// A numeral.
    Number,
    /// A word no meaning was found for.
    Unknown,
    /// A function word, which carries no term.
    Function,
    /// A negation cue, which denies the statement.
    Negation,
    /// A pronoun pointing back at the subject named before.
    Anaphor,
    /// A conjunction or relative pronoun continuing the clause before.
    Continuation,
}

impl WordKind {
    /// Whether words of this kind become terms of a statement.
    #[must_use]
    pub const fn is_term(self) -> bool {
        matches!(
            self,
            Self::Meaning | Self::Name | Self::Number | Self::Unknown
        )
    }

    /// Whether terms of this kind name something formal.
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Meaning | Self::Name | Self::Number)
    }
}

/// Whether a statement is asserted or denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// No negation cue.
    Asserted,
    /// A negation cue denies the statement.
    Denied,
}

impl Polarity {
    /// The label the identity of a statement carries.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Denied => "denied",
        }
    }
}

/// One word of a clause, or one term of a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// The text as written.
    pub surface: String,
    /// What the word is.
    pub kind: WordKind,
    /// The meaning slug, or `name:`, `number:` or `unknown:` and the word.
    pub id: String,
}

/// One clause read as a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    /// The clause text.
    pub text: String,
    /// The language it was read in.
    pub language: String,
    /// Asserted or denied.
    pub polarity: Polarity,
    /// The first term, or the subject carried over from the clause before.
    pub subject: Option<Word>,
    /// The other terms, in order.
    pub terms: Vec<Word>,
    /// The surfaces no meaning was found for.
    pub unknown: Vec<String>,
}

/// One sentence with the statements of its clauses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The sentence text.
    pub text: String,
    /// Its statements, in order.
    pub statements: Vec<Statement>,
}

/// A run of word characters, Han apart from any other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The text as written.
    pub surface: String,
    /// Whether the run is Han.
    pub han: bool,
}

/// A letter, mark or number of any script: what a word is made of.
#[must_use]
pub fn is_word_character(character: char) -> bool {
    matches!(
        get_general_category(character),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
    )
}

fn is_numeral_character(character: char) -> bool {
    matches!(
        get_general_category(character),
        GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
    )
}

fn is_han(character: char) -> bool {
    script_of(character) == Script::Han
}

/// Whether `surface` is a number: numerals, with `.` or `,` between numerals.
fn is_number(surface: &str) -> bool {
    surface
        .split(NUMBER_SEPARATORS)
        .all(|part| !part.is_empty() && part.chars().all(is_numeral_character))
}

fn seed_record() -> Option<&'static LinoNode> {
    static ROOT: OnceLock<LinoNode> = OnceLock::new();
    let root = ROOT.get_or_init(|| parse_lino(TEXT_FORMALIZATION_LINO));
    if root.name == "text-formalization" {
        return Some(root);
    }
    root.children
        .iter()
        .find(|node| node.name == "text-formalization")
}

/// The words of one kind the seed lists for `language`, lowercased.
#[must_use]
pub fn seed_words(language: &str, kind: &str) -> Vec<String> {
    seed_record()
        .and_then(|record| {
            record
                .children
                .iter()
                .find(|node| node.name == "language" && node.id == language)
        })
        .map(|record| {
            record
                .children
                .iter()
                .filter(|node| node.name == kind)
                .map(|node| node.id.trim_matches('"').to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

/// The seeded inflection endings of `language`, longest first.
#[must_use]
pub fn inflection_endings(language: &str) -> Vec<String> {
    let mut endings = seed_words(language, "inflection-ending");
    endings.sort_by_key(|ending| core::cmp::Reverse(ending.chars().count()));
    endings
}

/// The lowercased words of `role`'s meanings in `language`, each once, in
/// lexicon order.
#[must_use]
pub fn role_words_in(role: &str, language: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for meaning in lexicon().meanings_with_role(role) {
        for lexeme in meaning
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language)
        {
            for word in &lexeme.words {
                let lower = word.text.to_lowercase();
                if !out.contains(&lower) {
                    out.push(lower);
                }
            }
        }
    }
    out
}

/// Every function word of `language`.
#[must_use]
pub fn function_words(language: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for role in FUNCTION_WORD_ROLES {
        for word in role_words_in(role, language) {
            if !out.contains(&word) {
                out.push(word);
            }
        }
    }
    out
}

/// The maximal runs of word characters in `text`.
///
/// A Han run stands apart from any other run, an apostrophe between two word
/// characters stays inside its word, and a decimal or group separator between
/// two digits stays inside its number (`1.2`, `378,000`).
#[must_use]
pub fn tokens(text: &str) -> Vec<Token> {
    let list: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut buffer_han = false;
    for (index, character) in list.iter().copied().enumerate() {
        let next = list.get(index + 1).copied();
        let previous = index
            .checked_sub(1)
            .and_then(|before| list.get(before))
            .copied();
        let joins_word = APOSTROPHES.contains(&character)
            && !buffer.is_empty()
            && !buffer_han
            && next.is_some_and(|after| is_word_character(after) && !is_han(after));
        let joins_number = NUMBER_SEPARATORS.contains(&character)
            && !buffer.is_empty()
            && previous.is_some_and(is_numeral_character)
            && next.is_some_and(is_numeral_character);
        if joins_word || joins_number {
            buffer.push(character);
            continue;
        }
        if !is_word_character(character) {
            flush_token(&mut buffer, buffer_han, &mut out);
            continue;
        }
        let han = is_han(character);
        if !buffer.is_empty() && han != buffer_han {
            flush_token(&mut buffer, buffer_han, &mut out);
        }
        buffer_han = han;
        buffer.push(character);
    }
    flush_token(&mut buffer, buffer_han, &mut out);
    out
}

fn flush_token(buffer: &mut String, han: bool, out: &mut Vec<Token>) {
    if !buffer.is_empty() {
        out.push(Token {
            surface: core::mem::take(buffer),
            han,
        });
    }
}

/// A surface's lowercased words joined by one space, or `None` when it is not
/// plain words (a template, a Han phrase of several runs, too long).
#[must_use]
pub fn phrase_key(surface: &str) -> Option<String> {
    if surface
        .chars()
        .any(|character| PLACEHOLDERS.contains(&character))
    {
        return None;
    }
    let words = tokens(surface);
    if words.is_empty() || words.len() > LONGEST_PHRASE {
        return None;
    }
    if words.len() > 1 && words.iter().any(|word| word.han) {
        return None;
    }
    let key = words
        .iter()
        .map(|word| word.surface.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let plain = surface
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (key == plain).then_some(key)
}

/// Why one meaning holds a surface over another.
struct Rank {
    slug: String,
    languages: usize,
    size: usize,
    order: usize,
}

impl Rank {
    /// The meaning written in more languages wins, then the one with fewer
    /// surfaces across all its languages, then the one declared first.
    const fn outranks(&self, held: &Self) -> bool {
        if self.languages != held.languages {
            return self.languages > held.languages;
        }
        if self.size != held.size {
            return self.size < held.size;
        }
        self.order < held.order
    }
}

fn meaning_indexes() -> &'static HashMap<String, HashMap<String, String>> {
    static INDEXES: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
    INDEXES.get_or_init(build_meaning_indexes)
}

fn build_meaning_indexes() -> HashMap<String, HashMap<String, String>> {
    let mut best: HashMap<String, HashMap<String, Rank>> = HashMap::new();
    for (order, meaning) in lexicon().meanings.iter().enumerate() {
        let excluded = FUNCTION_WORD_ROLES
            .iter()
            .any(|role| meaning.has_role(role))
            || meaning.has_role(NEGATION_ROLE);
        if excluded {
            continue;
        }
        let languages: BTreeSet<&str> = meaning
            .lexemes
            .iter()
            .map(|lexeme| lexeme.language.as_str())
            .collect();
        // Counted across every language, so one meaning wins a shared word in
        // each language, and the rank still separates meanings at full parity.
        let size: usize = meaning
            .lexemes
            .iter()
            .map(|lexeme| lexeme.words.len())
            .sum();
        for language in &languages {
            let Some(lexeme) = meaning
                .lexemes
                .iter()
                .find(|lexeme| lexeme.language == *language)
            else {
                continue;
            };
            let held = best.entry((*language).to_owned()).or_default();
            for word in &lexeme.words {
                let Some(key) = phrase_key(&word.text) else {
                    continue;
                };
                let rank = Rank {
                    slug: meaning.slug.clone(),
                    languages: languages.len(),
                    size,
                    order,
                };
                let replace = held.get(&key).is_none_or(|current| rank.outranks(current));
                if replace {
                    held.insert(key, rank);
                }
            }
        }
    }
    best.into_iter()
        .map(|(language, ranks)| {
            let slugs: HashMap<String, String> = ranks
                .into_iter()
                .map(|(key, rank)| (key, rank.slug))
                .collect();
            (language, slugs)
        })
        .collect()
}

/// The meaning a lexicon surface (already a phrase key) names in `language`.
#[must_use]
pub fn indexed_meaning(key: &str, language: &str) -> Option<String> {
    meaning_indexes()
        .get(language)
        .and_then(|index| index.get(key))
        .cloned()
}

/// The meaning an inflected word names, reached by removing one seeded ending
/// and trying the bare stem or the stem with another ending.
#[must_use]
pub fn fold_inflection(lower: &str, language: &str) -> Option<String> {
    let endings = inflection_endings(language);
    let length = lower.chars().count();
    for ending in &endings {
        if !lower.ends_with(ending.as_str())
            || length.saturating_sub(ending.chars().count()) < SHORTEST_STEM
        {
            continue;
        }
        let stem = &lower[..lower.len() - ending.len()];
        if let Some(slug) = indexed_meaning(stem, language) {
            return Some(slug);
        }
        for replacement in &endings {
            if replacement == ending {
                continue;
            }
            if let Some(slug) = indexed_meaning(&format!("{stem}{replacement}"), language) {
                return Some(slug);
            }
        }
    }
    None
}

/// An unknown word without its longest seeded ending.
///
/// The stem keeps [`SHORTEST_STEM`] characters, so `orbits` and `orbited` are
/// one unknown term.
#[must_use]
pub fn unknown_stem(lower: &str, language: &str) -> String {
    let length = lower.chars().count();
    for ending in inflection_endings(language) {
        if lower.ends_with(ending.as_str())
            && length.saturating_sub(ending.chars().count()) >= SHORTEST_STEM
        {
            return lower[..lower.len() - ending.len()].to_owned();
        }
    }
    lower.to_owned()
}

/// The kind a listed lowercased surface carries in `language`: an anaphor,
/// a clause continuation, a negation cue or a function word.
#[must_use]
pub fn kind_of_listed(lower: &str, language: &str) -> Option<WordKind> {
    let listed = |words: Vec<String>| words.iter().any(|word| word == lower);
    if listed(seed_words(language, "anaphor")) {
        return Some(WordKind::Anaphor);
    }
    if listed(seed_words(language, "clause-continuation")) {
        return Some(WordKind::Continuation);
    }
    if listed(role_words_in(NEGATION_ROLE, language)) {
        return Some(WordKind::Negation);
    }
    if listed(function_words(language)) {
        return Some(WordKind::Function);
    }
    None
}

const fn word(surface: String, kind: WordKind, id: String) -> Word {
    Word { surface, kind, id }
}

/// The words of a run of non-Han tokens; `opens_sentence` when its first
/// token is the first word of a sentence.
fn read_plain_run(run: &[Token], language: &str, opens_sentence: bool) -> Vec<Word> {
    let mut out = Vec::new();
    let mut position = 0;
    while position < run.len() {
        if let Some((length, phrase)) = longest_phrase(run, position, language) {
            out.push(phrase);
            position += length;
            continue;
        }
        out.push(read_single(
            &run[position].surface,
            language,
            opens_sentence && position == 0,
        ));
        position += 1;
    }
    join_names(out)
}

/// A lexicon phrase of two or more tokens starting at `position`.
fn longest_phrase(run: &[Token], position: usize, language: &str) -> Option<(usize, Word)> {
    let longest = LONGEST_PHRASE.min(run.len() - position);
    for length in (2..=longest).rev() {
        let parts: Vec<&str> = run[position..position + length]
            .iter()
            .map(|token| token.surface.as_str())
            .collect();
        let key = parts
            .iter()
            .map(|part| part.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(slug) = indexed_meaning(&key, language) {
            return Some((length, word(parts.join(" "), WordKind::Meaning, slug)));
        }
    }
    None
}

/// One non-Han token as a word; `initial` when it opens a sentence.
fn read_single(surface: &str, language: &str, initial: bool) -> Word {
    let lower = surface.to_lowercase();
    if let Some(kind) = kind_of_listed(&lower, language) {
        return word(surface.to_owned(), kind, lower);
    }
    if let Some(slug) = indexed_meaning(&lower, language) {
        return word(surface.to_owned(), WordKind::Meaning, slug);
    }
    if is_number(surface) {
        return word(
            surface.to_owned(),
            WordKind::Number,
            format!("number:{surface}"),
        );
    }
    if let Some(slug) = fold_inflection(&lower, language) {
        return word(surface.to_owned(), WordKind::Meaning, slug);
    }
    if !initial && surface.chars().next().is_some_and(char::is_uppercase) {
        return word(surface.to_owned(), WordKind::Name, format!("name:{lower}"));
    }
    let stem = unknown_stem(&lower, language);
    word(
        surface.to_owned(),
        WordKind::Unknown,
        format!("unknown:{stem}"),
    )
}

/// Consecutive names become one name.
fn join_names(words: Vec<Word>) -> Vec<Word> {
    let mut out: Vec<Word> = Vec::new();
    for next in words {
        if let Some(last) = out.last_mut()
            && last.kind == WordKind::Name
            && next.kind == WordKind::Name
        {
            let surface = format!("{} {}", last.surface, next.surface);
            last.id = format!("name:{}", surface.to_lowercase());
            last.surface = surface;
            continue;
        }
        out.push(next);
    }
    out
}

/// A Han run cut by longest match against the lexicon and the listed words;
/// characters no entry covers gather into one unknown word.
fn read_han_run(surface: &str, language: &str) -> Vec<Word> {
    let list: Vec<char> = surface.chars().collect();
    let mut out = Vec::new();
    let mut unknown = String::new();
    let mut position = 0;
    while position < list.len() {
        let Some((length, found)) = longest_han_word(&list, position, language) else {
            unknown.push(list[position]);
            position += 1;
            continue;
        };
        flush_unknown(&mut unknown, &mut out);
        out.push(found);
        position += length;
    }
    flush_unknown(&mut unknown, &mut out);
    out
}

fn flush_unknown(unknown: &mut String, out: &mut Vec<Word>) {
    if unknown.is_empty() {
        return;
    }
    let surface = core::mem::take(unknown);
    let id = format!("unknown:{surface}");
    out.push(word(surface, WordKind::Unknown, id));
}

fn longest_han_word(list: &[char], position: usize, language: &str) -> Option<(usize, Word)> {
    let longest = LONGEST_HAN_WORD.min(list.len() - position);
    for length in (1..=longest).rev() {
        let surface: String = list[position..position + length].iter().collect();
        if let Some(kind) = kind_of_listed(&surface, language) {
            return Some((length, word(surface.clone(), kind, surface)));
        }
        if let Some(slug) = indexed_meaning(&surface, language) {
            return Some((length, word(surface, WordKind::Meaning, slug)));
        }
    }
    None
}

/// The words of one clause in `language`; `opens_sentence` when the clause
/// is the first of its sentence.
#[must_use]
pub fn read_words(text: &str, language: &str, opens_sentence: bool) -> Vec<Word> {
    let mut out = Vec::new();
    let mut run: Vec<Token> = Vec::new();
    for token in tokens(text) {
        if !token.han {
            run.push(token);
            continue;
        }
        flush_run(&mut run, language, opens_sentence, &mut out);
        out.extend(read_han_run(&token.surface, language));
    }
    flush_run(&mut run, language, opens_sentence, &mut out);
    out
}

fn flush_run(run: &mut Vec<Token>, language: &str, opens_sentence: bool, out: &mut Vec<Word>) {
    if run.is_empty() {
        return;
    }
    let opens = opens_sentence && out.is_empty();
    out.extend(read_plain_run(run, language, opens));
    run.clear();
}

/// One clause as a statement.
///
/// A clause whose first word that is not a function word is an anaphor or a
/// continuation keeps `previous_subject`.
#[must_use]
pub fn clause_statement(
    text: &str,
    language: &str,
    previous_subject: Option<&Word>,
    opens_sentence: bool,
) -> Statement {
    let words = read_words(text, language, opens_sentence);
    let inherits = words
        .iter()
        .find(|word| word.kind != WordKind::Function)
        .is_some_and(|opener| matches!(opener.kind, WordKind::Anaphor | WordKind::Continuation));
    let content: Vec<Word> = words
        .iter()
        .filter(|word| word.kind.is_term())
        .cloned()
        .collect();
    let unknown = content
        .iter()
        .filter(|term| term.kind == WordKind::Unknown)
        .map(|term| term.surface.clone())
        .collect();
    let polarity = if words.iter().any(|word| word.kind == WordKind::Negation) {
        Polarity::Denied
    } else {
        Polarity::Asserted
    };
    let (subject, terms) = if inherits {
        (previous_subject.cloned(), content)
    } else {
        let mut rest = content.into_iter();
        let first = rest.next();
        (first, rest.collect())
    };
    Statement {
        text: text.to_owned(),
        language: language.to_owned(),
        polarity,
        subject,
        terms,
        unknown,
    }
}

/// The clauses of a sentence, with a separator that stands between two digits
/// (`378,000`) kept inside its number.
#[must_use]
pub fn statement_clauses(sentence: &Segment) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for clause in clauses(sentence) {
        let splits_number = out.last().is_some_and(|last| {
            last.text.chars().last().is_some_and(is_numeral_character)
                && clause.text.chars().next().is_some_and(is_numeral_character)
        });
        if !splits_number {
            out.push(clause);
            continue;
        }
        if let Some(last) = out.last_mut() {
            let from = last.start - sentence.start;
            let to = clause.end - sentence.start;
            sentence.text[from..to].clone_into(&mut last.text);
            last.end = clause.end;
        }
    }
    out
}

/// Every sentence of `text` with the statements of its clauses.
///
/// A statement's subject carries over to the next clause that points back.
#[must_use]
pub fn formalize_sentences(text: &str, language: &str) -> Vec<Sentence> {
    let mut out = Vec::new();
    let mut previous_subject: Option<Word> = None;
    for sentence in sentences(text) {
        let mut statements = Vec::new();
        for (index, clause) in statement_clauses(&sentence).iter().enumerate() {
            let statement = clause_statement(
                &clause.text,
                language,
                previous_subject.as_ref(),
                index == 0,
            );
            if statement.subject.is_none() && statement.terms.is_empty() {
                continue;
            }
            if let Some(subject) = &statement.subject {
                previous_subject = Some(subject.clone());
            }
            statements.push(statement);
        }
        out.push(Sentence {
            text: sentence.text,
            statements,
        });
    }
    promote_names(out)
}

/// The sentences with every sentence-opening unknown word promoted to the
/// name the same text writes inside a sentence (`Earth is ...` after
/// `... of Earth`).
#[must_use]
pub fn promote_names(formalized: Vec<Sentence>) -> Vec<Sentence> {
    let names: BTreeSet<String> = formalized
        .iter()
        .flat_map(|sentence| sentence.statements.iter())
        .flat_map(statement_terms)
        .filter(|term| term.kind == WordKind::Name)
        .map(|term| term.id.clone())
        .collect();
    let promoted = |surface: &str| {
        surface.chars().next().is_some_and(char::is_uppercase)
            && names.contains(&format!("name:{}", surface.to_lowercase()))
    };
    let promote = |term: Word| {
        if term.kind == WordKind::Unknown && promoted(term.surface.as_str()) {
            Word {
                id: format!("name:{}", term.surface.to_lowercase()),
                kind: WordKind::Name,
                surface: term.surface,
            }
        } else {
            term
        }
    };
    formalized
        .into_iter()
        .map(|sentence| Sentence {
            text: sentence.text,
            statements: sentence
                .statements
                .into_iter()
                .map(|statement| Statement {
                    subject: statement.subject.map(promote),
                    terms: statement.terms.into_iter().map(promote).collect(),
                    unknown: statement
                        .unknown
                        .into_iter()
                        .filter(|surface| !promoted(surface.as_str()))
                        .collect(),
                    ..statement
                })
                .collect(),
        })
        .collect()
}

/// The statements of `text`, in order.
#[must_use]
pub fn formalize_text(text: &str, language: &str) -> Vec<Statement> {
    formalize_sentences(text, language)
        .into_iter()
        .flat_map(|sentence| sentence.statements)
        .collect()
}

/// The subject (when there is one) and the other terms.
#[must_use]
pub fn statement_terms(statement: &Statement) -> Vec<&Word> {
    statement
        .subject
        .iter()
        .chain(statement.terms.iter())
        .collect()
}

/// The distinct ids of every term, sorted.
#[must_use]
pub fn content_ids(statement: &Statement) -> Vec<String> {
    let ids: BTreeSet<String> = statement_terms(statement)
        .into_iter()
        .map(|term| term.id.clone())
        .collect();
    ids.into_iter().collect()
}

/// The distinct ids of the known terms, sorted.
#[must_use]
pub fn known_ids(statement: &Statement) -> Vec<String> {
    let ids: BTreeSet<String> = statement_terms(statement)
        .into_iter()
        .filter(|term| term.kind.is_known())
        .map(|term| term.id.clone())
        .collect();
    ids.into_iter().collect()
}

/// The formal identity of a statement: its polarity, its subject and the set
/// of its other terms.
///
/// Two statements with one identity say the same thing whatever their wording.
#[must_use]
pub fn statement_identity(statement: &Statement) -> String {
    let subject = statement
        .subject
        .as_ref()
        .map_or("", |subject| subject.id.as_str());
    let others: BTreeSet<&str> = statement
        .terms
        .iter()
        .map(|term| term.id.as_str())
        .filter(|id| *id != subject)
        .collect();
    format!(
        "{}|{subject}|{}",
        statement.polarity.slug(),
        others.into_iter().collect::<Vec<_>>().join(" ")
    )
}

/// Whether a statement has at least two terms, all of them known.
#[must_use]
pub fn has_no_unknown(statement: &Statement) -> bool {
    let terms = statement_terms(statement);
    terms.len() >= 2 && terms.iter().all(|term| term.kind.is_known())
}
