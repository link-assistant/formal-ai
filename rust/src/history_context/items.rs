//! Named top-level items and co-author trailers (issue #1180 R1/R2).
//!
//! The census histogram says how many `function_item` nodes a commit added;
//! a lineage question needs which ones. A top-level item opens at column
//! zero with an optional run of the seed's `item_modifier` words and then
//! one `item_keyword` (`fn`, `struct`, `function`, `class`, ...); its name
//! is the next identifier, or the whole header for an `item_span_keyword`
//! (`impl Display for Answer`). The item's body runs to the next top-level
//! item, so a commit that edits a function's body records it as modified.

use super::{BTreeMap, HistoryRules, SymbolChange};

/// Characters an item name is made of.
fn is_name_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '$'
}

/// The bare word a header token stands for: the text before any `(`, `<`
/// or `:` it carries (`pub(crate)` → `pub`, `impl<T>` → `impl`).
fn bare_word(token: &str) -> &str {
    token.split(['(', '<', ':']).next().unwrap_or(token)
}

/// The `keyword name` label of the item a top-level line opens, if any.
fn item_header(
    line: &str,
    keywords: &[String],
    span_keywords: &[String],
    modifiers: &[String],
) -> Option<String> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let is_keyword = |token: &str| {
        let word = bare_word(token);
        keywords.iter().any(|keyword| keyword == word)
    };
    for (index, token) in tokens.iter().enumerate() {
        let word = bare_word(token);
        if span_keywords.iter().any(|keyword| keyword == word) {
            let header = line
                .split(['{', ';'])
                .next()
                .unwrap_or(line)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let start = header.find(word)?;
            return Some(header[start..].to_owned());
        }
        if is_keyword(token) {
            // `const fn name`: a keyword followed by another keyword acts as
            // a modifier of the second.
            let next = tokens.get(index + 1)?;
            if is_keyword(next) {
                continue;
            }
            let name: String = next
                .trim_start_matches('*')
                .chars()
                .take_while(|character| is_name_character(*character))
                .collect();
            return (!name.is_empty()).then(|| [word, name.as_str()].join(" "));
        }
        if !modifiers.iter().any(|modifier| modifier == word) {
            return None;
        }
    }
    None
}

/// Every named top-level item of `source` with its body text, for the
/// `source` census id (`ast_census`, `es_meta_extract`).
#[must_use]
pub fn named_items(source: &str, census: &str, rules: &HistoryRules) -> BTreeMap<String, String> {
    let empty = Vec::new();
    let keywords = rules.item_keywords.get(census).unwrap_or(&empty);
    let span_keywords = rules.item_span_keywords.get(census).unwrap_or(&empty);
    let mut items = BTreeMap::new();
    let mut current: Option<(String, String)> = None;
    for line in source.lines() {
        if let Some(label) = item_header(line, keywords, span_keywords, &rules.item_modifiers) {
            if let Some((name, body)) = current.take() {
                items.insert(name, body);
            }
            current = Some((label, String::new()));
        }
        if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((name, body)) = current {
        items.insert(name, body);
    }
    items
}

/// The named items a change to `path` added (+1), removed (-1) or modified
/// (0), as symbol changes labelled `<path>#<keyword> <name>`.
#[must_use]
pub fn diff_named_items(
    path: &str,
    census: &str,
    before: &str,
    after: &str,
    rules: &HistoryRules,
) -> Vec<SymbolChange> {
    let before = named_items(before, census, rules);
    let after = named_items(after, census, rules);
    let change = |item: &str, delta: i64| SymbolChange {
        path: path.to_owned(),
        source: census.to_owned(),
        item: format!("{path}#{item}"),
        delta,
    };
    let mut changes = Vec::new();
    for (item, body) in &after {
        match before.get(item) {
            None => changes.push(change(item, 1)),
            Some(old) if old != body => changes.push(change(item, 0)),
            Some(_) => {}
        }
    }
    for item in before.keys().filter(|item| !after.contains_key(*item)) {
        changes.push(change(item, -1));
    }
    changes
}

/// The co-authors a commit body names on its co-author trailer lines (the
/// seed's prefix, case-insensitive), by name without the e-mail address.
/// Bodies that still carry literal `\n` escapes are split on them too.
#[must_use]
pub fn coauthors(body: &str, rules: &HistoryRules) -> Vec<String> {
    let prefix = rules.coauthor_prefix.to_lowercase();
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut names = Vec::new();
    for line in body.lines().flat_map(|line| line.split("\\n")) {
        let trimmed = line.trim();
        let Some(head) = trimmed.get(..prefix.len()) else {
            continue;
        };
        if head.to_lowercase() != prefix {
            continue;
        }
        let value = trimmed[prefix.len()..].trim();
        let name = value.split('<').next().unwrap_or(value).trim();
        if !name.is_empty() && !names.iter().any(|known: &String| known == name) {
            names.push(name.to_owned());
        }
    }
    names
}
