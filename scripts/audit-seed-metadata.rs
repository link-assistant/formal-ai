#!/usr/bin/env rust-script
//! Audits problem-solving metadata on seed meaning records (issue #918).
//!
//! Missing fields are themselves reviewed data: one file per seed source,
//! `data/meta/seed-metadata-gaps/<seed file>` (R1188-U5), so a file says whose
//! gaps it holds and a branch that edits one seed file changes one gap file.
//! A gap file states its source once, and the missing fields most of its gaps
//! share once as the file default (R1188-U7); a gap states only what differs.
//! Coding-path concepts are the regression floor and may not have gaps.
//!
//! Usage:
//!   rust-script scripts/audit-seed-metadata.rs
//!   rust-script scripts/audit-seed-metadata.rs --write
//!   rust-script --test scripts/audit-seed-metadata.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

#![cfg_attr(test, allow(dead_code, unused_imports))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;

const SCHEMA_PATH: &str = "data/meta/seed-metadata-schema.lino";
const SEED_ROOT: &str = "data/seed";
const GAP_DIRECTORY: &str = "data/meta/seed-metadata-gaps";
const AUDIT_SCOPE: &str = "problem-solving concept records under data/seed meanings roots";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MeaningRecord {
    source: String,
    name: String,
    fields: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Gap {
    source: String,
    record: String,
    missing: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct Schema {
    required_fields: Vec<String>,
    complete_sources: BTreeSet<String>,
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn parse_schema(text: &str) -> Result<Schema, String> {
    let required_fields = text
        .lines()
        .filter_map(|line| line.strip_prefix("  required_field "))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let complete_sources = text
        .lines()
        .filter_map(|line| line.strip_prefix("  complete_source "))
        .map(unquote)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let unique_fields = required_fields.iter().collect::<BTreeSet<_>>();
    if required_fields.is_empty() || unique_fields.len() != required_fields.len() {
        return Err("schema required_field rows must be nonempty and unique".to_owned());
    }
    if complete_sources.is_empty() {
        return Err("schema must name at least one complete_source".to_owned());
    }
    Ok(Schema {
        required_fields,
        complete_sources,
    })
}

fn indentation(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}

/// A `#` line, whatever its indentation.
fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

fn parse_meanings(source: &str, text: &str) -> Result<Vec<MeaningRecord>, String> {
    // The root is the first line that is neither blank nor a comment, so a
    // file may open with a comment header.
    let Some(root) = text
        .lines()
        .position(|line| !line.trim().is_empty() && !is_comment(line))
    else {
        return Ok(Vec::new());
    };
    if text.lines().nth(root) != Some("meanings") {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut current: Option<MeaningRecord> = None;
    for (index, line) in text.lines().enumerate().skip(root + 1) {
        if line.trim().is_empty() || is_comment(line) {
            continue;
        }
        match indentation(line) {
            2 => {
                if let Some(record) = current.take() {
                    records.push(record);
                }
                let name = line
                    .split_whitespace()
                    .next()
                    .ok_or_else(|| format!("{source}:{}: empty meaning record", index + 1))?;
                current = Some(MeaningRecord {
                    source: source.to_owned(),
                    name: name.to_owned(),
                    fields: BTreeSet::new(),
                });
            }
            4 => {
                let record = current.as_mut().ok_or_else(|| {
                    format!(
                        "{source}:{}: direct field before a meaning record",
                        index + 1
                    )
                })?;
                let trimmed = line.trim();
                if let Some(value_offset) = trimmed.find(char::is_whitespace) {
                    let field = &trimmed[..value_offset];
                    let raw_value = trimmed[value_offset..].trim();
                    if !unquote(raw_value).trim().is_empty() {
                        record.fields.insert(field.to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(record) = current {
        records.push(record);
    }

    let mut names = BTreeSet::new();
    for record in &records {
        if !names.insert(record.name.as_str()) {
            return Err(format!(
                "{source}: duplicate top-level meaning record {}",
                record.name
            ));
        }
    }
    Ok(records)
}

/// Every `.lino` file below `directory`, in no particular order.
fn lino_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("walk {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("walk {}: {error}", directory.display()))?
            .path();
        if path.is_dir() {
            lino_files(&path, files)?;
        } else if path.extension().is_some_and(|value| value == "lino") {
            files.push(path);
        }
    }
    Ok(())
}

fn collect_records(root: &Path) -> Result<Vec<MeaningRecord>, String> {
    let mut records = Vec::new();
    let mut paths = Vec::new();
    lino_files(&root.join(SEED_ROOT), &mut paths)?;
    for path in paths {
        let source = path
            .strip_prefix(root)
            .map_err(|error| format!("relative seed path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&path).map_err(|error| format!("read {source}: {error}"))?;
        records.extend(parse_meanings(&source, &text)?);
    }
    records.sort();
    Ok(records)
}

fn find_gaps(records: &[MeaningRecord], schema: &Schema) -> Result<Vec<Gap>, String> {
    let mut sources_seen = BTreeSet::new();
    let mut gaps = Vec::new();
    for record in records {
        sources_seen.insert(record.source.as_str());
        let missing = schema
            .required_fields
            .iter()
            .filter(|field| !record.fields.contains(field.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            gaps.push(Gap {
                source: record.source.clone(),
                record: record.name.clone(),
                missing,
            });
        }
    }

    for source in &schema.complete_sources {
        if !sources_seen.contains(source.as_str()) {
            return Err(format!("complete_source {source} has no meaning records"));
        }
        let source_gaps = gaps
            .iter()
            .filter(|gap| &gap.source == source)
            .collect::<Vec<_>>();
        if !source_gaps.is_empty() {
            let details = source_gaps
                .iter()
                .map(|gap| format!("{} [{}]", gap.record, gap.missing.join(",")))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "coding-path complete_source {source} has metadata gaps: {details}"
            ));
        }
    }
    Ok(gaps)
}

fn stable_hash(gap: &Gap) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in gap
        .source
        .bytes()
        .chain(std::iter::once(b'#'))
        .chain(gap.record.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The value more than half of `values` share, or the empty text when none
/// does.
fn shared_value(values: &[String]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value.as_str()).or_default() += 1;
    }
    counts
        .into_iter()
        .find(|(_, count)| count * 2 > values.len())
        .map_or_else(String::new, |(value, _)| value.to_owned())
}

/// The gap file of a seed source: the source's path below `data/seed`, below
/// the gap directory.
fn gap_file(source: &str) -> String {
    let relative = source
        .strip_prefix(SEED_ROOT)
        .map_or(source, |rest| rest.trim_start_matches('/'));
    format!("{GAP_DIRECTORY}/{relative}")
}

fn render_gap_files(gaps: &[Gap]) -> BTreeMap<String, String> {
    let mut by_source: BTreeMap<&str, Vec<&Gap>> = BTreeMap::new();
    for gap in gaps {
        by_source.entry(&gap.source).or_default().push(gap);
    }
    let mut ids = BTreeSet::new();
    by_source
        .into_iter()
        .map(|(source, gaps)| {
            let missing = gaps
                .iter()
                .map(|gap| gap.missing.join(","))
                .collect::<Vec<_>>();
            let shared = shared_value(&missing);
            let mut output = format!(
                "seed-metadata-gaps\n  issue 918\n  source {}\n  audit-scope {}\n",
                quoted(source),
                quoted(AUDIT_SCOPE)
            );
            if !shared.is_empty() {
                output.push_str(&format!("  missing {}\n", quoted(&shared)));
            }
            for (gap, missing) in gaps.iter().zip(&missing) {
                let id = format!("seed-metadata-gap-{:016x}", stable_hash(gap));
                assert!(ids.insert(id.clone()), "stable gap id collision: {id}");
                output.push_str(&format!("  gap {id}\n    record {}\n", quoted(&gap.record)));
                if *missing != shared {
                    output.push_str(&format!("    missing {}\n", quoted(missing)));
                }
            }
            (gap_file(source), output)
        })
        .collect()
}

fn check_or_write(
    root: &Path,
    expected: &BTreeMap<String, String>,
    write: bool,
) -> Result<(), String> {
    let mut present = Vec::new();
    if root.join(GAP_DIRECTORY).is_dir() {
        lino_files(&root.join(GAP_DIRECTORY), &mut present)?;
    }
    let present = present
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
                .map_err(|error| format!("relative gap path: {error}"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if write {
        for path in present.iter().filter(|path| !expected.contains_key(*path)) {
            fs::remove_file(root.join(path)).map_err(|error| format!("remove {path}: {error}"))?;
        }
        for (path, content) in expected {
            if let Some(parent) = root.join(path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("create {}: {error}", parent.display()))?;
            }
            fs::write(root.join(path), content)
                .map_err(|error| format!("write {path}: {error}"))?;
        }
        return Ok(());
    }

    let mut errors = Vec::new();
    for (path, expected_content) in expected {
        match fs::read_to_string(root.join(path)) {
            Ok(actual) if actual == *expected_content => {}
            Ok(_) => errors.push(format!("stale {path}; run this script with --write")),
            Err(error) => errors.push(format!("missing {path}: {error}; run with --write")),
        }
    }
    for path in present.iter().filter(|path| !expected.contains_key(*path)) {
        errors.push(format!(
            "unexpected stale gap file {path}; run with --write"
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn repository_root() -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|error| format!("current directory: {error}"))?;
    if current.join(SCHEMA_PATH).is_file() {
        Ok(current)
    } else {
        Err(format!(
            "run from the repository root; {SCHEMA_PATH} was not found"
        ))
    }
}

#[cfg(not(test))]
fn main() {
    let write = std::env::args().any(|argument| argument == "--write");
    let result = (|| -> Result<(usize, usize), String> {
        let root = repository_root()?;
        let schema_text = fs::read_to_string(root.join(SCHEMA_PATH))
            .map_err(|error| format!("read {SCHEMA_PATH}: {error}"))?;
        let schema = parse_schema(&schema_text)?;
        let records = collect_records(&root)?;
        let gaps = find_gaps(&records, &schema)?;
        let files = render_gap_files(&gaps);
        check_or_write(&root, &files, write)?;
        Ok((records.len(), gaps.len()))
    })();

    match result {
        Ok((records, gaps)) => println!(
            "seed metadata: audited {records} concepts; {gaps} per-record gaps captured in data"
        ),
        Err(error) => {
            eprintln!("seed metadata audit failed:\n{error}");
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Schema {
        Schema {
            required_fields: vec![
                "role".to_owned(),
                "precondition".to_owned(),
                "effect".to_owned(),
                "unit".to_owned(),
                "example".to_owned(),
            ],
            complete_sources: BTreeSet::from(["data/seed/coding.lino".to_owned()]),
        }
    }

    #[test]
    fn parses_only_direct_metadata_fields() {
        let records = parse_meanings(
            "data/seed/coding.lino",
            "meanings\n  coding_loop\n    role coding_control\n    precondition \"request present\"\n    effect \"loop selected\"\n    unit \"not applicable\"\n    example \"repeat three times\"\n    lexeme en\n      role nested_role_is_not_direct\n",
        )
        .expect("meaning records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields.len(), 6);
        assert!(records[0].fields.contains("role"));
    }

    #[test]
    fn a_comment_header_comes_before_the_meanings_root() {
        let records = parse_meanings(
            "data/seed/coding.lino",
            "# The coding loop.\n# Moved out of coding.lino.\nmeanings\n  coding_loop\n    role coding_control\n",
        )
        .expect("meaning records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "coding_loop");
    }

    #[test]
    fn complete_coding_sources_reject_any_gap() {
        let records = parse_meanings(
            "data/seed/coding.lino",
            "meanings\n  coding_loop\n    role coding_control\n",
        )
        .expect("meaning records");
        let error = find_gaps(&records, &schema()).expect_err("coding gap must fail");
        assert!(error.contains("precondition,effect,unit,example"));
    }

    #[test]
    fn complete_coding_sources_reject_empty_values() {
        let records = parse_meanings(
            "data/seed/coding.lino",
            "meanings\n  coding_loop\n    role \"\"\n    precondition ready\n    effect selected\n    unit \"not applicable\"\n    example loop\n",
        )
        .expect("meaning records");
        let error = find_gaps(&records, &schema()).expect_err("empty metadata must fail");
        assert!(error.contains("role"));
    }

    #[test]
    fn noncoding_gaps_are_stable_data() {
        let complete = parse_meanings(
            "data/seed/coding.lino",
            "meanings\n  coding_loop\n    role coding_control\n    precondition ready\n    effect selected\n    unit \"not applicable\"\n    example loop\n",
        )
        .expect("complete record");
        let incomplete = parse_meanings(
            "data/seed/domain.lino",
            "meanings\n  domain_term\n    role domain_role\n",
        )
        .expect("incomplete record");
        let gaps = find_gaps(&[complete[0].clone(), incomplete[0].clone()], &schema())
            .expect("noncoding gaps are data");
        assert_eq!(gaps.len(), 1);
        assert_eq!(
            gaps[0].missing,
            ["precondition", "effect", "unit", "example"]
        );
        assert!(
            render_gap_files(&gaps)
                .values()
                .any(|file| file.contains("record \"domain_term\""))
        );
    }

    #[test]
    fn renders_punctuation_in_record_names_as_quoted_data() {
        let gap = Gap {
            source: "data/seed/domain.lino".to_owned(),
            record: "lexical-sense:".to_owned(),
            missing: vec!["effect".to_owned()],
        };
        let rendered = render_gap_files(&[gap]);
        let file = rendered
            .values()
            .find(|content| content.contains("lexical-sense:"))
            .expect("gap file");
        assert!(file.contains("gap seed-metadata-gap-"));
        assert!(file.contains("record \"lexical-sense:\""));
    }

    #[test]
    fn a_gap_file_states_its_source_and_shared_missing_fields_once() {
        let gap = |record: &str, missing: &[&str]| Gap {
            source: "data/seed/domain.lino".to_owned(),
            record: record.to_owned(),
            missing: missing.iter().map(|field| (*field).to_owned()).collect(),
        };
        let rendered = render_gap_files(&[
            gap("alpha", &["effect", "unit"]),
            gap("beta", &["effect", "unit"]),
            gap("gamma", &["example"]),
        ]);
        let file = &rendered["data/meta/seed-metadata-gaps/domain.lino"];
        assert!(
            file.starts_with(
                "seed-metadata-gaps\n  issue 918\n  source \"data/seed/domain.lino\"\n"
            )
        );
        assert_eq!(file.matches("source ").count(), 1);
        assert!(file.contains("  missing \"effect,unit\"\n"));
        assert!(file.contains("    record \"alpha\"\n  gap "));
        assert!(file.contains("    record \"gamma\"\n    missing \"example\"\n"));
    }
}
