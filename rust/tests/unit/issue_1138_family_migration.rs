//! Issue #1138 B9, plan 09 leaves 16-36: the 300-case held-out family suite.
//!
//! Five meta-methods replace fifty-nine specialized handlers. Every prompt a
//! retired handler answered must still be answered — by the family interpreter
//! that replaced it, from a paraphrase that occurs in no file under
//! `data/seed/` and no file under `src/`.
//!
//! The corpus is `data/benchmarks/handler-family-paraphrases/{en,ru,hi,zh,es}.lino`
//! with `data/benchmarks/handler-family-paraphrases-suite.lino` as its header.
//! Written before the leaves that make it pass (plan 14 wave T).

use std::collections::{BTreeMap, BTreeSet};
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

/// Group a Links Notation document into its top-level records: a record starts
/// at an unindented line and runs to the next one.
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

/// Read one field out of a record, unquoting either delimiter and undoubling an
/// escaped delimiter, exactly as Links Notation defines it.
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

struct FamilyCase {
    id: String,
    language: String,
    family: String,
    prompt: String,
}

fn suite_header() -> Vec<String> {
    let text = fs::read_to_string(
        repo_root().join("data/benchmarks/handler-family-paraphrases-suite.lino"),
    )
    .expect("handler-family-paraphrases-suite.lino readable");
    let records = lino_records(&text);
    let header = &records[0];
    vec![
        lino_field(header, "minimum_pass_count"),
        lino_field(header, "languages"),
        lino_field(header, "families"),
    ]
}

fn family_cases() -> Vec<FamilyCase> {
    let mut cases = Vec::new();
    for language in ["en", "ru", "hi", "zh", "es"] {
        let path = repo_root().join(format!(
            "data/benchmarks/handler-family-paraphrases/{language}.lino"
        ));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("missing {language} family partition: {error}"));
        for record in lino_records(&text) {
            assert_eq!(lino_field(&record, "record_type"), "handler_family_case");
            assert_eq!(
                lino_field(&record, "source"),
                "self_authored_multilingual_variation"
            );
            assert_eq!(lino_field(&record, "language"), language);
            cases.push(FamilyCase {
                id: lino_field(&record, "id"),
                language: language.to_owned(),
                family: lino_field(&record, "expected_family"),
                prompt: lino_field(&record, "prompt"),
            });
        }
    }
    cases
}

#[test]
fn the_family_corpus_is_three_hundred_held_out_cases_in_five_languages() {
    let header = suite_header();
    assert_eq!(header[0], "300", "the suite header declares 300 cases");
    assert_eq!(header[1], "en|ru|hi|zh|es");
    assert_eq!(
        header[2],
        "retrieval_method|procedure_interpreter|structural_operator|dialogue_state_query|rule_interpreter"
    );

    let cases = family_cases();
    assert_eq!(
        cases.len(),
        300,
        "5 families x 5 languages x 12 paraphrases"
    );

    let prompts: BTreeSet<&str> = cases.iter().map(|case| case.prompt.as_str()).collect();
    assert_eq!(prompts.len(), 300, "every paraphrase is distinct");

    let mut per_cell: BTreeMap<(String, String), usize> = BTreeMap::new();
    for case in &cases {
        *per_cell
            .entry((case.family.clone(), case.language.clone()))
            .or_default() += 1;
    }
    assert_eq!(per_cell.len(), 25, "five families in five languages");
    for (cell, count) in &per_cell {
        assert_eq!(*count, 12, "cell {cell:?} must carry twelve paraphrases");
    }
}

#[test]
fn held_out_family_paraphrases_route_to_the_family_interpreter() {
    // The meta-method that replaced a family must claim the family's prompts.
    // Today every one of them reaches a specialized handler or the unknown
    // opener, so this is the suite plan 09's twenty-one migration leaves make
    // green, batch by batch.
    let cases = family_cases();
    let mut failures: Vec<String> = Vec::new();
    for case in &cases {
        let response = FormalAiEngine.answer(&case.prompt);
        if response.intent != case.family {
            failures.push(format!(
                "{} ({}): expected family `{}`, reached `{}`",
                case.id, case.language, case.family, response.intent
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} held-out family paraphrases did not reach their family interpreter. \
         Failures: {:?}",
        failures.len(),
        cases.len(),
        failures
    );
}

#[test]
fn no_answer_is_byte_equal_to_a_seed_body_field() {
    let documented = FormalAiEngine
        .answer("What is a fufloмицин — summarise it in one paragraph with the source.")
        .answer;
    assert_eq!(
        documented,
        "I identified a source-backed retrieval request, but this isolated turn contains no verified capture for the subject. A trusted-source walk is required before a summary can be asserted."
    );
    // #948 items 1-2, permanently: the three canned summary bodies and the
    // three `contains(...)` comparison blocks are deleted, and a family answer
    // is derived from a record rather than recited from a `body` field.
    let mut bodies: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(repo_root().join("data/seed"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("lino") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        for line in text.lines() {
            let trimmed = line.trim();
            for field in ["body ", "fallback_body "] {
                let Some(value) = trimmed.strip_prefix(field) else {
                    continue;
                };
                let value = value.trim().trim_matches('"').trim_matches('\'');
                // A canned answer is recognisable by a whole sentence of it;
                // `<topic>`-style placeholders are cut at the first angle
                // bracket so the fragment compared is literal text.
                let literal = value.split('<').next().unwrap_or(value).trim();
                if literal.len() >= 40 {
                    bodies.push(literal.to_owned());
                }
            }
        }
    }
    assert!(
        !bodies.is_empty(),
        "the seed must still carry `body` fields for this guard to mean anything"
    );

    let mut recitations: Vec<String> = Vec::new();
    for case in family_cases() {
        let answer = FormalAiEngine.answer(&case.prompt).answer;
        let trimmed = answer.trim();
        if let Some(body) = bodies
            .iter()
            .find(|body| *body == trimmed || trimmed.contains(body.as_str()))
        {
            recitations.push(format!(
                "{}: recited {:?}",
                case.id,
                &body[..40.min(body.len())]
            ));
        }
    }
    assert!(
        recitations.is_empty(),
        "{} answers reproduce a seed `body` field byte for byte, which is recitation, \
         not derivation (#948 items 1-2): {:?}",
        recitations.len(),
        recitations.iter().take(12).collect::<Vec<_>>()
    );

    // The permanent half: #948 item 1 is the three canned topic bodies in
    // `data/seed/summary-topics.lino` returned verbatim in any prompt language,
    // and item 2 is the three hard-coded English prose blocks in
    // `src/solver_handlers/research_table.rs`. Plan 09 leaves 20-21 delete both,
    // so neither can come back under a new prompt.
    let topics = fs::read_to_string(repo_root().join("data/seed/summary-topics.lino"))
        .expect("summary-topics.lino readable");
    let canned = topics.matches("\n    body ").count();
    assert_eq!(
        canned, 0,
        "data/seed/summary-topics.lino still holds {canned} per-topic `body` strings that \
         are returned verbatim whatever the prompt's language (#948 item 1); plan 09 \
         leaf 20 replaces them with a retrieval that carries provenance"
    );

    let research =
        fs::read_to_string(repo_root().join("rust/src/solver_handlers/research_table.rs"))
            .unwrap_or_default();
    let comparison_blocks = research.matches("normalized.contains(").count();
    assert_eq!(
        comparison_blocks, 0,
        "rust/src/solver_handlers/research_table.rs still branches on {comparison_blocks} \
         memorized English phrases and answers with hard-coded prose (#948 item 2); \
         plan 09 leaf 21 deletes them"
    );
}
