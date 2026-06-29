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
//! the *entire* `data/seed` tree (not one file). Browser builds read the
//! canonical seed via `scripts/sync-seed.sh` and `src/web/seed_loader.js`. The
//! transform is lossless and idempotent.
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
