use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let seed_dir = Path::new("data/seed");
    for path in lino_files(seed_dir)? {
        let original = fs::read_to_string(&path)?;
        let migrated = if is_meaning_file(&path) {
            migrate_meaning_file(&original)
        } else {
            migrate_scalar_file(&original)
        };
        if migrated != original {
            fs::write(&path, migrated)?;
        }
    }
    Ok(())
}

fn lino_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(lino_files(&path)?);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lino") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn is_meaning_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("meanings"))
}

fn migrate_meaning_file(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        let indent_len = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        let indent = &line[..indent_len];
        let trimmed = line[indent_len..].trim_end();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        if let Some(value) = quoted_value(trimmed, "meaning") {
            out.push_str(indent);
            out.push_str(&value);
            out.push_str(": # concept ");
            out.push_str(&comment_text(&value));
            out.push('\n');
        } else if quoted_value(trimmed, "gloss").is_some()
            || quoted_value(trimmed, "description").is_some()
            || quoted_value(trimmed, "wiktionary").is_some()
        {
            continue;
        } else if let Some(value) = quoted_value(trimmed, "defined_by") {
            write_safe_value_line(
                &mut out,
                indent,
                "defined-by",
                &value,
                Some("definition-link"),
            );
        } else if let Some(value) = quoted_value(trimmed, "wikidata") {
            write_safe_value_line(&mut out, indent, "grounded-in", &value, Some("source-id"));
        } else if let Some(value) = quoted_value(trimmed, "role") {
            write_safe_value_line(&mut out, indent, "role", &value, Some("semantic-role"));
        } else if let Some(value) = quoted_value(trimmed, "lexeme") {
            write_safe_value_line(&mut out, indent, "lexeme", &value, Some("language"));
        } else if let Some(value) = quoted_value(trimmed, "word") {
            write_raw_line(
                &mut out,
                indent,
                "surface",
                &value,
                Some("unresolved-surface"),
            );
        } else if let Some(value) = quoted_value(trimmed, "facet") {
            write_safe_value_line(&mut out, indent, "facet", &value, Some("facet"));
        } else if let Some(value) = quoted_value(trimmed, "action") {
            write_safe_value_line(&mut out, indent, "action", &value, Some("action"));
        } else {
            out.push_str(&migrate_scalar_line(line));
            out.push('\n');
        }
    }
    out
}

fn migrate_scalar_file(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        out.push_str(&migrate_scalar_line(line));
        out.push('\n');
    }
    out
}

fn migrate_scalar_line(line: &str) -> String {
    let indent_len = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let indent = &line[..indent_len];
    let trimmed = line[indent_len..].trim_end();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some((name, value)) = split_quoted_scalar(trimmed, '"') {
        let mut out = String::new();
        write_scalar_value(
            &mut out,
            indent,
            migrated_scalar_name(name),
            &unescape_double(&value),
        );
        return out;
    }
    if let Some((name, value)) = split_quoted_scalar(trimmed, '\'') {
        let mut out = String::new();
        write_raw_line(
            &mut out,
            indent,
            migrated_scalar_name(name),
            &unescape_single(&value),
            None,
        );
        return out.trim_end().to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("description ") {
        return format!("{indent}note {rest}");
    }
    if trimmed == "description" {
        return format!("{indent}note");
    }
    line.to_string()
}

fn migrated_scalar_name(name: &str) -> &str {
    if name == "description" {
        "note"
    } else {
        name
    }
}

fn write_scalar_value(out: &mut String, indent: &str, name: &str, value: &str) {
    if is_safe_reference(value) {
        write_safe_value_line(out, indent, name, value, None);
    } else {
        write_raw_line(out, indent, name, value, None);
    }
    if out.ends_with('\n') {
        out.pop();
    }
}

fn write_safe_value_line(
    out: &mut String,
    indent: &str,
    name: &str,
    value: &str,
    comment: Option<&str>,
) {
    out.push_str(indent);
    out.push_str(name);
    if !value.is_empty() {
        out.push(' ');
        out.push_str(value);
    }
    if let Some(comment) = comment {
        out.push_str(" # ");
        out.push_str(comment);
    }
    out.push('\n');
}

fn write_raw_line(out: &mut String, indent: &str, name: &str, value: &str, comment: Option<&str>) {
    // Issue #398: emit a human-readable quoted scalar rather than a codepoint
    // byte-dump. The seed/web/e2e parsers all decode the quoted form back to the
    // same string, so runtime values are preserved while the data stays legible.
    out.push_str(indent);
    out.push_str(name);
    if !value.is_empty() {
        out.push(' ');
        out.push_str(&quote_scalar(value));
    }
    if let Some(comment) = comment {
        out.push_str(" # ");
        out.push_str(comment);
    }
    out.push('\n');
}

/// Wrap `value` in a quote delimiter that does not occur in the text, so the
/// inner quote never needs backslash-escaping. The canonical Links Notation
/// parser mishandles a `\"` escape immediately followed by `)`, so emitting an
/// escaped delimiter must be avoided. Mirrors `quote()` in
/// `experiments/decode_seed_codepoints.py`.
fn quote_scalar(value: &str) -> String {
    let delimiter = if !value.contains('"') {
        '"'
    } else if !value.contains('\'') {
        '\''
    } else if !value.contains('`') {
        '`'
    } else {
        '"'
    };
    let mut body = value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    body = body.replace(delimiter, &format!("\\{delimiter}"));
    format!("{delimiter}{body}{delimiter}")
}

fn quoted_value(line: &str, name: &str) -> Option<String> {
    let rest = line.strip_prefix(name)?.trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let value = find_double_quote_value(rest)?;
    Some(unescape_double(value))
}

fn split_quoted_scalar(line: &str, quote: char) -> Option<(&str, String)> {
    let first_whitespace = line.find(char::is_whitespace)?;
    let name = line[..first_whitespace].trim();
    let rest = line[first_whitespace..].trim_start();
    if !rest.starts_with(quote) {
        return None;
    }
    let value = find_quoted_value(rest, quote)?;
    Some((name, value.to_string()))
}

fn find_double_quote_value(rest: &str) -> Option<&str> {
    find_quoted_value(rest, '"')
}

fn find_quoted_value(rest: &str, quote: char) -> Option<&str> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if first != quote {
        return None;
    }
    let mut escaped = false;
    for (index, character) in chars {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == quote {
            return Some(&rest[1..index]);
        }
    }
    None
}

fn unescape_double(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('"') => out.push('"'),
            Some('\\') | None => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    out
}

fn unescape_single(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') | None => out.push('\\'),
            Some('x') if chars.peek() == Some(&'2') => {
                chars.next();
                if chars.peek() == Some(&'7') {
                    chars.next();
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
    }
    out
}

fn is_safe_reference(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '|')
        })
}

fn comment_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character == '"' || character == '\'' || character == '#' {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
}
