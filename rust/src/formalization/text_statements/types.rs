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
    /// Typed question obligations and exact answer evidence.
    pub dialogue: super::super::dialogue::Dialogue,
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
