//! The text-sequence operand domain of the substitution query language.
//!
//! Operands are quoted character sequences, so a program lowers directly to
//! [`crate::normal_markov`]'s executor. See the [module docs](super) for how
//! this domain relates to the link one.

use std::fmt::Write as _;

use super::{err, escape, Parser, SubstitutionQueryError, MUST_QUOTE, TWO_SIDES};
use crate::normal_markov::{RewriteProgram, RewriteRule};
use crate::substitution::CrudEvent;

/// One `("text")` or `(terminal: "text")` operand on either side.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Operand {
    text: String,
    terminal: bool,
}

/// Classify what a rule does to the sequence it matches.
///
/// This is link-cli's framing of CRUD as substitution: an identity substitution
/// reads, an empty pattern creates, an empty replacement deletes.
#[must_use]
pub fn substitution_effect(rule: &RewriteRule) -> CrudEvent {
    if rule.pattern == rule.replacement {
        CrudEvent::Read
    } else if rule.pattern.is_empty() {
        CrudEvent::Create
    } else if rule.replacement.is_empty() {
        CrudEvent::Delete
    } else {
        CrudEvent::Update
    }
}

/// Parse a substitution query into a bounded normal algorithm.
///
/// `max_steps` is the caller's resource bound: the language is Turing complete,
/// so the executor — not the query — decides how long a rewrite may run.
///
/// # Errors
/// Returns [`SubstitutionQueryError`] when the two sides are malformed or their
/// operand counts cannot be paired.
pub fn parse_substitution_query(
    query: &str,
    max_steps: usize,
) -> Result<RewriteProgram, SubstitutionQueryError> {
    let mut parser = Parser::new(query);
    let matching = parser.parse_side()?;
    parser.skip_whitespace();
    if parser.at_end() {
        return Err(err(format!("{TWO_SIDES}; found only one")));
    }
    let substituting = parser.parse_side()?;
    parser.skip_whitespace();
    if !parser.at_end() {
        return Err(err(format!("{TWO_SIDES}; found trailing input")));
    }
    Ok(RewriteProgram::new(
        pair_operands(&matching, &substituting)?,
        max_steps,
    ))
}

/// Render a normal algorithm as a substitution query.
///
/// Output is canonical: [`parse_substitution_query`] round-trips it back to an
/// equal program, and link-cli's `()` shorthand is used whenever a whole side
/// is empty.
#[must_use]
pub fn render_substitution_query(program: &RewriteProgram) -> String {
    let rules = &program.rules;
    let patterns_empty = rules.iter().all(|rule| rule.pattern.is_empty());
    let replacements_empty = rules.iter().all(|rule| rule.replacement.is_empty());
    // A side may only be elided when the other side still carries the operands.
    // Without the second condition an all-`"" -> ""` program would render as
    // `() ()` and parse back as no rules at all.
    let elide_matching = !rules.is_empty() && patterns_empty && !replacements_empty;
    let elide_substituting = !rules.is_empty() && replacements_empty && !patterns_empty;
    // The terminal name must ride a side that is actually rendered.
    let name_on_matching = elide_substituting;

    let matching = if elide_matching {
        String::from("()")
    } else {
        render_side(
            rules
                .iter()
                .map(|rule| (rule.pattern.as_str(), rule.terminal && name_on_matching)),
        )
    };
    let substituting = if elide_substituting {
        String::from("()")
    } else {
        render_side(rules.iter().map(|rule| {
            (
                rule.replacement.as_str(),
                rule.terminal && !name_on_matching,
            )
        }))
    };
    format!("{matching} {substituting}")
}

fn render_side<'a>(operands: impl Iterator<Item = (&'a str, bool)>) -> String {
    let mut rendered = String::from("(");
    for (index, (text, terminal)) in operands.enumerate() {
        if index > 0 {
            rendered.push(' ');
        }
        let name = if terminal { "terminal: " } else { "" };
        let _ = write!(rendered, "({name}\"{}\")", escape(text));
    }
    rendered.push(')');
    rendered
}

/// Pair the two sides into ordered rules, distributing an elided side.
fn pair_operands(
    matching: &[Operand],
    substituting: &[Operand],
) -> Result<Vec<RewriteRule>, SubstitutionQueryError> {
    let rules = match (matching, substituting) {
        ([], []) => Vec::new(),
        ([], creations) => creations
            .iter()
            .map(|operand| rule(String::new(), operand.text.clone(), operand.terminal))
            .collect(),
        (deletions, []) => deletions
            .iter()
            .map(|operand| rule(operand.text.clone(), String::new(), operand.terminal))
            .collect(),
        (old, new) => {
            if old.len() != new.len() {
                return Err(err(format!(
                    "operand count must match across sides, or one side must be `()`; \
                     the matching side has {} and the substitution side has {}",
                    old.len(),
                    new.len()
                )));
            }
            old.iter()
                .zip(new)
                .map(|(old, new)| {
                    rule(
                        old.text.clone(),
                        new.text.clone(),
                        old.terminal || new.terminal,
                    )
                })
                .collect()
        }
    };
    Ok(rules)
}

fn rule(pattern: String, replacement: String, terminal: bool) -> RewriteRule {
    let rule = RewriteRule::new(pattern, replacement);
    if terminal {
        rule.terminal()
    } else {
        rule
    }
}

impl Parser<'_> {
    fn parse_side(&mut self) -> Result<Vec<Operand>, SubstitutionQueryError> {
        self.skip_whitespace();
        if !self.eat('(') {
            return Err(err(format!("{TWO_SIDES}; each side opens with `(`")));
        }
        let mut operands = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(')') => {
                    self.bump();
                    return Ok(operands);
                }
                Some('(') => operands.push(self.parse_operand()?),
                Some(unexpected) => {
                    return Err(err(format!("unexpected `{unexpected}`; {MUST_QUOTE}")));
                }
                None => {
                    return Err(err("unbalanced parentheses: a side is missing its `)`"));
                }
            }
        }
    }

    fn parse_operand(&mut self) -> Result<Operand, SubstitutionQueryError> {
        self.bump(); // the operand's `(`
        self.skip_whitespace();
        let terminal = match self.peek() {
            None => {
                return Err(err("unbalanced parentheses: an operand is missing its `)`"));
            }
            Some('"') => false,
            Some(_) => self.parse_operand_name()?,
        };
        let text = self.parse_string()?;
        self.skip_whitespace();
        if !self.eat(')') {
            return Err(err("unbalanced parentheses: an operand is missing its `)`"));
        }
        Ok(Operand { text, terminal })
    }

    /// Consume a `name:` prefix. `terminal` is the only recognized name, so a
    /// typo fails loudly instead of silently dropping the marker.
    fn parse_operand_name(&mut self) -> Result<bool, SubstitutionQueryError> {
        let start = self.cursor;
        while self
            .peek()
            .is_some_and(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
        {
            self.bump();
        }
        let name = self.input[start..self.cursor].to_owned();
        self.skip_whitespace();
        if name.is_empty() || !self.eat(':') {
            return Err(err(MUST_QUOTE));
        }
        if name != "terminal" {
            return Err(err(format!(
                "`{name}:` is not a recognized operand name; only `terminal:` marks a terminal rule"
            )));
        }
        self.skip_whitespace();
        Ok(true)
    }
}
