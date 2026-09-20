//! Issue #1138, plan 14 **wave F** — self-use observations for plan 08.
//!
//! Thirty held-out prompts from `data/benchmarks/verifiable-task-paraphrases.lino`
//! were given to Formal AI through the real `@link-assistant/agent` CLI and
//! through `formal-ai chat`. The expected answers and the observed outcomes are
//! recorded in `data/benchmarks/self-use-verifiable-task.lino`; the raw
//! transcripts are under `docs/case-studies/issue-1138/self-use/`.
//!
//! Every assertion here was observed **failing** on `formal-ai 0.350.0` at
//! commit `dc9b0574607a26f3e1c8bdb8ce93c0c7f786f197` — except for the one
//! language of one family that already works, which is why that test reports
//! which languages are short rather than asserting a blanket failure. Nothing
//! here was repaired by hand and no expectation is ever weakened.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::solver::solve;

/// Where the held-out prompts live. One copy only: this file reads them back
/// rather than restating them, so the corpus and the suite cannot drift.
const PROMPTS: &str = "data/benchmarks/verifiable-task-paraphrases.lino";

/// Where the wave F expectations and observed outcomes live.
const EXPECTATIONS: &str = "data/benchmarks/self-use-verifiable-task.lino";

/// The five languages every held-out corpus in this plan set is written in.
const LANGUAGES: &[&str] = &["en", "ru", "hi", "zh", "es"];

/// The canned description of the retrieval machinery the solver emits in place
/// of retrieving anything. Reciting how search would work is not search — the
/// same class of substitution as counting a read-back plan as executed
/// semantics (`docs/case-studies/issue-710/plans/07-…:80-81`).
const CAPABILITY_DESCRIPTION: &str = "Providers considered";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    inner.replace("\"\"", "\"")
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

/// Every `(language, prompt)` of one family of the plan 08 corpus.
fn prompts(family_id: &str) -> Vec<(String, String)> {
    let text = read(PROMPTS);
    let mut out = Vec::new();
    let mut id = String::new();
    let mut language = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            id = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            language = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("prompt ")
            && id == family_id
        {
            out.push((language.clone(), unquote(value)));
        }
    }
    assert_eq!(
        out.len(),
        LANGUAGES.len(),
        "family `{family_id}` must carry one prompt per registered language in {PROMPTS}"
    );
    out
}

/// One field of one wave F expectation record.
fn expectation(family_id: &str, field: &str) -> String {
    let text = read(EXPECTATIONS);
    let mut id = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            id = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix(&format!("{field} "))
            && id == family_id
        {
            return unquote(value);
        }
    }
    panic!("{EXPECTATIONS} should carry `{field}` for family `{family_id}`");
}

/// Solve every paraphrase of a family and report the languages whose answer is
/// not exactly the expected value.
///
/// Exact equality is the plan 08 contract, not a convenience: an answer of
/// shape `trailing_number` *is* the number, and the one family that already
/// works returns exactly `5` and nothing else. Accepting a number embedded in
/// surrounding prose would accept a number scraped off a web page, which is the
/// specific failure `a_named_unknown_is_solved_rather_than_scraped` exists to
/// catch.
fn languages_not_answering(family_id: &str, expected: &str) -> Vec<String> {
    let mut short = Vec::new();
    for (language, prompt) in prompts(family_id) {
        let answer = solve(&prompt).answer;
        if answer.trim() != expected {
            let head: String = answer.trim().chars().take(120).collect();
            short.push(format!("{language}: {head}"));
        }
    }
    short
}

/// **Wave F observation, plan 08.** A multi-step arithmetic narrative is the
/// plainest possible verifiable task: three subtractions over numbers the prompt
/// supplies, no retrieval needed, one right answer.
///
/// Observed: not derived in any of the five languages. Through the Agent CLI the
/// English prompt is sent to `websearch` and the system tries to open a
/// homework-answer page **for the same word problem**; the fetch returns 403 and
/// the run ends by offering to file an issue report. Looking for someone else's
/// answer is not derivation.
#[test]
fn an_arithmetic_narrative_is_derived_in_every_language() {
    let expected = expectation("arithmetic_narrative", "expected_answer");
    let short = languages_not_answering("arithmetic_narrative", &expected);
    assert!(
        short.is_empty(),
        "plan 08: `{}` is derived from the prompt, never searched for. Languages not \
         answering it:\n{}",
        expectation("arithmetic_narrative", "derivation"),
        short.join("\n")
    );
}

/// **Wave F observation, plan 08.** Two litres minus 750 millilitres, answered in
/// millilitres. Observed: routed to `websearch` in English, two pages about
/// *similar but different* word problems fetched, 403, give up. Spanish reaches
/// the unknown-language fallback.
#[test]
fn a_unit_conversion_is_derived_in_every_language() {
    let expected = expectation("unit_conversion", "expected_answer");
    let short = languages_not_answering("unit_conversion", &expected);
    assert!(
        short.is_empty(),
        "plan 08: `{}` is derived, never searched for. Languages not answering it:\n{}",
        expectation("unit_conversion", "derivation"),
        short.join("\n")
    );
}

/// **Wave F observation, plan 08 — the most serious of the pass.** `Find y: 7 * y
/// = 84` has the answer 12. Through the Agent CLI the prompt is sent to
/// `websearch`, three algebra sites are fetched, and the **raw scraped text** of
/// one of them is emitted into the answer channel — including `y=29`, the
/// solution to a different equation that happened to be on the same page.
///
/// A wrong number lifted from an unrelated problem reached the answer channel
/// with no derivation and no check. This test asserts both halves: the right
/// answer is produced, and the scraped one never appears.
#[test]
fn a_named_unknown_is_solved_rather_than_scraped() {
    let expected = expectation("named_unknown", "expected_answer");
    let forbidden = expectation("named_unknown", "forbidden_answer");
    let mut offenders = Vec::new();
    for (language, prompt) in prompts("named_unknown") {
        let answer = solve(&prompt).answer;
        if answer.contains(&forbidden) {
            offenders.push(format!(
                "{language}: answer carries `{forbidden}`, a number from an unrelated problem"
            ));
        }
        assert_eq!(
            answer.trim(),
            expected,
            "{language}: a named unknown must have the documented exact answer"
        );
    }
    assert!(
        offenders.is_empty(),
        "plan 08: a named unknown is solved by derivation, and no number from a fetched \
         page may reach the answer channel. Observed:\n{}",
        offenders.join("\n")
    );
}

/// **Wave F observation, plan 08 — a partial success.** Counting with
/// multiplicity over a category the seed does not carry works, and works
/// exactly: English answers `5` and nothing else. The same prompt in the other
/// four languages does not reach the same route — Russian is sent to `websearch`
/// and ends in an empty conversation-summary scaffold after three unrelated
/// fetches, Hindi and Chinese receive the capability description, Spanish the
/// unknown-language fallback.
///
/// The capability exists. It is reachable in one language of five.
#[test]
fn a_counted_category_is_answered_in_every_language() {
    let expected = expectation("counted_category", "expected_answer");
    let short = languages_not_answering("counted_category", &expected);
    assert!(
        short.is_empty(),
        "plan 08: a capability that works in English must be reachable in all five \
         registered languages. Languages not answering `{expected}`:\n{}",
        short.join("\n")
    );
}

/// **Wave F observation, plan 08.** No verifiable task may be answered with the
/// canned description of the retrieval machinery. Across all thirty held-out
/// prompts — including the honest-gap family, which produces no number but also
/// names no skill gap and no research trail, so its required negative is right
/// by accident rather than by mechanism.
#[test]
fn no_verifiable_task_is_answered_with_the_search_capability_description() {
    let documented = solve(
        "A baker makes 24 rolls each morning and 18 each afternoon. She sells 35 rolls during the day and gives 4 to her neighbour. How many rolls does she have left at closing time?",
    )
    .answer;
    assert_eq!(documented, "3");
    let families = [
        "arithmetic_narrative",
        "counted_category",
        "instructed_edit",
        "named_unknown",
        "unit_conversion",
        "honest_gap",
    ];
    let mut offenders = Vec::new();
    for family in families {
        for (language, prompt) in prompts(family) {
            if solve(&prompt).answer.contains(CAPABILITY_DESCRIPTION) {
                offenders.push(format!("{family}/{language}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 08: describing the search machinery is not answering the task, and not \
         searching either. {} of 30 held-out prompts answered with the capability \
         description:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}
