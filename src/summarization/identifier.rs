//! The identifier rung: one step shorter than a topic label.
//!
//! `Topic` used to be the bottom of the summarization ladder — 1–5 words of
//! prose. Issue #844 extends the ladder downward: an *identifier* is a single
//! token that a compiler, a shell, or a commit log will accept. The rung
//! honours three constraints the prose rungs never had to:
//!
//! 1. **Syntactic validity.** The result must be a legal name in the target
//!    convention, and must not collide with a programming-language keyword
//!    (the seed's [`crate::seed::ROLE_IDENTIFIER_RESERVED_WORD`] list).
//! 2. **A length budget.** [`IdentifierBudget`] caps both the number of words
//!    and the number of characters; words are dropped from the tail before any
//!    character is cut, so the head of the phrase survives.
//! 3. **Naming conventions.** [`NamingConvention`] renders the same word
//!    sequence as `snake_case`, `SCREAMING_SNAKE_CASE`, `camelCase`,
//!    `PascalCase`, or a `Commit subject` line.
//!
//! Rendering is deterministic and non-neural: tokenize, drop function words
//! (seed-driven), cap, render, escape. Non-ASCII words are *kept* rather than
//! transliterated — Rust, Python and JavaScript all accept Unicode identifiers,
//! so `解析器快` is a legal name and inventing a romanization would be a guess.
//! `kebab-case` is deliberately absent: its alphabet has no escape character
//! for a leading digit or a keyword collision, so the rung cannot guarantee a
//! valid result for it.

use super::vocabulary;

/// Default character budget for a code identifier.
pub const DEFAULT_IDENTIFIER_MAX_LENGTH: usize = 40;

/// Default word budget for a code identifier.
pub const DEFAULT_IDENTIFIER_MAX_WORDS: usize = 4;

/// How to render a word sequence as a single name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamingConvention {
    /// `parser_is_fast` — Rust/Python functions and variables.
    #[default]
    SnakeCase,
    /// `PARSER_IS_FAST` — constants.
    ScreamingSnakeCase,
    /// `parserIsFast` — JavaScript/Java members.
    CamelCase,
    /// `ParserIsFast` — types.
    PascalCase,
    /// `Parser is fast` — a commit subject line: prose, capitalized, no
    /// trailing period.
    CommitSubject,
}

impl NamingConvention {
    /// `true` for the conventions that must produce a compiler-legal name.
    /// [`Self::CommitSubject`] is prose and is exempt from keyword escaping.
    #[must_use]
    pub const fn is_code(self) -> bool {
        !matches!(self, Self::CommitSubject)
    }
}

/// Word and character caps for one identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierBudget {
    /// Maximum number of characters in the rendered identifier.
    pub max_length: usize,
    /// Maximum number of words kept from the source phrase.
    pub max_words: usize,
}

impl Default for IdentifierBudget {
    fn default() -> Self {
        Self {
            max_length: DEFAULT_IDENTIFIER_MAX_LENGTH,
            max_words: DEFAULT_IDENTIFIER_MAX_WORDS,
        }
    }
}

impl IdentifierBudget {
    /// Build a budget from explicit caps.
    #[must_use]
    pub const fn new(max_length: usize, max_words: usize) -> Self {
        Self {
            max_length,
            max_words,
        }
    }

    /// The conventional budget for a commit subject: 50 characters, which is
    /// the width every `git log --oneline` renderer assumes, and enough words
    /// to stay readable prose.
    #[must_use]
    pub const fn commit_subject() -> Self {
        Self {
            max_length: 50,
            max_words: 10,
        }
    }

    /// Does `candidate` fit this budget's character cap?
    ///
    /// The word cap is enforced while rendering (words are dropped from the
    /// tail), so it cannot be rechecked from the rendered form — a `snake_case`
    /// name and a commit subject separate words differently.
    #[must_use]
    pub fn admits(&self, candidate: &str) -> bool {
        !candidate.is_empty() && candidate.chars().count() <= self.max_length
    }
}

/// Shorten `text` to a single identifier in `convention`, within `budget`.
///
/// Function words are dropped ("the type of a match" → `type_match`), the word
/// list is capped, and the rendered form is escaped so it is never a bare
/// keyword and never starts with a digit. Returns an empty string only when
/// `text` carries no alphanumeric character at all, or when the budget admits
/// no characters.
#[must_use]
pub fn to_identifier(
    text: &str,
    convention: NamingConvention,
    budget: &IdentifierBudget,
) -> String {
    let tokens = vocabulary::tokenize(text);
    if tokens.is_empty() {
        return String::new();
    }
    let mut words = vocabulary::strip_words(&tokens, vocabulary::function_words());
    if words.is_empty() {
        // A phrase made only of function words ("as of the") still has to yield
        // a name: fall back to the raw tokens rather than nothing.
        words = tokens;
    }
    if convention.is_code() {
        for word in &mut words {
            // `don't` → `dont`: the apostrophe survives tokenization so the
            // seed's negation cues match, but no identifier alphabet has it.
            word.retain(|ch| ch != '\'');
        }
        words.retain(|word| !word.is_empty());
    }
    if words.is_empty() {
        return String::new();
    }
    assemble(&words, convention, budget)
}

/// Is `candidate` a legal name in `convention`?
///
/// For the code conventions: it starts with a letter or underscore, contains
/// only alphanumeric characters and underscores, is not a reserved word, and
/// respects the convention's casing (`snake_case` has no uppercase letter,
/// `SCREAMING_SNAKE_CASE` no lowercase one, `camelCase`/`PascalCase` no inner
/// underscore). Leading and trailing underscores are escape characters
/// [`to_identifier`] may add, so they are accepted in every convention.
///
/// For [`NamingConvention::CommitSubject`]: a single non-empty line that does
/// not end in a period.
#[must_use]
pub fn is_valid_identifier(candidate: &str, convention: NamingConvention) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if convention == NamingConvention::CommitSubject {
        return !candidate.contains('\n')
            && !candidate.ends_with('.')
            && candidate.trim() == candidate;
    }
    let first = candidate.chars().next().unwrap_or('0');
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    if !candidate
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '_')
    {
        return false;
    }
    if vocabulary::is_reserved_word(candidate) {
        return false;
    }
    let core = candidate.trim_matches('_');
    if core.is_empty() {
        return false;
    }
    match convention {
        NamingConvention::SnakeCase => !core.chars().any(char::is_uppercase),
        NamingConvention::ScreamingSnakeCase => !core.chars().any(char::is_lowercase),
        NamingConvention::CamelCase => {
            !core.contains('_') && !core.chars().next().is_some_and(char::is_uppercase)
        }
        NamingConvention::PascalCase => {
            !core.contains('_') && !core.chars().next().is_some_and(char::is_lowercase)
        }
        NamingConvention::CommitSubject => true,
    }
}

/// Render as many leading words as the budget allows, dropping words from the
/// tail before cutting characters.
fn assemble(words: &[String], convention: NamingConvention, budget: &IdentifierBudget) -> String {
    let mut count = words.len().min(budget.max_words.max(1));
    loop {
        let candidate = polish(&render(&words[..count], convention), convention);
        if budget.admits(&candidate) || count <= 1 {
            if budget.admits(&candidate) {
                return candidate;
            }
            // One word that overruns the character cap: cut it. Polishing the
            // cut form can append an escape character, so when that pushes the
            // result back over the cap, cut one character deeper.
            let cut = polish(&clip(&candidate, budget.max_length), convention);
            if budget.admits(&cut) || budget.max_length == 0 {
                return cut;
            }
            return polish(
                &clip(&candidate, budget.max_length.saturating_sub(1)),
                convention,
            );
        }
        count -= 1;
    }
}

fn render(words: &[String], convention: NamingConvention) -> String {
    match convention {
        NamingConvention::SnakeCase => words.join("_"),
        NamingConvention::ScreamingSnakeCase => words.join("_").to_uppercase(),
        NamingConvention::CamelCase => words
            .iter()
            .enumerate()
            .map(|(index, word)| {
                if index == 0 {
                    word.clone()
                } else {
                    capitalize(word)
                }
            })
            .collect(),
        NamingConvention::PascalCase => words.iter().map(|word| capitalize(word)).collect(),
        NamingConvention::CommitSubject => {
            let joined = words.join(" ");
            capitalize(&joined)
        }
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

/// Apply the escapes that keep a code identifier legal: a leading digit gets an
/// underscore in front, a keyword collision an underscore behind.
fn polish(candidate: &str, convention: NamingConvention) -> String {
    if !convention.is_code() {
        return candidate.trim().to_string();
    }
    let mut out = candidate.to_string();
    if out.chars().next().is_some_and(char::is_numeric) {
        out.insert(0, '_');
    }
    if vocabulary::is_reserved_word(&out) {
        out.push('_');
    }
    out
}

/// Keep the first `max` characters, then drop separators the cut left dangling.
fn clip(candidate: &str, max: usize) -> String {
    candidate
        .chars()
        .take(max)
        .collect::<String>()
        .trim_end_matches(['_', ' '])
        .to_string()
}
