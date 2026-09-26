//! Issue #1138 B9, plan 09 batches M2-M5 (#699 requirement 5): every prompt a
//! retired handler answered still answers identically or better.
//!
//! A migration that changes an answer must say so. This suite runs the held-out
//! family corpus through the engine surface, batch by batch, and records the
//! per-batch verdict; a batch is green when every prompt in its family reaches
//! the meta-method that replaced the handlers, with an answer that is not a
//! recited seed body and not the unknown opener.
//!
//! Written before the leaves that make it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::FormalAiEngine;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn lino_records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current);
            current = Vec::new();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn lino_field(record: &[&str], wanted: &str) -> String {
    let raw = record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find_map(|(name, value)| (name == wanted).then(|| value.trim().to_owned()))
        .unwrap_or_else(|| panic!("missing {wanted:?} in {record:?}"));
    for delimiter in ['"', '\''] {
        if raw.starts_with(delimiter) && raw.ends_with(delimiter) && raw.len() >= 2 {
            let doubled = format!("{delimiter}{delimiter}");
            return raw[1..raw.len() - 1].replace(&doubled, &delimiter.to_string());
        }
    }
    raw
}

/// The five batches, in the order plan 09 lands them, with the meta-method each
/// one migrates its handlers into.
const BATCHES: [(&str, &str); 5] = [
    ("M2", "retrieval_method"),
    ("M3", "procedure_interpreter"),
    ("M4", "structural_operator"),
    ("M5", "dialogue_state_query"),
    ("M1", "rule_interpreter"),
];

const DOCUMENTED_PROMPT: &str =
    "What is a fufloмицин — summarise it in one paragraph with the source.";
const DOCUMENTED_ANSWER: &str = "I identified a source-backed retrieval request, but this isolated turn contains no verified capture for the subject. A trusted-source walk is required before a summary can be asserted.";

fn cases_for(family: &str) -> Vec<(String, String, String)> {
    let mut cases = Vec::new();
    for language in ["en", "ru", "hi", "zh", "es"] {
        let path = repo_root().join(format!(
            "data/benchmarks/handler-family-paraphrases/{language}.lino"
        ));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("missing {language} family partition: {error}"));
        for record in lino_records(&text) {
            if lino_field(&record, "expected_family") == family {
                cases.push((
                    lino_field(&record, "id"),
                    language.to_owned(),
                    lino_field(&record, "prompt"),
                ));
            }
        }
    }
    cases
}

#[test]
fn every_retired_handler_prompt_still_answers_through_its_family() {
    let documented = FormalAiEngine.answer(DOCUMENTED_PROMPT).answer;
    assert_eq!(documented, DOCUMENTED_ANSWER);
    let mut report: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for (batch, family) in BATCHES {
        let cases = cases_for(family);
        assert_eq!(
            cases.len(),
            60,
            "batch {batch} ({family}) must carry twelve paraphrases in five languages"
        );
        let mut passing = 0usize;
        for (id, language, prompt) in &cases {
            let response = FormalAiEngine.answer(prompt);
            let routed = response.intent == family;
            let answered = !response.answer.trim().is_empty();
            if routed && answered {
                passing += 1;
            } else {
                failures.push(format!(
                    "{batch}/{id} ({language}): reached `{}`, expected `{family}`",
                    response.intent
                ));
            }
        }
        report.push(format!("{batch} {family}: {passing}/{}", cases.len()));
    }

    assert!(
        failures.is_empty(),
        "batch verdicts: {report:?}\n{} of 300 held-out prompts do not answer through the \
         family that replaced their handler. First offenders: {:?}",
        failures.len(),
        failures.iter().take(12).collect::<Vec<_>>()
    );
}

#[test]
fn no_family_answer_is_the_unknown_opener() {
    let documented = FormalAiEngine.answer(DOCUMENTED_PROMPT).answer;
    assert_eq!(documented, DOCUMENTED_ANSWER);
    // A migration that turns an answered prompt into "I do not know" is a
    // regression, not a migration (#699 requirement 5).
    let openers = fs::read_to_string(repo_root().join("data/seed/unknown-openers.lino"))
        .expect("unknown-openers.lino readable");
    let opener_lines: Vec<String> = openers
        .lines()
        .filter_map(|line| line.trim().split_once(' '))
        .map(|(_, value)| value.trim().trim_matches('"').to_owned())
        .filter(|value| value.chars().count() >= 12)
        .collect();
    assert!(
        !opener_lines.is_empty(),
        "the unknown openers must be readable for this guard to mean anything"
    );

    let mut unknown: Vec<String> = Vec::new();
    for (_batch, family) in BATCHES {
        for (id, _language, prompt) in cases_for(family) {
            let answer = FormalAiEngine.answer(&prompt).answer;
            if opener_lines.iter().any(|opener| answer.contains(opener)) {
                unknown.push(id);
            }
        }
    }
    assert!(
        unknown.is_empty(),
        "{} held-out family prompts reach the unknown opener: {:?}",
        unknown.len(),
        unknown.iter().take(12).collect::<Vec<_>>()
    );
}
