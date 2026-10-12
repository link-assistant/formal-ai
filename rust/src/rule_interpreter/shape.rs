//! Structural shapes a `shape` condition asks about: properties of the input
//! that carry no natural language (issue #1138 B9, plan 09 leaf 9).

use super::Shape;

/// Quotation marks that open and close a span, paired.
///
/// Seed data, not prose: these are the delimiter pairs the five registered
/// languages write quotations with. A `shape quoted` asks whether a span is
/// delimited, never what it says.
const QUOTE_PAIRS: [(char, char); 6] = [
    ('"', '"'),
    ('\'', '\''),
    ('\u{ab}', '\u{bb}'),
    ('\u{201c}', '\u{201d}'),
    ('\u{300c}', '\u{300d}'),
    ('\u{2018}', '\u{2019}'),
];

impl Shape {
    /// Whether the subject text has this structural property.
    pub(super) fn holds(self, text: &str) -> bool {
        match self {
            Self::Digit => text.chars().any(char::is_numeric),
            Self::TimeSeparator => text.contains(':') || text.contains('\u{ff1a}'),
            Self::Url => text
                .split_whitespace()
                .any(|token| is_url(&token.to_lowercase())),
            Self::Path => text.split_whitespace().any(|token| {
                let lowered = token.to_lowercase();
                !is_url(&lowered) && (token.contains('/') || token.contains('\\'))
            }),
            Self::Quoted => QUOTE_PAIRS.iter().any(|(open, close)| {
                if open == close {
                    text.matches(*open).count() >= 2
                } else {
                    text.find(*open)
                        .is_some_and(|start| text[start..].chars().skip(1).any(|ch| ch == *close))
                }
            }),
        }
    }
}

/// How an absolute web address begins. Structural tokens, not prose: a scheme
/// and the conventional bare host, neither of which is written in any language.
const URL_PREFIXES: [&str; 3] = ["http://", "https://", "www."];

/// Whether one whitespace-delimited, already lowercased token is a web address.
fn is_url(token: &str) -> bool {
    URL_PREFIXES.iter().any(|prefix| token.starts_with(*prefix))
}
