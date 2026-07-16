//! The link-cli substitution query language, over text sequences and links
//! (#715).
//!
//! [`link-cli`](https://github.com/link-foundation/link-cli) expresses every
//! CRUD operation as a single substitution written as two sides:
//!
//! ```text
//! (matching pattern) (substitution pattern)
//! ```
//!
//! and documents that shape as "all CRUD operations for links using single
//! substitution operation which is turing complete" — hyperlinking "substitution
//! operation" to the Markov algorithm. That is precisely the model
//! [`crate::normal_markov`] executes, so this module is the
//! bridge: it parses the query language into a
//! [`RewriteProgram`](crate::normal_markov::RewriteProgram) and renders a
//! program back out. The query is the meta-language representation that sits
//! between a natural-language request and a harness's read/write tools, so the
//! same request lowers identically on every agentic CLI.
//!
//! ## Two operand domains, one language
//!
//! The substitution model is the operand-independent part, so the language is
//! written once and its operands are read two ways. A code file is a character
//! sequence; the associative store is a set of doublets. Both are addressed by
//! the same two-sided query, the same positional pairing, and the same ordered
//! Markov control model:
//!
//! | effect | text operands            | link operands             |
//! | ------ | ------------------------ | ------------------------- |
//! | create | `() (("new"))`           | `() ((1 1))`              |
//! | delete | `(("old")) ()`           | `((1 1)) ()`              |
//! | update | `(("old")) (("new"))`    | `((1: 1 1)) ((1: 1 2))`   |
//! | read   | `(("x")) (("x"))`        | `((1: 1 1)) ((1: 1 1))`   |
//!
//! The link column is link-cli's own documented syntax, verbatim. Creation is
//! therefore the empty sequence substituted to a non-empty one, and deletion is
//! its reverse — the two directions issue #715 requires — in either domain.
//!
//! [`parse_substitution_query`] reads the text domain and
//! [`parse_link_substitution_query`] reads the link domain. Operands pair
//! positionally, and each pair becomes one rule in Markov priority order. A side
//! written `()` distributes across the other side, which is what makes the
//! create and delete shorthands total.
//!
//! ## Recorded divergences
//!
//! Markov's terminal rules have no link-cli counterpart. Rather than invent
//! punctuation, the text dialect reuses link-cli's named-reference slot — the
//! `child` in `(child: father mother)` — so a terminal rule is
//! `(terminal: "text")`. When a side is elided the name rides the side that
//! survives. The link dialect cannot reuse that slot, because there the
//! pre-colon position is the link's index; link rules are therefore always
//! non-terminal.
//!
//! Creation may not force an index. link-cli's documented creation shorthand is
//! `() ((1 1))`, whose operands are the source and target; the index is the
//! store's to assign. Defining what `() ((5: 1 2))` should do when index 5 is
//! taken would be invention, so it is rejected instead of guessed.

mod links;
mod text;

pub use links::{
    link_substitution_effect, parse_link_substitution_query, render_link,
    render_link_substitution_query, LinkPattern, LinkRewriteOutcome, LinkRewriteProgram,
    LinkRewriteRule, LinkRewriteStep, Slot,
};
pub use text::{parse_substitution_query, render_substitution_query, substitution_effect};

/// A parse failure with a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionQueryError {
    pub message: String,
}

impl std::fmt::Display for SubstitutionQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for SubstitutionQueryError {}

fn err(message: impl Into<String>) -> SubstitutionQueryError {
    SubstitutionQueryError {
        message: message.into(),
    }
}

const TWO_SIDES: &str =
    "a substitution query needs two sides: (matching pattern) (substitution pattern)";
const MUST_QUOTE: &str = "operands must be quoted, as in (\"text\")";

fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped
}

struct Parser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> Parser<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.cursor..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.cursor += character.len_utf8();
        Some(character)
    }

    fn eat(&mut self, expected: char) -> bool {
        let found = self.peek() == Some(expected);
        if found {
            self.cursor += expected.len_utf8();
        }
        found
    }

    fn skip_whitespace(&mut self) {
        // `bump`, not `cursor += 1`: whitespace such as U+00A0 is multi-byte.
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    const fn at_end(&self) -> bool {
        self.cursor >= self.input.len()
    }

    fn parse_string(&mut self) -> Result<String, SubstitutionQueryError> {
        if !self.eat('"') {
            return Err(err(MUST_QUOTE));
        }
        let mut text = String::new();
        loop {
            match self.bump() {
                None => {
                    return Err(err(
                        "unbalanced quotes: a quoted operand is missing its closing `\"`",
                    ));
                }
                Some('"') => return Ok(text),
                Some('\\') => text.push(self.parse_escape()?),
                Some(character) => text.push(character),
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, SubstitutionQueryError> {
        match self.bump() {
            Some('\\') => Ok('\\'),
            Some('"') => Ok('"'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some(unknown) => Err(err(format!(
                "unknown escape `\\{unknown}` in a quoted operand"
            ))),
            None => Err(err(
                "unbalanced quotes: a quoted operand is missing its closing `\"`",
            )),
        }
    }
}
