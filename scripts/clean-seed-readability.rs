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
//! Both transforms are idempotent. After rewriting the seed the embedded browser
//! worker fallback (`src/web/formal_ai_worker.js`) is regenerated so the web
//! runtime keeps the same bytes.
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

/// Meaning seed files mirrored into the browser worker fallback, in load order.
const MEANING_SEED_FILES: &[&str] = &[
    "data/seed/meanings.lino",
    "data/seed/meanings-units.lino",
    "data/seed/meanings-calendar.lino",
    "data/seed/meanings-calculator.lino",
    "data/seed/meanings-facts.lino",
    "data/seed/meanings-software-project.lino",
    "data/seed/meanings-program-synthesis.lino",
    "data/seed/meanings-intent.lino",
    "data/seed/meanings-how.lino",
    "data/seed/meanings-meta.lino",
    "data/seed/meanings-web-navigation.lino",
    "data/seed/meanings-web-search.lino",
    "data/seed/meanings-web-search-query.lino",
    "data/seed/meanings-web-research.lino",
    "data/seed/meanings-web-followup.lino",
    "data/seed/meanings-translation.lino",
    "data/seed/meanings-ontology.lino",
    "data/seed/meanings-semantic-meta.lino",
    "data/seed/meanings-lexical-meta.lino",
    "data/seed/meanings-links-root.lino",
    "data/seed/meanings-wikidata.lino",
    "data/seed/meanings-behavior-rules.lino",
    "data/seed/meanings-proof.lino",
    "data/seed/meanings-policy.lino",
    "data/seed/meanings-docs.lino",
    "data/seed/meanings-skill-compiler.lino",
    "data/seed/meanings-finance.lino",
    "data/seed/meanings-definition-merge.lino",
    "data/seed/meanings-tool-access.lino",
    "data/seed/meanings-feature-capability.lino",
    "data/seed/meanings-playwright.lino",
    "data/seed/meanings-research-table.lino",
    "data/seed/meanings-conversation.lino",
    "data/seed/meanings-summary.lino",
    "data/seed/meanings-coding-catalog.lino",
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
    refresh_worker_meanings(Path::new("src/web/formal_ai_worker.js"))?;
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

fn refresh_worker_meanings(worker_path: &Path) -> io::Result<()> {
    if !worker_path.exists() {
        return Ok(());
    }
    let mut seed_lines = Vec::new();
    for file in MEANING_SEED_FILES {
        let content = fs::read_to_string(file)?;
        let content = content.strip_suffix('\n').unwrap_or(&content);
        seed_lines.extend(content.lines().map(ToOwned::to_owned));
    }

    let mut replacement = String::from("const MEANINGS_LINO = [\n");
    for line in seed_lines {
        replacement.push_str("  ");
        replacement.push_str(&js_string(&line));
        replacement.push_str(",\n");
    }
    replacement.push_str("].join(\"\\n\");");

    let original = fs::read_to_string(worker_path)?;
    let start = original.find("const MEANINGS_LINO = [").ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "MEANINGS_LINO start not found")
    })?;
    let end_marker = "].join(\"\\n\");";
    let end = original[start..]
        .find(end_marker)
        .map(|offset| start + offset + end_marker.len())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "MEANINGS_LINO end not found"))?;
    let mut next = String::new();
    next.push_str(&original[..start]);
    next.push_str(&replacement);
    next.push_str(&original[end..]);
    if next != original {
        fs::write(worker_path, next)?;
    }
    Ok(())
}

fn js_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character.is_control() => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
    out
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
