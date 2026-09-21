//! Issue #1138 B8 (plan 08, L8, L17'): a verifiable expectation outranks a lexical route.
//!
//! `Find y: 7 * y = 84` is claimed by the terminal-command route today because
//! `Find` is a program name; an edit instruction with no quoted operand is
//! declined by the quoted-operand parser. Both must reach the verifiable route —
//! and no prompt any existing handler answers today may be taken away from it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::FormalAiEngine;

use super::{LANGUAGES, case};

const SUITE: &str = "data/benchmarks/conversational-variations-suite.lino";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn lino_records(text: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(std::mem::take(&mut current));
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn field(record: &[String], wanted: &str) -> Option<String> {
    record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find(|(name, _)| *name == wanted)
        .map(|(_, raw)| raw.trim().trim_matches('"').to_owned())
}

fn fields(record: &[String], wanted: &str) -> Vec<String> {
    record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .filter(|(name, _)| *name == wanted)
        .map(|(_, raw)| raw.trim().trim_matches('"').to_owned())
        .collect()
}

/// Every recorded conversational wording, with the intent it routes to today.
fn conversational_cases() -> BTreeMap<String, String> {
    let manifest = fs::read_to_string(repo_root().join(SUITE))
        .unwrap_or_else(|error| panic!("{SUITE} should be readable: {error}"));
    let manifest_records = lino_records(&manifest);
    let member_files: Vec<String> = manifest_records
        .iter()
        .flat_map(|record| fields(record, "member_file"))
        .collect();

    let mut cases = BTreeMap::new();
    for member in member_files {
        let path = repo_root().join("data/benchmarks").join(&member);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for record in lino_records(&text) {
            let Some(expected) = field(&record, "expected_intent") else {
                continue;
            };
            for prompt in fields(&record, "prompt") {
                cases.insert(prompt, expected.clone());
            }
        }
    }
    cases
}

/// A named unknown is a verifiable task, not a shell command.
#[test]
fn a_named_unknown_outranks_the_terminal_command_route() {
    for language in LANGUAGES {
        let paraphrase = case("named_unknown", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);
        assert_ne!(
            response.intent, "agent_suggestion",
            "{language}: `Find` is a program name, but a declared unknown outranks it: {}",
            response.answer
        );
        assert_eq!(
            response.intent, "verifiable_task",
            "{language}: the verifiable route must claim a named unknown, got `{}`",
            response.intent
        );
    }
}

/// An edit instruction with no quoted operand reaches the verifiable route
/// before the quoted-operand parser declines it.
#[test]
fn an_edit_instruction_reaches_the_verifiable_route_before_text_manipulation() {
    for language in LANGUAGES {
        let paraphrase = case("instructed_edit", language);
        let response = FormalAiEngine.answer(&paraphrase.prompt);
        assert_eq!(
            response.intent, "verifiable_task",
            "{language}: an instructed edit is a verifiable task, got `{}`: {}",
            response.intent, response.answer
        );
    }
}

/// The new route is strictly additive: every prompt the conversational suite
/// records still routes exactly where it routed before.
#[test]
fn no_existing_handler_loses_a_prompt_it_answers_today() {
    let cases = conversational_cases();
    assert!(
        cases.len() >= 228,
        "the conversational suite must still carry its recorded wordings, found {}",
        cases.len()
    );

    let mut moved: Vec<String> = Vec::new();
    for (prompt, expected) in &cases {
        let response = FormalAiEngine.answer(prompt);
        if response.intent != *expected {
            moved.push(format!(
                "{prompt:?} moved from `{expected}` to `{}`",
                response.intent
            ));
        }
    }
    assert!(
        moved.is_empty(),
        "the verifiable route may not take a prompt away from the handler that answers it: {moved:?}"
    );
}
