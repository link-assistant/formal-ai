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
//! The migration walks the *entire* `data/seed` tree (not one file) and strips
//! the trailing colon from every `^\s*[A-Za-z0-9_.-]+:\s*$` line. Browser builds
//! read the canonical seed via `scripts/sync-seed.sh` and
//! `src/web/seed_loader.js`. The transform is lossless and idempotent.
//!
//! Run with `rust-script scripts/migrate-empty-redefinition-fields.rs`
//! (std-only; can also be compiled directly with `rustc`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
        || !slug.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-')
        })
    {
        return None;
    }
    Some(format!("{}{slug}", " ".repeat(indent)))
}

fn leading_spaces(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
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
