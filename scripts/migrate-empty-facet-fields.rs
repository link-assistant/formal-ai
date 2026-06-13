//! Empty-redefinition field migration for the canonical seed (issue #398).
//!
//! PR #399 review (comment 4663407299, defect 1): semantic facets were written
//! as a `facet <kind>` wrapper whose single child was an *empty colon*
//! redefinition of a meaning already defined elsewhere:
//!
//! ```text
//! facet notation
//!   word_surface:
//! facet denotation
//!   lexical_sense:
//! ```
//!
//! `word_surface:` / `lexical_sense:` are valueless `concept:` lines — exactly
//! the shape the review bans. The native Links Notation form is two references
//! on one line (`subject predicate`):
//!
//! ```text
//! notation word_surface
//! denotation lexical_sense
//! ```
//!
//! This migration collapses every `facet <kind>` block whose children are empty
//! colon targets into direct `<kind> <target>` subject-predicate lines, across
//! the *entire* `data/seed` tree (not one file), then regenerates the embedded
//! browser worker fallback (`src/web/formal_ai_worker.js`) so the web runtime
//! keeps identical bytes. The transform is lossless and idempotent.
//!
//! Run with `rust-script scripts/migrate-empty-facet-fields.rs` (std-only; can
//! also be compiled directly with `rustc`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The closed facet vocabulary. A `facet <kind>` wrapper is only collapsed when
/// `<kind>` is one of these; `src/seed/meanings.rs` recognises the same set as
/// direct subject-predicate facet lines.
const FACET_KINDS: &[&str] = &[
    "notation",
    "annotation",
    "denotation",
    "connotation",
    "part_of_speech",
    "self-equation",
];

/// Meaning seed files mirrored into the browser worker fallback, in load order.
/// Mirrors `scripts/clean-seed-readability.rs` so the embed regenerates the same
/// way every migration does.
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
    let mut collapsed = 0usize;
    for path in lino_files(seed_dir)? {
        let original = fs::read_to_string(&path)?;
        let (migrated, count) = migrate(&original);
        collapsed += count;
        if migrated != original {
            fs::write(&path, migrated)?;
        }
    }
    refresh_worker_meanings(Path::new("src/web/formal_ai_worker.js"))?;
    println!("collapsed {collapsed} empty-redefinition facet field(s)");
    Ok(())
}

/// Collapse every `facet <kind>` block of empty colon targets into direct
/// `<kind> <target>` lines. Returns the rewritten text and the number of
/// collapsed targets.
fn migrate(content: &str) -> (String, usize) {
    let trailing_newline = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut collapsed = 0usize;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if let Some((indent, kind)) = facet_header(line) {
            let mut targets = Vec::new();
            let mut cursor = index + 1;
            while cursor < lines.len() {
                let child = lines[cursor];
                if child.trim().is_empty() {
                    break;
                }
                let child_indent = leading_spaces(child);
                if child_indent <= indent {
                    break;
                }
                let Some(target) = empty_colon_target(child) else {
                    break;
                };
                // Only direct children (one level deeper) are facet targets.
                if child_indent != indent + 2 {
                    break;
                }
                targets.push(target);
                cursor += 1;
            }
            if !targets.is_empty() {
                let pad = " ".repeat(indent);
                for target in &targets {
                    out.push(format!("{pad}{kind} {target}"));
                }
                collapsed += targets.len();
                index = cursor;
                continue;
            }
        }
        out.push(line.to_string());
        index += 1;
    }
    let mut joined = out.join("\n");
    if trailing_newline {
        joined.push('\n');
    }
    (joined, collapsed)
}

/// A `facet <kind>` header line; returns `(indent, kind)` when `kind` is in the
/// closed facet vocabulary.
fn facet_header(line: &str) -> Option<(usize, &str)> {
    let indent = leading_spaces(line);
    let body = line[indent..].trim_end();
    let kind = body.strip_prefix("facet ")?.trim();
    if kind.is_empty() || kind.contains(char::is_whitespace) {
        return None;
    }
    FACET_KINDS.contains(&kind).then_some((indent, kind))
}

/// An empty colon redefinition (`word_surface:`); returns the slug.
fn empty_colon_target(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let slug = trimmed.strip_suffix(':')?;
    if slug.is_empty() || slug.contains(char::is_whitespace) {
        return None;
    }
    Some(slug)
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
