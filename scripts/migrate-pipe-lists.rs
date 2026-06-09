//! Convert legacy `"a|b|c"` pipe-packed multi-values into the canonical
//! `("a" "b" "c")` reference-list form (issue #398, PR #399 review defect #4).
//!
//! A multi-value field must be a sequence of separate references, not a single
//! string with an in-band `|` separator. This rewrites every list-semantic
//! field whose value packs alternatives with `|` (`aliases`, `tasks`,
//! `languages`, `inputs`, `outputs`, `supported_languages`, …); it deliberately
//! leaves `code` and single/backtick-quoted prose alone, because those
//! legitimately contain `|` (e.g. Rust closures `|entry|` or `||` short-circuits).
//!
//! The transform is idempotent: a value already in `(...)` form does not start
//! with a quote, so it is skipped.
//!
//! Run with `rust-script scripts/migrate-pipe-lists.rs` (std-only; can also be
//! compiled directly with `rustc -O scripts/migrate-pipe-lists.rs`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Keywords whose value legitimately contains `|` and must never be rewritten.
/// `code` holds source listings (Rust closures `|x|`, shell pipes, `||`).
const PROSE_FIELDS: &[&str] = &["code"];

fn main() -> io::Result<()> {
    let seed_dir = Path::new("data/seed");
    let mut changed = 0usize;
    for path in lino_files(seed_dir)? {
        let original = fs::read_to_string(&path)?;
        let migrated = migrate_file(&original);
        if migrated != original {
            fs::write(&path, migrated)?;
            changed += 1;
        }
    }
    println!("migrated pipe-packed lists in {changed} file(s)");
    Ok(())
}

fn migrate_file(content: &str) -> String {
    let trailing_newline = content.ends_with('\n');
    let mut out = String::new();
    for (index, line) in content.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&migrate_line(line));
    }
    if trailing_newline {
        out.push('\n');
    }
    out
}

fn migrate_line(line: &str) -> String {
    let indent_len = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let indent = &line[..indent_len];
    let body = &line[indent_len..];

    let (code, comment) = split_comment(body);
    let code_trimmed = code.trim_end();

    let Some((keyword, value)) = code_trimmed.split_once(char::is_whitespace) else {
        return line.to_string();
    };
    if PROSE_FIELDS.contains(&keyword) {
        return line.to_string();
    }
    let value = value.trim();

    // Already a `(...)` reference list — idempotent no-op.
    if value.starts_with('(') {
        return line.to_string();
    }

    // Accept either a double-quoted scalar (`"a|b|c"`) or a bare token run
    // (`a|b|c`). Single/backtick-quoted values are left alone (e.g. `code`).
    let inner = if let Some(rest) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        rest
    } else if value.starts_with('\'') || value.starts_with('`') {
        return line.to_string();
    } else {
        value
    };
    if !inner.contains('|') {
        return line.to_string();
    }

    let items: Vec<String> = inner
        .split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(quote_item)
        .collect();
    let rebuilt = format!("{indent}{keyword} ({})", items.join(" "));
    match comment {
        Some(comment_body) => format!("{rebuilt} #{comment_body}"),
        None => rebuilt,
    }
}

fn quote_item(item: &str) -> String {
    let escaped = item.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Split a line body into its code span and the optional trailing `#` comment
/// body, mirroring the seed parser's quote-aware comment detection.
fn split_comment(body: &str) -> (&str, Option<&str>) {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut previous_was_space = true;
    let mut characters = body.char_indices().peekable();
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
            return (&body[..index], Some(&body[index + 1..]));
        }
        previous_was_space = character.is_whitespace();
    }
    (body, None)
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
