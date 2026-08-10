#!/usr/bin/env rust-script
//! Audits problem-solving metadata on seed meaning records (issue #918).
//!
//! Missing fields are themselves reviewed data, sharded below `data/meta/`.
//! Coding-path concepts are the regression floor and may not have gaps.
//!
//! Usage:
//!   rust-script scripts/audit-seed-metadata.rs
//!   rust-script scripts/audit-seed-metadata.rs --write
//!   rust-script --test scripts/audit-seed-metadata.rs
//!
//! ```cargo
//! [dependencies]
//! walkdir = "2"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

const SCHEMA_PATH: &str = "data/meta/seed-metadata-schema.lino";
const SEED_ROOT: &str = "data/seed";
const GAP_PREFIX: &str = "data/meta/seed-metadata-gaps-";
const SHARD_COUNT: usize = 16;

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

fn parse_meanings(source: &str, text: &str) -> Result<Vec<MeaningRecord>, String> {
    if text.lines().find(|line| !line.trim().is_empty()) != Some("meanings") {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut current: Option<MeaningRecord> = None;
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
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
                if let Some(field) = line.split_whitespace().next() {
                    record.fields.insert(field.to_owned());
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

fn collect_records(root: &Path) -> Result<Vec<MeaningRecord>, String> {
    let mut records = Vec::new();
    for entry in WalkDir::new(root.join(SEED_ROOT)) {
        let entry = entry.map_err(|error| format!("walk {SEED_ROOT}: {error}"))?;
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().is_none_or(|value| value != "lino") {
            continue;
        }
        let source = path
            .strip_prefix(root)
            .map_err(|error| format!("relative seed path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(path).map_err(|error| format!("read {source}: {error}"))?;
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

fn stable_shard(gap: &Gap) -> usize {
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
    hash as usize % SHARD_COUNT
}

fn render_shards(gaps: &[Gap]) -> BTreeMap<String, String> {
    let mut sharded = vec![Vec::new(); SHARD_COUNT];
    for gap in gaps {
        sharded[stable_shard(gap)].push(gap);
    }

    sharded
        .into_iter()
        .enumerate()
        .map(|(index, gaps)| {
            let mut output = format!(
                "seed_metadata_gaps\n  issue 918\n  shard {index}\n  shard_count {SHARD_COUNT}\n  audit_scope \"problem-solving concept records under data/seed meanings roots\"\n"
            );
            for gap in gaps {
                output.push_str(&format!(
                    "  gap {}\n    source \"{}\"\n    missing \"{}\"\n",
                    gap.record,
                    gap.source,
                    gap.missing.join(",")
                ));
            }
            (
                format!("{GAP_PREFIX}{index:02}.lino"),
                output,
            )
        })
        .collect()
}

fn check_or_write(
    root: &Path,
    expected: &BTreeMap<String, String>,
    write: bool,
) -> Result<(), String> {
    if write {
        for (path, content) in expected {
            fs::write(root.join(path), content)
                .map_err(|error| format!("write {path}: {error}"))?;
        }
    }

    let mut errors = Vec::new();
    for (path, expected_content) in expected {
        match fs::read_to_string(root.join(path)) {
            Ok(actual) if actual == *expected_content => {}
            Ok(_) => errors.push(format!("stale {path}; run this script with --write")),
            Err(error) => errors.push(format!("missing {path}: {error}; run with --write")),
        }
    }

    let expected_paths = expected.keys().map(String::as_str).collect::<BTreeSet<_>>();
    for entry in WalkDir::new(root.join("data/meta")).max_depth(1) {
        let entry = entry.map_err(|error| format!("walk data/meta: {error}"))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| format!("relative gap path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        if relative.starts_with(GAP_PREFIX)
            && relative.ends_with(".lino")
            && !expected_paths.contains(relative.as_str())
        {
            errors.push(format!("unexpected stale gap shard {relative}"));
        }
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
        let shards = render_shards(&gaps);
        check_or_write(&root, &shards, write)?;
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
        assert!(render_shards(&gaps)
            .values()
            .any(|shard| shard.contains("gap domain_term")));
    }
}
