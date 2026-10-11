#!/usr/bin/env rust-script
//! Generate the requirement-status ledger from requirement shards.
//!
//! The ledger is sharded because a record for every requirement cannot fit the
//! repository's 1,500-line data-file limit. It is sharded the way the
//! requirements are (R1188-U5): one ledger file per requirement shard, named
//! after it, so a file says whose requirements it holds and a branch that edits
//! one issue's requirements changes one ledger file. Requirement prose remains owned by
//! `docs/requirements/`; this projection records only machine-checkable status
//! and attribution. A verdict is `implemented` only when the owning row names
//! a test file that exists. Unknown prose is kept honest as `partial`.
//!
//! Usage:
//!   rust-script scripts/generate-requirement-status.rs
//!   rust-script scripts/generate-requirement-status.rs --write
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The assembled register, split into parts under the 1500-line cap.
const REQUIREMENT_PARTS: &str = "docs/requirements/assembled";
const REQUIREMENT_SHARDS: &str = "docs/requirements";
const TRACEABILITY: &str = "docs/requirements-traceability.md";
const MANIFEST: &str = "data/meta/requirement-status-ledger.lino";
const LEDGER_DIRECTORY: &str = "data/meta/requirement-status-ledger";

#[derive(Clone, Debug, Default)]
struct TraceRow {
    delivered: String,
    automated_test: String,
    manual: String,
}

#[derive(Clone, Debug)]
struct Requirement {
    id: String,
    shard: String,
    verdict: String,
    delivered: String,
    issue: String,
    automated_test: String,
    manual: String,
}

fn requirement_ids(text: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('R') {
        let tail = &rest[index..];
        let id: String = tail
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
            .collect();
        let begins_with_number = id
            .strip_prefix('R')
            .is_some_and(|suffix| suffix.starts_with(|character: char| character.is_ascii_digit()));
        if begins_with_number && !ids.contains(&id) {
            ids.push(id);
        }
        rest = &tail[1..];
    }
    ids
}

/// A line defines the requirement it leads with: a table row (`| R12 |`), a
/// heading (`### R12`) or a list item (`- R12`).
fn is_definition_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("| R") || line.starts_with("### R") || line.starts_with("- R")
}

/// The requirements the register defines. Every other id it names is a
/// reference: a range endpoint (`R97-R100`, `R649-01 … R649-14`), a case-study
/// sub-requirement, or a cross-reference such as "R379-clean".
fn defined_requirement_ids(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter(|line| is_definition_line(line))
        .filter_map(|line| requirement_ids(line).into_iter().next())
        .collect()
}

fn quoted(value: &str) -> String {
    // Canonical Links Notation reads a doubled delimiter as an escape and an
    // even run of quotes as a possible longer opener, so a value carrying a
    // double quote is wrapped in single quotes (the canonical writer's own
    // preference); apostrophes then take the \x27 form this repository's
    // readers decode. Backslashes double in both branches because the
    // line-based reader unescapes them.
    if value.contains('"') {
        format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\x27"))
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\"))
    }
}

fn test_path(text: &str, root: &Path) -> String {
    let mut rust_test = None;
    for token in text.split(|character: char| {
        character.is_whitespace()
            || matches!(character, '`' | '|' | ',' | ';' | '(' | ')' | '[' | ']')
    }) {
        let candidate = token.trim_matches(|character| matches!(character, '.' | ':' | '"'));
        // JavaScript is the first root (R997): a browser-worker test under
        // `rust/tests/web/` pins a requirement as well as a Rust test does.
        // The Playwright suites (`rust/tests/e2e/tests/*.spec.js`) and the
        // desktop library tests (`desktop/scripts/*.test.mjs`, the
        // check-desktop-library gate) run in CI as well.
        let Some(candidate) = [".test.mjs", ".spec.js", ".rs"].iter().find_map(|suffix| {
            candidate
                .find(suffix)
                .map(|end| &candidate[..end + suffix.len()])
        }) else {
            continue;
        };
        // The workspace move (plan 16 L1) put the Rust tests under `rust/`;
        // shards name the new location, older trace rows the old one.
        if (candidate.starts_with("tests/")
            || candidate.starts_with("rust/tests/")
            || candidate.starts_with("desktop/scripts/"))
            && root.join(candidate).is_file()
        {
            // JavaScript first (R1188-U29): a row that names tests in both
            // roots cites the JavaScript one.
            if !candidate.ends_with(".rs") {
                return candidate.to_owned();
            }
            rust_test = rust_test.or(Some(candidate));
        }
    }
    rust_test.unwrap_or_default().to_owned()
}

fn trace_rows(text: &str, root: &Path) -> BTreeMap<String, TraceRow> {
    let mut rows = BTreeMap::new();
    for line in text.lines().filter(|line| line.starts_with("| R")) {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 5 {
            continue;
        }
        let Some(id) = requirement_ids(cells[0]).into_iter().next() else {
            continue;
        };
        rows.insert(
            id,
            TraceRow {
                delivered: cells[2].to_owned(),
                automated_test: test_path(cells[3], root),
                manual: cells[4].to_owned(),
            },
        );
    }
    rows
}

fn issue_from_shard(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.strip_prefix("issue-")
        .map(|rest| rest.chars().take_while(char::is_ascii_digit).collect())
        .unwrap_or_default()
}

/// `needle` occurs in `text` as words, not inside an identifier such as the
/// `web_search:provider_planned` event a row may name, or a compound such as
/// "the pending-task state" a delivered row may describe.
fn contains_word(text: &str, needle: &str) -> bool {
    let is_word = |character: char| character.is_alphanumeric() || matches!(character, '_' | '-');
    text.match_indices(needle).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + needle.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

fn verdict(line: &str, automated_test: &str) -> String {
    // A table row states its status after the id and requirement cells; the
    // requirement text may use the same words ("narration of the planned search").
    let status = if line.trim_start().starts_with('|') {
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
        if cells.len() > 2 {
            cells[2..].join("|")
        } else {
            line.to_owned()
        }
    } else {
        line.to_owned()
    };
    let lower = status.to_ascii_lowercase();
    // A status that opens with its verdict states it (R1188-U27): "Partial: …
    // delivered …" is partial and "Implemented: … planned …" implemented,
    // whatever words follow.
    let opening = lower.trim_start().trim_start_matches('*');
    if opening.starts_with("partial") {
        return "partial".to_owned();
    }
    if opening.starts_with("not delivered") {
        return "not-delivered".to_owned();
    }
    let stated = ["implemented", "delivered"].iter().any(|word| {
        opening
            .strip_prefix(word)
            .is_some_and(|rest| rest.trim_start().starts_with([':', '.', '(']))
    });
    if stated && !automated_test.is_empty() {
        return "implemented".to_owned();
    }
    // Words, not substrings: R1017-7 names its pinning test
    // `superseded_read_only_work_releases_its_runners`, and a substring match
    // filed a delivered row as superseded.
    if contains_word(&lower, "withdrawn") {
        return "withdrawn".to_owned();
    }
    if contains_word(&lower, "superseded") {
        return "superseded".to_owned();
    }
    if ["not delivered", "not implemented", "pending", "planned"]
        .iter()
        .any(|needle| contains_word(&lower, needle))
    {
        return "not-delivered".to_owned();
    }
    if !automated_test.is_empty()
        && ["implemented", "delivered", "complete", "covered by"]
            .iter()
            .any(|needle| lower.contains(needle))
    {
        return "implemented".to_owned();
    }
    "partial".to_owned()
}

/// The assembled requirement register: `REQUIREMENTS.md` is an index, and the
/// register itself is `docs/requirements/assembled/<area>.md`, read in name
/// order (`scripts/assemble-requirements.rs` writes both).
fn read_register(root: &Path) -> Result<String, String> {
    let directory = root.join(REQUIREMENT_PARTS);
    let mut parts: Vec<PathBuf> = fs::read_dir(&directory)
        .map_err(|error| format!("cannot read {REQUIREMENT_PARTS}: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".md"))
        })
        .collect();
    parts.sort();
    if parts.is_empty() {
        return Err(format!("{REQUIREMENT_PARTS} holds no assembled area files"));
    }
    let mut register = String::new();
    for part in parts {
        register.push_str(
            &fs::read_to_string(&part)
                .map_err(|error| format!("cannot read {}: {error}", part.display()))?,
        );
        register.push('\n');
    }
    Ok(register)
}

fn requirement_rows(root: &Path) -> Result<Vec<Requirement>, String> {
    let assembled = read_register(root)?;
    let expected = defined_requirement_ids(&assembled);
    let trace = trace_rows(
        &fs::read_to_string(root.join(TRACEABILITY)).unwrap_or_default(),
        root,
    );
    let mut ownership: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut paths: Vec<PathBuf> = fs::read_dir(root.join(REQUIREMENT_SHARDS))
        .map_err(|error| format!("cannot read {REQUIREMENT_SHARDS}: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect();
    paths.sort();
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        for line in source.lines() {
            let ids = requirement_ids(line);
            // A definition row defines only the id it leads with; the other ids
            // it names ("R1172-1's word-boundary fix") are references.
            let defined = ids.first().cloned();
            for id in ids {
                if expected.contains(&id) {
                    let is_definition = is_definition_line(line) && defined.as_ref() == Some(&id);
                    match ownership.get(&id) {
                        None => {
                            ownership.insert(id, (relative.clone(), line.to_owned()));
                        }
                        Some(_) if is_definition => {
                            ownership.insert(id, (relative.clone(), line.to_owned()));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    let missing: Vec<&String> = expected
        .iter()
        .filter(|id| !ownership.contains_key(*id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} assembled requirement ids have no owning shard: {missing:?}",
            missing.len()
        ));
    }

    let mut rows = Vec::with_capacity(expected.len());
    for id in expected {
        let (shard, line) = ownership.remove(&id).expect("checked above");
        let traced = trace.get(&id).cloned().unwrap_or_default();
        let source_test = test_path(&line, root);
        let automated_test = if source_test.is_empty() {
            traced.automated_test
        } else {
            source_test
        };
        rows.push(Requirement {
            verdict: verdict(&line, &automated_test),
            delivered: traced.delivered,
            issue: issue_from_shard(Path::new(&shard)),
            manual: if traced.manual.is_empty() {
                "not yet confirmed".to_owned()
            } else {
                traced.manual
            },
            id,
            shard,
            automated_test,
        });
    }
    Ok(rows)
}

fn render_manifest(rows: &[Requirement], ledger_files: &[String]) -> String {
    let implemented = rows
        .iter()
        .filter(|row| row.verdict == "implemented")
        .count();
    let mut output = String::from(
        "# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n\
         requirement_status_ledger\n",
    );
    output.push_str("  source \"REQUIREMENTS.md and docs/requirements/*.md\"\n");
    output.push_str(
        "  verdict_vocabulary \"implemented | partial | not-delivered | superseded | withdrawn\"\n",
    );
    // Issue #1090 (E112), the retire branch: the manual-confirmation column
    // is aspirational. `not yet confirmed` is the honest resting state of a
    // row whose machinery is tested but nobody has yet watched run by hand;
    // it carries no debt and gates nothing. The vocabulary line records the
    // decision where every consumer of this manifest reads it.
    output.push_str(
        "  manual_column \"aspirational since 2026-09-30 (issue #1090): not yet confirmed rows carry no debt\"\n",
    );
    output.push_str(&format!("  requirement_count {}\n", rows.len()));
    output.push_str(&format!("  implemented_count {implemented}\n"));
    for name in ledger_files {
        output.push_str(&format!("  shard \"{LEDGER_DIRECTORY}/{name}.lino\"\n"));
    }
    output
}

/// The value more than half of `values` share, or the empty text when none
/// does.
fn shared_value(values: &[&str]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .find(|(_, count)| count * 2 > values.len())
        .map_or_else(String::new, |(value, _)| value.to_owned())
}

/// The fields a ledger file may state once for its records, in record order.
const SHARED_FIELDS: [&str; 3] = ["delivered", "automated_test", "manual"];

/// Choose a smaller source default, preserving the original majority rule.
fn shared_default(values: &[&str], field: &str) -> String {
    assert!(SHARED_FIELDS.contains(&field), "unowned shared field");
    let majority = shared_value(values);
    if !majority.is_empty() {
        return majority;
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    let empty_count = counts.get("").copied().unwrap_or_default();
    let bytes = |value: &str| format!("    {field} {}\n", quoted(value)).len();
    let mut selected = String::new();
    let mut saved = 0;
    for value in values {
        let count = counts[value];
        if value.is_empty() || count < 2 || count - 1 <= empty_count {
            continue;
        }
        let removed = (count - 1) * bytes(value) + 2;
        let added = empty_count * bytes("");
        if removed > added && removed - added > saved {
            selected = (*value).to_owned();
            saved = removed - added;
        }
    }
    selected
}

/// The value of one of the `SHARED_FIELDS` of a record.
fn shared_field<'a>(row: &'a Requirement, name: &str) -> &'a str {
    match name {
        "delivered" => &row.delivered,
        "automated_test" => &row.automated_test,
        _ => &row.manual,
    }
}

/// One ledger file. Shared structure is stated once (R1188-U7,
/// docs/links-notation-style.md): the shard and its issue are the same for
/// every record of the file, so the file states them, and each of the
/// `SHARED_FIELDS` whose value more than half of the records share (such as
/// `manual "not yet confirmed"`) is stated on the file as the default. A
/// record states such a field only where it differs from the file's value
/// (the empty text when the file states none), so a reader that applies the
/// defaults reads the same values as before. `id` and `verdict` stay on every
/// record.
fn render_shard(rows: &[&Requirement]) -> String {
    let shared: Vec<String> = SHARED_FIELDS
        .iter()
        .map(|name| {
            shared_default(
                &rows
                    .iter()
                    .map(|row| shared_field(row, name))
                    .collect::<Vec<_>>(),
                name,
            )
        })
        .collect();
    // The file name and the `shard` line already say whose requirements these
    // are, so the root line does not repeat the shard's file name.
    let mut output = String::from(
        "# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n\
         requirement_status_ledger_shard\n",
    );
    if let Some(row) = rows.first() {
        output.push_str(&format!("  shard {}\n", quoted(&row.shard)));
        output.push_str(&format!("  issue {}\n", quoted(&row.issue)));
    }
    for (name, value) in SHARED_FIELDS.iter().zip(&shared) {
        if !value.is_empty() {
            output.push_str(&format!("  {name} {}\n", quoted(value)));
        }
    }
    for row in rows {
        output.push_str("  requirement\n");
        output.push_str(&format!("    id {}\n", quoted(&row.id)));
        output.push_str(&format!("    verdict {}\n", quoted(&row.verdict)));
        for (name, value) in SHARED_FIELDS.iter().zip(&shared) {
            let own = shared_field(row, name);
            if own != value.as_str() {
                output.push_str(&format!("    {name} {}\n", quoted(own)));
            }
        }
    }
    output
}

/// The ledger file a requirement's record lives in: its shard's file stem.
fn ledger_name(shard: &str) -> String {
    Path::new(shard)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(shard)
        .to_owned()
}

fn generated(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let rows = requirement_rows(root)?;
    let mut by_shard: BTreeMap<String, Vec<&Requirement>> = BTreeMap::new();
    for row in &rows {
        by_shard
            .entry(ledger_name(&row.shard))
            .or_default()
            .push(row);
    }
    let names: Vec<String> = by_shard.keys().cloned().collect();
    let mut files = BTreeMap::new();
    files.insert(root.join(MANIFEST), render_manifest(&rows, &names));
    for (name, shard_rows) in &by_shard {
        files.insert(
            root.join(format!("{LEDGER_DIRECTORY}/{name}.lino")),
            render_shard(shard_rows),
        );
    }
    Ok(files)
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let write = std::env::args().nth(1).as_deref() == Some("--write");
    let files = generated(&root).unwrap_or_else(|error| {
        eprintln!("generate-requirement-status: {error}");
        std::process::exit(1);
    });
    let expected_paths: BTreeSet<PathBuf> = files.keys().cloned().collect();

    if write {
        fs::create_dir_all(root.join(LEDGER_DIRECTORY)).expect("create ledger directory");
        for entry in fs::read_dir(root.join(LEDGER_DIRECTORY))
            .expect("read ledger directory")
            .filter_map(Result::ok)
        {
            if entry.path().extension().and_then(|value| value.to_str()) == Some("lino")
                && !expected_paths.contains(&entry.path())
            {
                fs::remove_file(entry.path()).expect("remove obsolete generated shard");
            }
        }
        for (path, content) in &files {
            fs::write(path, content)
                .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        }
        println!("generated {} requirement-status files", files.len());
        return;
    }

    let stale: Vec<String> = files
        .iter()
        .filter(|(path, content)| fs::read_to_string(path).ok().as_ref() != Some(content))
        .map(|(path, _)| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    if stale.is_empty() {
        println!(
            "requirement-status ledger is current ({} files)",
            files.len()
        );
    } else {
        eprintln!("stale generated requirement-status files: {stale:?}");
        eprintln!("run rust-script scripts/generate-requirement-status.rs --write");
        std::process::exit(1);
    }
}
