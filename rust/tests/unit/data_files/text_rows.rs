use std::path::Path;

pub(super) fn strip_lino_comment(line: &str) -> &str {
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
            if quote_character == '"' && character == '\\' {
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
        if matches!(character, '"' | '\'') {
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

/// The body of a line's trailing `#` comment (everything after the `#`), or
/// `None` when the line has no comment. Mirrors [`strip_lino_comment`]'s
/// quote-aware boundary detection.
pub(super) fn lino_comment_body(line: &str) -> Option<&str> {
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
            if quote_character == '"' && character == '\\' {
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
        if matches!(character, '"' | '\'') {
            quote = Some(character);
            previous_was_space = false;
            continue;
        }
        if character == '#' && previous_was_space {
            return Some(&line[index + 1..]);
        }
        previous_was_space = character.is_whitespace();
    }
    None
}

/// Remove quoted scalar spans (`"..."`, `'...'`, `` `...` ``) from a line so
/// structural assertions only inspect the Links Notation skeleton. Backslash
/// escapes are honoured inside double-quote and backtick spans (matching the
/// seed parser), so an escaped delimiter does not prematurely close the span.
pub(super) fn strip_quoted_spans(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(character) = chars.next() {
        if matches!(character, '"' | '\'' | '`') {
            let delimiter = character;
            let escapes = delimiter != '\'';
            while let Some(inner) = chars.next() {
                if escapes && inner == '\\' {
                    chars.next();
                    continue;
                }
                if inner == delimiter {
                    break;
                }
            }
            out.push(' ');
        } else {
            out.push(character);
        }
    }
    out
}

pub(super) fn wikidata_cache_path(root: &Path, id: &str, extension: &str) -> std::path::PathBuf {
    let kind = match id.chars().next() {
        Some('L') => "lexeme",
        Some('P') => "property",
        Some('Q') => "entity",
        _ => panic!("unexpected Wikidata id `{id}`"),
    };
    root.join(kind).join(format!("{id}.{extension}"))
}

pub(super) fn forbidden_meaning_key(trimmed: &str) -> bool {
    trimmed.starts_with("meaning ")
        || trimmed.starts_with("gloss ")
        || trimmed.starts_with("description ")
        || trimmed.starts_with("word ")
}

pub(super) fn path_has_component(path: &Path, component: &str) -> bool {
    path.components().any(|path_component| {
        path_component
            .as_os_str()
            .to_str()
            .is_some_and(|part| part == component)
    })
}
