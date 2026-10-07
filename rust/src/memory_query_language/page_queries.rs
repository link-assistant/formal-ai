//! Page queries over formalized web pages (issue #1163 R8).
//!
//! The memory query language resolves two surfaces against the pages the
//! generic formalizer stored: "code blocks on `<domain>` whose text contains
//! `<term>`" and "the command in the paragraph that mentions `<phrase>`".
//! Both templates live in `data/seed/page-formalization-rules.lino`
//! (`page_query` records); this module only matches a query against them and
//! runs the matching [`FormalizedPageStore`] predicate, which answers with
//! `Link` values, never strings.

use std::collections::BTreeMap;

use meta_language::Link;

use crate::web_formalize::{FormalizedPageStore, page_query_templates, with_working_memory};

/// Characters a slot value may be wrapped in, trimmed before it is used.
const SLOT_QUOTES: [char; 3] = ['"', '\'', '`'];
/// Sentence punctuation a query may end with.
const QUERY_END_PUNCTUATION: [char; 3] = ['?', '.', '!'];
/// The braces that delimit a slot name in a template.
const SLOT_OPEN: char = '{';
const SLOT_CLOSE: char = '}';

/// A page query the memory query language resolves (issue #1163 R8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageQuery {
    /// Code blocks on a domain whose text contains a term.
    CodeBlocksOn { domain: String, term: String },
    /// The command tied to each paragraph that mentions a phrase.
    CommandMentioning { phrase: String },
}

impl PageQuery {
    /// Run the query against a store of formalized pages.
    #[must_use]
    pub fn execute(&self, store: &FormalizedPageStore) -> Vec<Link> {
        match self {
            Self::CodeBlocksOn { domain, term } => store.code_blocks_on(domain, term),
            Self::CommandMentioning { phrase } => store.command_mentioning(phrase),
        }
    }
}

/// Parse a query against the seed's page-query templates; `None` when no
/// template matches.
#[must_use]
pub fn parse_page_query(text: &str) -> Option<PageQuery> {
    for (name, template) in page_query_templates() {
        let Some(slots) = match_template(&template, text) else {
            continue;
        };
        let slot = |key: &str| slots.get(key).cloned().unwrap_or_default();
        match name.as_str() {
            "code_blocks_on" => {
                return Some(PageQuery::CodeBlocksOn {
                    domain: slot("domain").to_ascii_lowercase(),
                    term: slot("term"),
                });
            }
            "command_mentioning" => {
                return Some(PageQuery::CommandMentioning {
                    phrase: slot("phrase"),
                });
            }
            _ => {}
        }
    }
    None
}

/// Parse and run a page query against a store.
#[must_use]
pub fn run_page_query(store: &FormalizedPageStore, text: &str) -> Option<Vec<Link>> {
    parse_page_query(text).map(|query| query.execute(store))
}

/// Parse and run a page query against the solver's working memory of
/// formalized pages (issue #1163 R6/R8).
#[must_use]
pub fn run_page_query_in_working_memory(text: &str) -> Option<Vec<Link>> {
    let query = parse_page_query(text)?;
    Some(with_working_memory(|store| query.execute(store)))
}

/// One piece of a template: literal text or a named slot.
enum TemplatePiece {
    Literal(String),
    Slot(String),
}

/// Split a template into literal chunks and `{slot}` names.
fn template_pieces(template: &str) -> Vec<TemplatePiece> {
    let mut pieces = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find(SLOT_OPEN) {
        let Some(close) = rest[open..].find(SLOT_CLOSE).map(|offset| open + offset) else {
            break;
        };
        if open > 0 {
            pieces.push(TemplatePiece::Literal(rest[..open].to_ascii_lowercase()));
        }
        pieces.push(TemplatePiece::Slot(rest[open + 1..close].to_owned()));
        rest = &rest[close + 1..];
    }
    if !rest.is_empty() {
        pieces.push(TemplatePiece::Literal(rest.to_ascii_lowercase()));
    }
    pieces
}

/// Match a query against a template: literal chunks match ASCII
/// case-insensitively, each slot takes the text up to the next literal
/// chunk (the last slot takes the rest), and every slot must be non-empty.
fn match_template(template: &str, text: &str) -> Option<BTreeMap<String, String>> {
    let text = text
        .trim()
        .trim_end_matches(QUERY_END_PUNCTUATION)
        .trim_end();
    let lower = text.to_ascii_lowercase();
    let pieces = template_pieces(template);
    let mut slots = BTreeMap::new();
    let mut position = 0usize;
    for (index, piece) in pieces.iter().enumerate() {
        match piece {
            TemplatePiece::Literal(literal) => {
                if !lower[position..].starts_with(literal.as_str()) {
                    return None;
                }
                position += literal.len();
            }
            TemplatePiece::Slot(name) => {
                let end = match pieces.get(index + 1) {
                    Some(TemplatePiece::Literal(next)) => {
                        position + lower[position..].find(next.as_str())?
                    }
                    Some(TemplatePiece::Slot(_)) => return None,
                    None => text.len(),
                };
                let value = text[position..end].trim().trim_matches(SLOT_QUOTES).trim();
                if value.is_empty() {
                    return None;
                }
                slots.insert(name.clone(), value.to_owned());
                position = end;
            }
        }
    }
    (position == text.len()).then_some(slots)
}
