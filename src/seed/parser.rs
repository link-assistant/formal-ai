//! Tiny Links Notation parser shared by every `seed` loader.
//!
//! Seed files are indentation trees. Historical files used `name "value"`;
//! issue #398 migrates data to unquoted links with `#` comments and explicit
//! codepoint metadata for text that is not yet formalized as a lexeme id.

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
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_closing_single_quote(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'\'' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_closing_backtick(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'`' {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub fn unescape_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars();
    while let Some(c) = iter.next() {
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
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
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
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
        let close = find_closing_single_quote(rest)?;
        if rest[close + 1..].trim().is_empty() {
            return Some(unescape_single_value(&rest[..close]));
        }
    }
    if let Some(rest) = raw.strip_prefix('`') {
        let close = find_closing_backtick(rest)?;
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
        return body
            .split_whitespace()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect();
    }
    trimmed
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
