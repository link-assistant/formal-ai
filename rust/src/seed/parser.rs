//! Tiny Links Notation parser shared by every `seed` loader.
//!
//! Seed files are indentation trees. Historical files used `name "value"`;
//! issue #398 migrates data to unquoted links with `#` comments and explicit
//! codepoint metadata for text that is not yet formalized as a lexeme id.
//!
//! Quoting follows Links Notation: a delimiter inside a value is *doubled*. The
//! backslash escapes below are a historical dialect the corpus still carries,
//! and are read alongside it.

extern crate alloc;

use alloc::borrow::{Cow, ToOwned};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::iter::Peekable;
use core::str::Chars;

#[derive(Debug, Default, Clone)]
pub struct LinoNode {
    pub name: String,
    pub id: String,
    pub children: Vec<Self>,
}

impl LinoNode {
    #[must_use]
    pub fn find_child_value(&self, name: &str) -> &str {
        for child in &self.children {
            if child.name == name {
                return &child.id;
            }
        }
        ""
    }
}

#[must_use]
fn parse_lino_unlowered(text: &str) -> LinoNode {
    let mut root = LinoNode::default();
    let mut stack: Vec<(Option<usize>, Vec<usize>)> = vec![(None, Vec::new())];
    for line in expand_concise_lexemes(text).lines() {
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

/// Expand the concise lexeme form to the long form before parsing.
///
/// PR #1188, R1188-U7 (`docs/links-notation-style.md`): the words after the
/// language of `lexeme en "read" "read the file"`, then the words of each
/// `words` child, become one `surface` each, in order, with a `text` line and
/// the lexeme's other children as its fields; an explicit `surface` child
/// stays as written. The expansion is textual, so every loader sees the long
/// form. Mirrors `expandConciseLexemes` in `js/seed_loader.js`.
#[must_use]
pub fn expand_concise_lexemes(text: &str) -> Cow<'_, str> {
    if !text.lines().any(is_concise_hint) {
        return Cow::Borrowed(text);
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let Some((indent, language, rest)) = lexeme_head(line) else {
            out.push(line.to_owned());
            index += 1;
            continue;
        };
        let mut end = index + 1;
        while end < lines.len()
            && (is_blank_line(lines[end]) || leading_spaces(lines[end]) > indent)
        {
            end += 1;
        }
        while end > index + 1 && is_blank_line(lines[end - 1]) {
            end -= 1;
        }
        let inline = word_tokens(rest);
        let children = child_blocks(&lines[index + 1..end]);
        if inline.is_empty() && !children.iter().any(ChildBlock::is_words) {
            out.push(line.to_owned());
            index += 1;
            continue;
        }
        let step = children
            .first()
            .filter(|child| child.depth > indent)
            .map_or(2, |child| child.depth - indent);
        let fields: Vec<&ChildBlock<'_>> = children
            .iter()
            .filter(|child| !child.is_words() && !child.is_surface())
            .collect();
        let emit_word = |out: &mut Vec<String>, word: &str| {
            out.push(alloc::format!("{}{SURFACE}", " ".repeat(indent + step)));
            out.push(alloc::format!(
                "{}text {word}",
                " ".repeat(indent + 2 * step)
            ));
            for field in &fields {
                out.extend(field.reindented(indent + 2 * step));
            }
        };
        out.push(alloc::format!("{}{LEXEME} {language}", " ".repeat(indent)));
        for &word in &inline {
            emit_word(&mut out, word);
        }
        for child in &children {
            if child.is_words() {
                for word in word_tokens(&child.content[WORDS.len()..]) {
                    emit_word(&mut out, word);
                }
            } else if child.is_surface() {
                out.extend(child.reindented(indent + step));
            }
        }
        index = end;
    }
    Cow::Owned(out.join("\n"))
}

/// The link names of the concise lexeme form.
const LEXEME: &str = "lexeme";
const WORDS: &str = "words";
const SURFACE: &str = "surface";

/// Whether `content` opens with the link name `word`, as a whole token.
fn opens_with(content: &str, word: &str) -> bool {
    content
        .strip_prefix(word)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

/// Whether a line may open or continue a concise lexeme: `lexeme` with words
/// after its language, or a `words` line.
fn is_concise_hint(line: &str) -> bool {
    let content = strip_comment(line).trim_start();
    opens_with(content, WORDS)
        || (opens_with(content, LEXEME) && content.split_whitespace().count() >= 3)
}

/// The indentation, language and comment-free tail of a `lexeme` line.
fn lexeme_head(line: &str) -> Option<(usize, &str, &str)> {
    if is_blank_line(line) {
        return None;
    }
    let content = strip_comment(line);
    let indent = leading_spaces(content);
    let rest = content[indent..].strip_prefix(LEXEME)?;
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let rest = rest.trim_start_matches([' ', '\t']);
    let language_end = rest
        .find(|character: char| character.is_whitespace() || character == '#')
        .unwrap_or(rest.len());
    if language_end == 0 {
        return None;
    }
    Some((indent, &rest[..language_end], &rest[language_end..]))
}

fn leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_blank_line(line: &str) -> bool {
    strip_comment(line).trim().is_empty()
}

/// The word tokens of a comment-free line tail: a quoted word keeps its
/// quotes, so `text <word>` reads it exactly as written; a bare word ends at
/// whitespace.
fn word_tokens(rest: &str) -> Vec<&str> {
    let bytes = rest.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        let start = index;
        let quote = bytes[index];
        if matches!(quote, b'"' | b'\'' | b'`') {
            index += 1;
            while index < bytes.len() {
                // A backslash escape and a doubled quote each span two bytes.
                if bytes[index] == b'\\'
                    || (bytes[index] == quote && bytes.get(index + 1) == Some(&quote))
                {
                    index += 2;
                } else if bytes[index] == quote {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
        } else {
            while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }
        }
        tokens.push(&rest[start..index.min(bytes.len())]);
    }
    tokens
}

/// One direct child of a lexeme block, with the lines of its subtree.
struct ChildBlock<'text> {
    content: &'text str,
    depth: usize,
    lines: Vec<&'text str>,
}

impl ChildBlock<'_> {
    fn is_words(&self) -> bool {
        opens_with(self.content, WORDS)
    }

    fn is_surface(&self) -> bool {
        opens_with(self.content, SURFACE)
    }

    /// The subtree moved to `depth`, its inner indentation kept.
    fn reindented(&self, depth: usize) -> impl Iterator<Item = String> + '_ {
        self.lines.iter().map(move |line| {
            let own = leading_spaces(line);
            alloc::format!("{}{}", " ".repeat(own - self.depth + depth), &line[own..])
        })
    }
}

/// The direct children of a block; the first child line sets their depth.
fn child_blocks<'text>(lines: &[&'text str]) -> Vec<ChildBlock<'text>> {
    let mut children: Vec<ChildBlock<'text>> = Vec::new();
    let mut child_indent = None;
    for line in lines {
        if is_blank_line(line) {
            continue;
        }
        let depth = leading_spaces(line);
        let first = *child_indent.get_or_insert(depth);
        match children.last_mut() {
            Some(last) if depth > first => last.lines.push(line),
            _ => children.push(ChildBlock {
                content: strip_comment(line).trim(),
                depth,
                lines: vec![line],
            }),
        }
    }
    children
}

/// Lower owned shared lexical facets before typed seed consumers read surfaces.
/// Unknown syntax refuses the whole document; legacy trees stay unchanged.
pub fn lower_shared_lexeme_fields(root: &mut LinoNode) -> Result<(), &'static str> {
    fn stable_identity(value: &str) -> bool {
        let mut bytes = value.bytes();
        bytes.next().is_some_and(|first| first.is_ascii_lowercase())
            && bytes.all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            })
    }
    fn marker(node: &LinoNode) -> bool {
        node.name == "shared-lexeme-fields" || node.name == "use-lexeme-fields"
    }
    fn contains_marker(node: &LinoNode) -> bool {
        marker(node) || node.children.iter().any(contains_marker)
    }
    if !contains_marker(root) {
        return Ok(());
    }
    // Rust parse_lino keeps a synthetic document root, unlike the JS API.
    for container in &mut root.children {
        if !contains_marker(container) {
            continue;
        }
        if container.name != "meanings" {
            return Err("shared fields require meanings root");
        }
        for meaning in &mut container.children {
            let owner = if meaning.name == "meaning" {
                &meaning.id
            } else {
                &meaning.name
            };
            if contains_marker(meaning)
                && (!stable_identity(owner)
                    || matches!(
                        meaning.name.as_str(),
                        "meanings"
                            | "lexeme"
                            | "surface"
                            | "shared-lexeme-fields"
                            | "use-lexeme-fields"
                            | "part_of_speech"
                            | "grammatical_number"
                    ))
            {
                return Err("reserved or invalid shared field owner");
            }
            let declarations: Vec<usize> = meaning
                .children
                .iter()
                .enumerate()
                .filter(|(_, node)| node.name == "shared-lexeme-fields")
                .map(|(index, _)| index)
                .collect();
            if declarations.is_empty() {
                if contains_marker(meaning) {
                    return Err("unresolved shared fields");
                }
                continue;
            }
            if declarations.len() != 1 {
                return Err("duplicate shared declaration");
            }
            let declaration_index = declarations[0];
            let declaration = meaning.children[declaration_index].clone();
            if !stable_identity(&declaration.id) {
                return Err("shared declaration requires identity");
            }
            let fields = &declaration.children;
            if fields.is_empty() {
                return Err("empty shared fields");
            }
            for (index, field) in fields.iter().enumerate() {
                if !matches!(field.name.as_str(), "part_of_speech" | "grammatical_number")
                    || !stable_identity(&field.id)
                    || !field.children.is_empty()
                {
                    return Err("unknown, nested or effect-bearing shared field");
                }
                if fields[..index].iter().any(|other| other.name == field.name) {
                    return Err("duplicate shared fields");
                }
            }
            let languages: Vec<&str> = meaning
                .children
                .iter()
                .filter(|node| node.name == "lexeme")
                .map(|node| node.id.as_str())
                .collect();
            if languages.len() < 2 {
                return Err("shared fields require multiple languages");
            }
            for (index, language) in languages.iter().enumerate() {
                if !stable_identity(language) || languages[..index].contains(language) {
                    return Err("duplicate language");
                }
            }
            for (index, node) in meaning.children.iter_mut().enumerate() {
                if index == declaration_index {
                    continue;
                }
                if node.name != "lexeme" {
                    if contains_marker(node) {
                        return Err("invalid shared declaration scope");
                    }
                    continue;
                }
                if node.children.is_empty() {
                    return Err("shared lexeme has no surfaces");
                }
                for surface in &mut node.children {
                    if surface.name != "surface" {
                        return Err("shared reference requires surface");
                    }
                    let references: Vec<usize> = surface
                        .children
                        .iter()
                        .enumerate()
                        .filter(|(_, field)| field.name == "use-lexeme-fields")
                        .map(|(index, _)| index)
                        .collect();
                    if references.len() != 1 {
                        return Err("unknown or duplicate reference");
                    }
                    let reference_index = references[0];
                    let reference = &surface.children[reference_index];
                    if reference.id != declaration.id || !reference.children.is_empty() {
                        return Err("unknown or recursive reference");
                    }
                    for (index, field) in surface.children.iter().enumerate() {
                        if index != reference_index
                            && (contains_marker(field)
                                || fields.iter().any(|shared| shared.name == field.name))
                        {
                            return Err("shared field conflict or nested reference");
                        }
                    }
                    // Clone each occurrence: fields never share mutable ownership.
                    surface
                        .children
                        .splice(reference_index..=reference_index, fields.iter().cloned());
                }
            }
            meaning.children.remove(declaration_index);
        }
    }
    Ok(())
}

/// Checked owned notation entrypoint with explicit shared-field diagnostics.
pub fn parse_lino_checked(text: &str) -> Result<LinoNode, &'static str> {
    validate_shared_lexeme_source(text)?;
    let mut root = parse_lino_unlowered(text);
    lower_shared_lexeme_fields(&mut root)?;
    Ok(root)
}

/// Legacy callers refuse the entire malformed shared document, never a partial tree.
#[must_use]
pub fn parse_lino(text: &str) -> LinoNode {
    parse_lino_checked(text).expect("invalid owned shared lexical field declaration")
}

fn validate_shared_lexeme_source(text: &str) -> Result<(), &'static str> {
    let has_shared = text.lines().any(|line| {
        matches!(
            strip_comment(line).split_whitespace().next(),
            Some("shared-lexeme-fields" | "use-lexeme-fields")
        )
    });
    if !has_shared {
        return Ok(());
    }
    let mut stack: Vec<(usize, &str)> = Vec::new();
    for line in text.lines() {
        let content = strip_comment(line);
        let tokens: Vec<&str> = content.split_whitespace().collect();
        let Some(head) = tokens.first().copied() else {
            continue;
        };
        let indent = line
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();
        if line.chars().take(indent).any(|character| character != ' ') {
            return Err("shared source indentation requires spaces");
        }
        while stack.last().is_some_and(|(depth, _)| *depth >= indent) {
            stack.pop();
        }
        let parent = stack.last().map_or("", |(_, name)| *name);
        let identity_valid = tokens.len() == 2
            && tokens[1]
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
            && tokens[1].bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            });
        if matches!(head, "shared-lexeme-fields" | "use-lexeme-fields") {
            if !identity_valid {
                return Err("invalid shared source identity");
            }
            if head == "use-lexeme-fields"
                && (stack.len() < 4
                    || parent != "surface"
                    || stack[stack.len() - 2].1 != "lexeme"
                    || stack[stack.len() - 4].1 != "meanings")
            {
                return Err("shared references require explicit surface source scope");
            }
        }
        if parent == "shared-lexeme-fields"
            && (!matches!(head, "part_of_speech" | "grammatical_number") || !identity_valid)
        {
            return Err("shared fields require bare semantic facet references");
        }
        stack.push((indent, head));
    }
    Ok(())
}

#[cfg(test)]
mod shared_lexeme_fields_tests {
    use super::{parse_lino, parse_lino_checked};

    const SHARED: &str = "meanings\n  example\n    shared-lexeme-fields lexical\n      part_of_speech noun\n    lexeme en\n      surface\n        text first\n        use-lexeme-fields lexical\n    lexeme es\n      surface\n        text second\n        use-lexeme-fields lexical\n";
    const LEGACY: &str = "meanings\n  example\n    lexeme en\n      surface\n        text first\n        part_of_speech noun\n    lexeme es\n      surface\n        text second\n        part_of_speech noun\n";

    #[test]
    fn shared_fields_preserve_complete_legacy_tree_and_combined_roots() {
        let actual = parse_lino_checked(SHARED).expect("valid shared fields");
        let expected = parse_lino_checked(LEGACY).expect("valid legacy tree");
        assert_eq!(alloc::format!("{actual:?}"), alloc::format!("{expected:?}"));
        let combined = alloc::format!("{LEGACY}{SHARED}");
        let original = alloc::format!("{LEGACY}{LEGACY}");
        assert_eq!(
            alloc::format!("{:?}", parse_lino_checked(&combined)),
            alloc::format!("{:?}", parse_lino_checked(&original))
        );
    }

    #[test]
    fn malformed_shared_fields_refuse_whole_documents() {
        for invalid in [
            SHARED.replace("use-lexeme-fields lexical", "use-lexeme-fields missing"),
            SHARED.replace("part_of_speech noun", "effect mutation"),
            SHARED.replace("lexeme es", "lexeme en"),
            SHARED.replace("meanings\n", "other-root\n"),
        ] {
            assert!(parse_lino_checked(&invalid).is_err());
            assert!(std::panic::catch_unwind(|| parse_lino(&invalid)).is_err());
        }
    }
}
