//! Tiny Links Notation parser shared by every `seed` loader.
//!
//! Seed files are indentation trees. Historical files used `name "value"`;
//! issue #398 migrates data to unquoted links with `#` comments and explicit
//! codepoint metadata for text that is not yet formalized as a lexeme id.
//!
//! Quoting follows Links Notation: a delimiter inside a value is *doubled*. The
//! backslash escapes below are a historical dialect the corpus still carries,
//! and are read alongside it.

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Default, Clone)]
pub struct LinoNode {
    pub name: String,
    pub id: String,
    pub children: Vec<Self>,
}

impl LinoNode {
    pub fn find_child_value(&self, name: &str) -> &str {
        for child in &self.children {
            if child.name == name {
                return &child.id;
            }
        }
        ""
    }
}

pub fn parse_lino(text: &str) -> LinoNode {
    let mut root = LinoNode::default();
    let mut stack: Vec<(Option<usize>, Vec<usize>)> = vec![(None, Vec::new())];
    for line in text.lines() {
        if strip_comment(line).trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        let Some(node) = parse_lino_line(content) else {
            continue;
        };
        while stack.len() > 1
            && stack
                .last()
                .and_then(|s| s.0)
                .is_some_and(|top| top >= indent)
        {
            stack.pop();
        }
        let parent_path = stack.last().map(|s| s.1.clone()).unwrap_or_default();
        let parent = navigate_mut(&mut root, &parent_path);
        parent.children.push(node);
        let new_index = parent.children.len() - 1;
        let mut new_path = parent_path;
        new_path.push(new_index);
        stack.push((Some(indent), new_path));
    }
    root
}

fn navigate_mut<'a>(root: &'a mut LinoNode, path: &[usize]) -> &'a mut LinoNode {
    let mut current = root;
    for &index in path {
        current = &mut current.children[index];
    }
    current
}

fn parse_lino_line(content: &str) -> Option<LinoNode> {
    let mut node = LinoNode::default();
    let content = strip_comment(content).trim();
    if content.is_empty() {
        return None;
    }

    if let Some((name, id)) = parse_colon_definition(content) {
        node.name = name.to_string();
        node.id = decode_raw_reference_preserving_quotes(id);
    } else if let Some((name, id)) = content.split_once(char::is_whitespace) {
        node.name = name.trim().to_string();
        node.id = decode_raw_reference(id.trim());
    } else {
        node.name = content.trim().to_string();
    }
    Some(node)
}

fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    let mut previous_was_space = true;
    let mut characters = line.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if let Some(quote_character) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if (quote_character == '"' || quote_character == '`') && character == '\\' {
                escaped = true;
                continue;
            }
            if quote_character == '\''
                && character == '\''
                && characters.peek().is_some_and(|(_, next)| *next == '\'')
            {
                characters.next();
                continue;
            }
            if character == quote_character {
                quote = None;
            }
            continue;
        }

        if matches!(character, '"' | '\'' | '`') {
            quote = Some(character);
            previous_was_space = false;
            continue;
        }
        if character == '#' && previous_was_space {
            return &line[..index];
        }
        previous_was_space = character.is_whitespace();
    }
    line
}

fn parse_colon_definition(content: &str) -> Option<(&str, &str)> {
    let first_whitespace = content.find(char::is_whitespace);
    let colon = content.find(':')?;
    if first_whitespace.is_some_and(|space| space < colon) {
        return None;
    }
    let name = content[..colon].trim();
    if name.is_empty() {
        return None;
    }
    Some((name, content[colon + 1..].trim()))
}

pub fn find_closing_quote(rest: &str) -> Option<usize> {
    find_closing_delimiter(rest, b'"')
}

/// Find the quote that closes a value opened with `quote`.
///
/// Links Notation escapes a delimiter by *doubling* it, so a doubled quote is
/// part of the value rather than its end. `strip_comment` already reads it that
/// way; this did not, so a value carrying the delimiter had no closing quote on
/// its own line, failed to decode, and fell back to raw text — quotes and
/// doubling still in it. The corpus writes that form (`the subject''s name` in
/// `data/cache/wikidata/property/P138.lino`), so the two had to agree.
fn find_closing_delimiter(rest: &str, quote: u8) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            if bytes.get(i + 1) == Some(&quote) {
                i += 2;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}
/// Collapse a delimiter the writer doubled, per Links Notation.
///
/// Returns `true` when `c` opened a doubled pair and the pair was consumed.
fn take_doubled(out: &mut String, c: char, quote: char, iter: &mut Peekable<Chars<'_>>) -> bool {
    if c != quote || iter.peek() != Some(&quote) {
        return false;
    }
    iter.next();
    out.push(quote);
    true
}

/// Decode a double-quoted value, reading both escapes this repository writes:
/// Links Notation's own doubled delimiter, and the backslash dialect
/// [`crate::links_format::sanitize_lino_value`] emits for this line-based reader.
///
/// The `t` and `r` arms exist because that writer emits `\t` and `\r`. Without
/// them a tab was written and then read back as the two characters `\` and `t`,
/// which is silent corruption of exactly the content issue #715 is about: a
/// Makefile recipe line is *required* to begin with a tab, and Go is
/// conventionally tab-indented, so a substitution rule derived from such a
/// fragment round-tripped into one that no longer matched the code it came from.
///
/// The catch-all is load-bearing and stays: it passes an unknown escape through
/// with its backslash, which is what lets a value carrying prose or LaTeX
/// (`\ldots`) survive a reader that has no arm for it.
pub fn unescape_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars().peekable();
    while let Some(c) = iter.next() {
        if take_doubled(&mut out, c, '"', &mut iter) {
            continue;
        }
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescape_single_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars().peekable();
    while let Some(c) = iter.next() {
        if take_doubled(&mut out, c, '\'', &mut iter) {
            continue;
        }
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') | None => out.push('\\'),
                Some('x') if iter.peek() == Some(&'2') => {
                    iter.next();
                    if iter.peek() == Some(&'7') {
                        iter.next();
                        out.push('\'');
                    } else {
                        out.push_str("\\x2");
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn decode_raw_reference(raw: &str) -> String {
    if let Some(value) = decode_codepoint_reference(raw) {
        return value;
    }
    if let Some(value) = decode_quoted_reference(raw) {
        return value;
    }
    raw.to_string()
}

fn decode_raw_reference_preserving_quotes(raw: &str) -> String {
    decode_codepoint_reference(raw).unwrap_or_else(|| raw.to_string())
}

fn decode_codepoint_reference(raw: &str) -> Option<String> {
    if raw == "codepoints" || raw == "unformalized-raw" {
        return Some(String::new());
    }
    if let Some(codepoints) = raw.strip_prefix("codepoints ") {
        return Some(decode_codepoints(codepoints));
    }
    if let Some(codepoints) = raw.strip_prefix("unformalized-raw ") {
        return Some(decode_codepoints(codepoints));
    }
    None
}

fn decode_quoted_reference(raw: &str) -> Option<String> {
    if let Some(rest) = raw.strip_prefix('"') {
        let close = find_closing_quote(rest)?;
        if rest[close + 1..].trim().is_empty() {
            return Some(unescape_value(&rest[..close]));
        }
    }
    if let Some(rest) = raw.strip_prefix('\'') {
        let close = find_closing_delimiter(rest, b'\'')?;
        if rest[close + 1..].trim().is_empty() {
            return Some(unescape_single_value(&rest[..close]));
        }
    }
    if let Some(rest) = raw.strip_prefix('`') {
        let close = find_closing_delimiter(rest, b'`')?;
        if rest[close + 1..].trim().is_empty() {
            return Some(unescape_value(&rest[..close]));
        }
    }
    None
}

pub fn decode_codepoints(raw: &str) -> String {
    raw.split_whitespace()
        .filter_map(|token| char::from_u32(parse_codepoint(token)))
        .collect()
}

pub fn escape_value(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn parse_codepoint(value: &str) -> u32 {
    let trimmed = value.trim();
    trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .map_or_else(
            || trimmed.parse::<u32>().unwrap_or(0),
            |stripped| u32::from_str_radix(stripped, 16).unwrap_or(0),
        )
}

pub fn split_pipe_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Some(body) = trimmed
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return split_reference_tokens(body);
    }
    trimmed
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Tokenize the body of a `(a "b c" d)` reference list. Each item is either a
/// quoted scalar (`"…"`, `'…'`, or `` `…` ``, which may contain spaces) or a
/// bare whitespace-delimited token. This is the canonical multi-value form that
/// replaced the legacy `"a|b|c"` pipe packing.
fn split_reference_tokens(body: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&character) = chars.peek() {
        if character.is_whitespace() {
            chars.next();
            continue;
        }
        if matches!(character, '"' | '\'' | '`') {
            let quote = character;
            chars.next();
            let mut value = String::new();
            let mut escaped = false;
            for current in chars.by_ref() {
                if escaped {
                    value.push(current);
                    escaped = false;
                    continue;
                }
                if (quote == '"' || quote == '`') && current == '\\' {
                    escaped = true;
                    continue;
                }
                if current == quote {
                    break;
                }
                value.push(current);
            }
            tokens.push(value);
        } else {
            let mut value = String::new();
            while let Some(&current) = chars.peek() {
                if current.is_whitespace() {
                    break;
                }
                value.push(current);
                chars.next();
            }
            if !value.is_empty() {
                tokens.push(value);
            }
        }
    }
    tokens
}
