//! Data-driven semantic-role order for search-fusion deformalization.

const LANGUAGE_GRAMMAR: &str = include_str!("../data/seed/search-fusion-language-grammar.lino");
const DEFAULT_ORDER: [&str; 3] = ["subject", "predicate", "object"];

/// Return the semantic-role order declared for `language`, or the declared
/// fallback order when no language-specific row exists.
#[must_use]
pub fn role_order(language: &str) -> [&'static str; 3] {
    let mut fallback = DEFAULT_ORDER;
    let mut current_language = "";
    for raw in LANGUAGE_GRAMMAR.lines() {
        let indent = raw.len() - raw.trim_start().len();
        let line = raw.trim();
        if indent == 2 && line.starts_with("fallback_order ") {
            fallback = parse_order(line[15..].trim()).unwrap_or(DEFAULT_ORDER);
        } else if indent == 2 && line.starts_with("language ") {
            current_language = line[9..].trim();
        } else if indent == 4 && line.starts_with("order ") && current_language == language {
            return parse_order(line[6..].trim()).unwrap_or(fallback);
        }
    }
    fallback
}

/// The exact Agent-authored policy consumed by native and browser fusion.
#[must_use]
pub const fn policy_document() -> &'static str {
    LANGUAGE_GRAMMAR
}

fn parse_order(value: &'static str) -> Option<[&'static str; 3]> {
    let unquoted = value.strip_prefix('"')?.strip_suffix('"')?;
    let mut roles = unquoted.split_whitespace();
    let order = [roles.next()?, roles.next()?, roles.next()?];
    roles.next().is_none().then_some(order)
}
