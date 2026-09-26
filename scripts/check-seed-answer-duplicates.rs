#!/usr/bin/env rust-script
//! Fail when one byte-identical `answer` value appears more than once in
//! `data/seed/**/*.lino` (issue #1138 B9, plan 09 leaf 37).
//!
//! A repeated answer is a memo, not a fact: `data/seed/greetings.lino` and
//! `data/seed/identity.lino` used to restate the reply text on every trigger
//! row, so fixing one wording meant hunting its copies across rows and
//! languages. The trigger rows now point at the canonical reply through
//! `response_link`, and this lint keeps the duplication from coming back —
//! any byte-identical restatement of an answer value anywhere in the seed
//! fails the gate, whatever the file.
//!
//! Usage:
//!   rust-script scripts/check-seed-answer-duplicates.rs          # check (CI/local)
//!   rust-script --test scripts/check-seed-answer-duplicates.rs   # inline unit tests
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! walkdir = "2"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

/// The seed root scanned for duplicated answer values.
const SEED_ROOT: &str = "data/seed";

/// One occurrence of an answer value: where the repetition lives.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Occurrence {
    /// Repository-relative path, `/`-separated.
    file: String,
    /// One-based line number.
    line: usize,
}

/// A value stated more than once, with every place that states it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Duplicate {
    value: String,
    occurrences: Vec<Occurrence>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct CheckResult {
    duplicates: Vec<Duplicate>,
    /// Files skipped because they are not valid UTF-8; reported so silence
    /// never hides an unscanned file.
    unreadable: Vec<String>,
}

impl CheckResult {
    const fn is_clean(&self) -> bool {
        self.duplicates.is_empty() && self.unreadable.is_empty()
    }
}

fn normalized_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// The verbatim value an `  answer <value>` line states, or `None` for any
/// other line. The remainder after `answer ` is kept byte for byte — quoting
/// style, escapes and trailing spaces are part of the value's identity, since
/// the leaf's rule is byte-identical repetition.
fn answer_value(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("answer ")?;
    Some(rest.trim_end())
}

/// Scan every `.lino` file under `data/seed` and collect the answer values
/// that appear more than once.
fn check_seed(root: &Path) -> CheckResult {
    let mut seen: BTreeMap<String, Vec<Occurrence>> = BTreeMap::new();
    let mut result = CheckResult::default();

    let seed_dir = root.join(SEED_ROOT);
    for entry in WalkDir::new(&seed_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "lino"))
    {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(path) else {
            result.unreadable.push(normalized_path(path, root));
            continue;
        };
        let file = normalized_path(path, root);
        for (index, line) in content.lines().enumerate() {
            if let Some(value) = answer_value(line) {
                seen.entry(value.to_string()).or_default().push(Occurrence {
                    file: file.clone(),
                    line: index + 1,
                });
            }
        }
    }

    result.duplicates = seen
        .into_iter()
        .filter(|(_, occurrences)| occurrences.len() > 1)
        .map(|(value, mut occurrences)| {
            // WalkDir yields files in directory order; a sorted occurrence list
            // keeps the report and the tests deterministic.
            occurrences.sort();
            Duplicate { value, occurrences }
        })
        .collect();
    result
}

#[cfg(not(test))]
fn report(result: &CheckResult) {
    if !result.duplicates.is_empty() {
        println!("\nByte-identical answer values repeated in the seed:\n");
        for duplicate in &result.duplicates {
            println!("  {}", duplicate.value);
            for occurrence in &duplicate.occurrences {
                println!(
                    "    {}:{} (line {})",
                    occurrence.file, occurrence.line, occurrence.line
                );
            }
        }
        println!(
            "\nAn answer is stated once and referenced by `response_link`; a repeated answer\n\
             is a memo that will drift. Point the extra rows at the canonical response.\n"
        );
    }
    for file in &result.unreadable {
        println!("NOTICE: {file} is not valid UTF-8 and was not scanned");
    }
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking data/seed for byte-identical repeated answer values...\n");

    let root = std::env::current_dir().expect("Failed to get current directory");
    let result = check_seed(&root);
    report(&result);

    if result.is_clean() {
        println!("Every answer value in the seed is stated exactly once\n");
        exit(0);
    }
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("seed-answer-dup-{name}-{nanos}"));
        fs::create_dir_all(path.join(SEED_ROOT)).unwrap();
        path
    }

    fn write_seed(repo: &Path, file: &str, body: &str) {
        fs::write(repo.join(SEED_ROOT).join(file), body).unwrap();
    }

    #[test]
    fn answer_value_reads_the_verbatim_remainder_and_nothing_else() {
        assert_eq!(
            answer_value("  answer \"Hi, how may I help you?\""),
            Some("\"Hi, how may I help you?\"")
        );
        assert_eq!(answer_value("  answer не_цитата"), Some("не_цитата"));
        assert_eq!(answer_value("  text Hi"), None);
        assert_eq!(answer_value("  response_link \"response:greeting\""), None);
    }

    #[test]
    fn a_value_repeated_across_rows_is_a_duplicate_naming_every_row() {
        let repo = temp_repo("repeat");
        write_seed(
            &repo,
            "greetings.lino",
            "greeting_1\n  text Hi\n  intent greeting\n  response_link \"response:greeting\"\n  answer \"Hi\"\ngreeting_2\n  text Hello\n  intent greeting\n  response_link \"response:greeting\"\n  answer \"Hi\"\n",
        );

        let result = check_seed(&repo);

        assert_eq!(result.duplicates.len(), 1);
        let duplicate = &result.duplicates[0];
        assert_eq!(duplicate.value, "\"Hi\"");
        assert_eq!(duplicate.occurrences.len(), 2);
        assert_eq!(duplicate.occurrences[0].line, 5);
        assert_eq!(duplicate.occurrences[1].line, 10);
        assert!(!result.is_clean());
    }

    #[test]
    fn distinct_values_and_single_statements_pass() {
        let repo = temp_repo("clean");
        write_seed(
            &repo,
            "identity.lino",
            "identity_1\n  text \"Who are you?\"\n  intent identity\n  response_link \"response:identity\"\n",
        );
        write_seed(
            &repo,
            "greetings.lino",
            "greeting_1\n  text Hi\n  intent greeting\n  response_link \"response:greeting\"\n  answer \"Hi, how may I help you?\"\n",
        );

        let result = check_seed(&repo);

        assert_eq!(result, CheckResult::default());
        assert!(result.is_clean());
    }

    /// Byte-identical is the bar: two wordings that differ by one character are
    /// two facts, and neither fails.
    #[test]
    fn values_that_differ_by_a_single_byte_are_not_duplicates() {
        let repo = temp_repo("byte-diff");
        write_seed(
            &repo,
            "greetings.lino",
            "greeting_1\n  answer \"Hi\"\ngreeting_2\n  answer \"Hi!\"\n",
        );

        let result = check_seed(&repo);

        assert_eq!(result.duplicates, Vec::new());
    }

    #[test]
    fn a_repetition_across_two_files_is_still_a_repetition() {
        let repo = temp_repo("cross-file");
        write_seed(
            &repo,
            "greetings.lino",
            "greeting_1\n  answer \"_shared\"\n",
        );
        write_seed(&repo, "identity.lino", "identity_1\n  answer \"_shared\"\n");

        let result = check_seed(&repo);

        assert_eq!(result.duplicates.len(), 1);
        let files: Vec<&str> = result.duplicates[0]
            .occurrences
            .iter()
            .map(|occurrence| occurrence.file.as_str())
            .collect();
        assert_eq!(
            files,
            vec!["data/seed/greetings.lino", "data/seed/identity.lino",]
        );
    }

    /// The shipped seed must itself pass the lint this leaf adds.
    #[test]
    fn the_committed_seed_has_no_repeated_answer_values() {
        // `rust-script --test` runs from its own build directory, so locate the
        // repository through this file's path rather than the process cwd.
        let repository_root = Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .expect("script lives in <repo>/scripts");

        let result = check_seed(repository_root);

        assert_eq!(
            result.duplicates,
            Vec::new(),
            "plan 09 leaf 37: every answer in data/seed must be stated once and referenced via response_link"
        );
    }
}
