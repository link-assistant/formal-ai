//! Empty-redefinition (trailing-colon) field migration for the canonical seed
//! (issue #398).
//!
//! PR #399 review (comment 4664274427, defect 4 / CI check 4): every meaning
//! header was written as a YAML-style trailing-colon line:
//!
//! ```text
//! monday:
//!   defined-by calendar_day
//!   role calendar_weekday
//! ```
//!
//! The reviewer counts each `^\s*[\w-]+:\s*$` line (428 of them) as an empty
//! redefinition: the trailing colon restates the slug as a key with no value,
//! which is not native Links Notation. In Links Notation a node is just an
//! indented name — exactly like its own `surface` / `lexeme en` children, which
//! already carry no colon. The native form drops the colon:
//!
//! ```text
//! monday
//!   defined-by calendar_day
//!   role calendar_weekday
//! ```
//!
//! This is *parse-equivalent*: `src/seed/parser.rs::parse_colon_definition`
//! turns `monday:` into `(name = "monday", id = "")`, identical to the bare
//! node `monday`. Removing the colon therefore changes bytes, not meaning — the
//! loader, solver, CLI, Telegram bot, HTTP server, and browser worker all keep
//! identical behaviour.
//!
//! The migration walks the *entire* `data/seed` tree (not one file), strips the
//! trailing colon from every `^\s*[A-Za-z0-9_.-]+:\s*$` line, then regenerates
//! the embedded browser worker fallback (`src/web/formal_ai_worker.js`) so the
//! web runtime keeps identical bytes. The transform is lossless and idempotent.
//!
//! Run with `rust-script scripts/migrate-empty-redefinition-fields.rs`
//! (std-only; can also be compiled directly with `rustc`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Meaning seed files mirrored into the browser worker fallback, in load order.
/// Mirrors `scripts/migrate-empty-facet-fields.rs` so the embed regenerates the
/// same way every migration does.
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
    let mut stripped = 0usize;
    for path in lino_files(seed_dir)? {
        let original = fs::read_to_string(&path)?;
        let (migrated, count) = migrate(&original);
        stripped += count;
        if migrated != original {
            fs::write(&path, migrated)?;
        }
    }
    refresh_worker_meanings(Path::new("src/web/formal_ai_worker.js"))?;
    println!("stripped {stripped} trailing-colon redefinition field(s)");
    Ok(())
}

/// Strip the trailing colon from every `^\s*[A-Za-z0-9_.-]+:\s*$` line. Returns
/// the rewritten text and the number of lines changed. A line with content
/// after the colon (`key: value`) or with a space before the colon
/// (`text monday:`) is never matched, so only valueless redefinition headers
/// are affected.
fn migrate(content: &str) -> (String, usize) {
    let trailing_newline = content.ends_with('\n');
    let mut out: Vec<String> = Vec::new();
    let mut stripped = 0usize;
    for line in content.lines() {
        if let Some(rewritten) = strip_trailing_colon(line) {
            out.push(rewritten);
            stripped += 1;
        } else {
            out.push(line.to_string());
        }
    }
    let mut joined = out.join("\n");
    if trailing_newline {
        joined.push('\n');
    }
    (joined, stripped)
}

/// When `line` is an empty colon redefinition (`^\s*[A-Za-z0-9_.-]+:\s*$`),
/// return the same line without the trailing colon; otherwise `None`.
fn strip_trailing_colon(line: &str) -> Option<String> {
    let indent = leading_spaces(line);
    let body = line[indent..].trim_end();
    let slug = body.strip_suffix(':')?;
    if slug.is_empty()
        || !slug
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-'))
    {
        return None;
    }
    Some(format!("{}{slug}", " ".repeat(indent)))
}

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|character| *character == ' ').count()
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
