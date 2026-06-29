//! Readability migration for the canonical Links Notation seed (issue #398).
//!
//! Two purely mechanical, lossless clean-ups requested in PR #399 review:
//!
//! 1. Drop the synthetic `seed-surface-<hash>` ids. A surface is the text (and
//!    facets) recorded under a language; the opaque hashed id carried no meaning
//!    and existed only to give the node an id. The runtime never read it (the
//!    surface text comes from the nested `text` child), so removing it leaves a
//!    valueless `surface` parent that every parser already accepts.
//!
//! 2. Strip keyword-restating "noise" comments. A trailing `# language`,
//!    `# definition-link`, `# semantic-role`, `# facet`, `# seed lexical surface`,
//!    `# source-id`, or `# action` merely repeats the keyword already on the
//!    line and adds nothing. Comments that carry the human meaning of an opaque
//!    id (`Q146786 # plural`, `# concept not`, `# wikidata ...`) are preserved.
//!
//! Both transforms are idempotent. Browser builds read the canonical seed via
//! `scripts/sync-seed.sh` and `src/web/seed_loader.js`, so this script does not
//! produce a JavaScript seed copy.
//!
//! Run with `rust-script scripts/clean-seed-readability.rs` (std-only; can also
//! be compiled directly with `rustc`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Comment bodies that only restate the keyword already on the line.
const NOISE_COMMENTS: &[&str] = &[
    "language",
    "definition-link",
    "semantic-role",
    "facet",
    "seed lexical surface",
    "source-id",
    "action",
];

fn main() -> io::Result<()> {
    let seed_dir = Path::new("data/seed");
    for path in lino_files(seed_dir)? {
        let original = fs::read_to_string(&path)?;
        let cleaned = clean_file(&original);
        if cleaned != original {
            fs::write(&path, cleaned)?;
        }
    }
    Ok(())
}

fn clean_file(content: &str) -> String {
    let trailing_newline = content.ends_with('\n');
    let mut out = String::new();
    for (index, line) in content.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&clean_line(line));
    }
    if trailing_newline {
        out.push('\n');
    }
    out
}

fn clean_line(line: &str) -> String {
    let indent_len = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let indent = &line[..indent_len];
    let body = &line[indent_len..];

    let (code, comment) = split_comment(body);
    let code_trimmed = code.trim_end();

    // 1. Synthetic surface id -> valueless `surface`.
    if let Some(rest) = code_trimmed.strip_prefix("surface ") {
        if rest.trim_start().starts_with("seed-surface-") {
            return format!("{indent}surface");
        }
    }

    // 2. Drop a keyword-restating noise comment; keep meaningful ones.
    if let Some(comment_body) = comment {
        if NOISE_COMMENTS.contains(&comment_body.trim()) {
            return format!("{indent}{code_trimmed}");
        }
    }

    line.to_string()
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
