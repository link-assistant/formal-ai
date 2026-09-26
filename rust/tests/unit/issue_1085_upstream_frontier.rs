//! Issue #1085 (D5.1, D5.2): failing upstream cases feed the learning cycle,
//! and a suite that stands still for three runs is a warning.

use std::fs;
use std::path::Path;

use formal_ai::external_benchmarks::grade::CaseOutcome;
use formal_ai::external_benchmarks::learning::{one_line, render_frontier_record};
use formal_ai::external_benchmarks::ledger::{Ledger, LedgerRecord};
use formal_ai::external_benchmarks::{SuiteRun, ratchet};
use formal_ai::learning_cycle::{
    UPSTREAM_BENCHMARKS_FRONTIER, UPSTREAM_BENCHMARKS_FRONTIER_PARTS, parse_frontier_record,
    recorded_frontier,
};

const LEDGER_PATH: &str = "data/benchmarks/external-results.lino";
const FRONTIER_PATH: &str = "data/meta/learning-frontier-upstream-benchmarks.lino";

fn committed_ledger() -> Ledger {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    Ledger::parse(&fs::read_to_string(root.join(LEDGER_PATH)).expect("ledger must be readable"))
        .expect("the committed ledger must parse")
}

fn result_row(template: &LedgerRecord, date: &str, passed: usize, slice: usize) -> LedgerRecord {
    let mut row = template.clone();
    row.name = format!(
        "external_benchmark_result_gsm8k_{}_{slice}",
        date.replace('-', "_")
    );
    for (key, value) in &mut row.fields {
        match key.as_str() {
            "date" => date.clone_into(value),
            "passed" => *value = passed.to_string(),
            "failed" => *value = (slice - passed).to_string(),
            _ => {}
        }
    }
    row
}

#[test]
fn a_suite_scoring_the_same_three_times_is_reported_stagnant() {
    let mut ledger = committed_ledger();
    let template = ledger
        .records
        .iter()
        .find(|record| {
            record.field("record_type") == Some("external_benchmark_result")
                && record.field("suite") == Some("gsm8k")
                && record.field("slice") == Some("20")
        })
        .cloned()
        .expect("the committed ledger records gsm8k at slice 20");
    let best = ledger
        .results()
        .into_iter()
        .filter(|result| result.suite == "gsm8k" && result.slice == 20)
        .map(|result| result.passed)
        .max()
        .expect("gsm8k has results");
    ledger.records.retain(|record| {
        !(record.field("record_type") == Some("external_benchmark_result")
            && record.field("suite") == Some("gsm8k"))
    });
    for date in ["2027-01-05", "2027-01-12", "2027-01-19"] {
        ledger.records.push(result_row(&template, date, best, 20));
    }
    let stalls = ratchet::stagnant(&ledger);
    assert!(
        stalls
            .iter()
            .any(|stall| stall.starts_with("gsm8k:") && stall.contains("2027-01-19")),
        "three equal runs must be reported: {stalls:?}"
    );
    assert!(
        ratchet::violations(&ledger).is_empty(),
        "standing still is a warning, not a ratchet violation"
    );

    ledger
        .records
        .push(result_row(&template, "2027-01-26", best + 1, 20));
    assert!(
        !ratchet::stagnant(&ledger)
            .iter()
            .any(|stall| stall.starts_with("gsm8k:")),
        "a suite that moved is no longer stagnant"
    );
}

#[test]
fn a_suite_with_fewer_than_three_runs_is_never_stagnant() {
    let mut ledger = committed_ledger();
    ledger.records.retain(|record| {
        record.field("record_type") != Some("external_benchmark_result")
            || record.field("suite") == Some("swebench_lite")
    });
    let swebench_runs = ledger
        .results()
        .into_iter()
        .filter(|result| result.suite == "swebench_lite")
        .count();
    if swebench_runs < 3 {
        assert!(ratchet::stagnant(&ledger).is_empty());
    }
}

fn outcome(id: &str, passed: bool, prompt: &str) -> CaseOutcome {
    CaseOutcome {
        id: id.to_owned(),
        passed,
        detail: if passed {
            String::new()
        } else {
            String::from("expected 4, got 5")
        },
        prompt_excerpt: one_line(prompt, 160),
    }
}

#[test]
fn the_frontier_record_is_rewritten_from_failed_cases_and_keeps_other_suites() {
    let existing = "learning_frontier\n  frontier \"upstream-benchmarks\"\n  frontier_prompt\n    rank \"1\"\n    query \"gsm8k/GSM8K/3\"\n    language \"en\"\n    variation \"gsm8k\"\n    prompt \"Janet has 3 ducks\"\n    engine_intent \"benchmark_failure\"\n  frontier_prompt\n    rank \"2\"\n    query \"mbpp/MBPP/9\"\n    language \"en\"\n    variation \"mbpp\"\n    prompt \"stale mbpp item\"\n    engine_intent \"benchmark_failure\"\n";
    let run = SuiteRun {
        suite: "mbpp".to_owned(),
        slice: 2,
        passed: 1,
        failed: 1,
        total: 2,
        outcomes: vec![
            outcome(
                "MBPP/2",
                false,
                "Write a function to find the similar elements from the given two tuple lists.\nassert similar_elements((3, 4), (4, 5)) == (4,)",
            ),
            outcome(
                "MBPP/3",
                true,
                "Write a python function to identify non-prime numbers.",
            ),
        ],
        unavailable: None,
        solver_version: "0.0.0-test".to_owned(),
    };
    let document = render_frontier_record(existing, &[run], "2026-09-08");
    let items = parse_frontier_record(&document);
    let summary: Vec<(usize, &str, &str, &str)> = items
        .iter()
        .map(|item| {
            (
                item.rank,
                item.query.as_str(),
                item.variation.as_str(),
                item.prompt.as_str(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            (1, "gsm8k/GSM8K/3", "gsm8k", "Janet has 3 ducks"),
            (
                2,
                "mbpp/MBPP/2",
                "mbpp",
                "Write a function to find the similar elements from the given two tuple lists."
            ),
        ],
        "the mbpp items are replaced by this run's failure, gsm8k is kept, ranks are dense"
    );
    assert!(document.contains("  recorded_on \"2026-09-08\"\n"));
    assert!(document.contains("  total_prompts \"2\"\n"));
}

#[test]
fn one_line_keeps_a_value_readable_on_a_single_notation_line() {
    assert_eq!(
        one_line("  first \"quoted\" line\nsecond", 160),
        "first 'quoted' line"
    );
    assert_eq!(
        one_line("\n\n   spaced    out   words  ", 160),
        "spaced out words"
    );
    assert_eq!(one_line("abcdef", 3), "abc…");
    assert_eq!(one_line("", 10), "");
}

#[test]
fn the_committed_upstream_frontier_is_registered_and_replayable() {
    let frontier = recorded_frontier(UPSTREAM_BENCHMARKS_FRONTIER)
        .expect("the upstream frontier must be registered for `formal-ai learn cycle`");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    // The recorded frontier is the base page plus every `-partN` continuation,
    // concatenated in order; empty placeholder parts carry only the header.
    let mut committed = fs::read_to_string(root.join(FRONTIER_PATH)).expect("record must exist");
    for part in 1..=UPSTREAM_BENCHMARKS_FRONTIER_PARTS {
        committed.push_str(
            &fs::read_to_string(root.join(format!(
                "data/meta/learning-frontier-upstream-benchmarks-part{part}.lino"
            )))
            .expect("the provisioned part must exist"),
        );
    }
    assert_eq!(
        frontier.document, committed,
        "the embedded record is the committed pages"
    );
    let items = parse_frontier_record(frontier.document);
    assert!(
        !items.is_empty(),
        "the record carries the last scheduled run's failures"
    );
    for item in &items {
        assert_eq!(item.language, "en");
        assert_eq!(item.engine_intent, "benchmark_failure");
        assert!(
            item.query.starts_with(&format!("{}/", item.variation)),
            "query {} must be suite/case under {}",
            item.query,
            item.variation
        );
        assert!(
            !item.prompt.is_empty(),
            "{} must carry a prompt line",
            item.query
        );
    }
    let ranks: Vec<usize> = items.iter().map(|item| item.rank).collect();
    assert_eq!(
        ranks,
        (1..=items.len()).collect::<Vec<_>>(),
        "ranks are dense"
    );
}

#[test]
fn the_scheduled_workflow_rewrites_and_commits_the_frontier() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let workflow = fs::read_to_string(root.join(".github/workflows/external-benchmarks.yml"))
        .expect("workflow must be readable");
    assert!(
        workflow.contains("--frontier-record data/meta/learning-frontier-upstream-benchmarks.lino")
    );
    assert!(
        workflow.contains(
            "git add data/benchmarks/external-results.lino 'data/meta/learning-frontier-upstream-benchmarks*.lino'"
        ),
        "the scheduled commit covers the base page and every `-partN` continuation"
    );
}

// The 1500-line data cap (issue #960) covers every committed LiNo file, and
// one honest frontier prompt per failed case grows without bound — the mbpp-500
// slice alone contributes hundreds of items on top of the recorded HumanEval
// failures. The ledger therefore splits at `frontier_prompt` boundaries into a
// base page plus `-partN` continuations, each a self-describing document, and
// the learn cycle replays the concatenation.
#[test]
fn a_frontier_wider_than_the_data_cap_is_written_in_bounded_self_describing_parts() {
    let outcomes = (0..300)
        .map(|index| {
            outcome(
                &format!("MBPP/{index}"),
                false,
                &format!("Write a function for case {index}.\nassert f({index}) == {index}"),
            )
        })
        .collect();
    let run = SuiteRun {
        suite: String::from("mbpp"),
        slice: 500,
        passed: 200,
        failed: 300,
        total: 500,
        outcomes,
        unavailable: None,
        solver_version: String::from("0.0.0-test"),
    };
    let document = render_frontier_record("", &[run], "2026-09-18");
    assert!(
        document.lines().count() > 1500,
        "the fixture must actually breach the data cap"
    );

    let pages = formal_ai::external_benchmarks::learning::split_frontier_document(&document, 1400);
    assert!(pages.len() >= 2, "one document over the budget must split");
    for page in &pages {
        assert!(
            page.lines().count() <= 1400,
            "each page honours the line budget"
        );
        assert!(
            page.starts_with("learning_frontier\n"),
            "each page is a self-describing document: {}",
            page.get(..40).unwrap_or(page)
        );
    }
    let replayed: Vec<String> = pages
        .iter()
        .flat_map(|page| {
            parse_frontier_record(page)
                .into_iter()
                .map(|item| item.query)
        })
        .collect();
    let expected: Vec<String> = parse_frontier_record(&document)
        .into_iter()
        .map(|item| item.query)
        .collect();
    assert_eq!(
        replayed, expected,
        "the pages replay to exactly the frontier they split"
    );
}

// The writer provisions exactly `UPSTREAM_BENCHMARKS_FRONTIER_PARTS` part
// files, because the embedded record compiles in exactly that many
// `include_str!` siblings: a shrinking frontier must reset a part to a
// header-only placeholder — never delete it — so no stale frontier item
// survives and the embed stays compilable whatever the frontier's size.
#[test]
fn a_shrinking_frontier_resets_unfilled_parts_to_placeholders_instead_of_deleting_them() {
    let dir = std::env::temp_dir().join(format!("formal-ai-frontier-parts-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir must be creatable");
    let base = dir.join("frontier.lino");

    let wide = "learning_frontier\n  record_type \"learning_frontier_record\"\n  frontier_prompt\n    rank \"1\"\n    query \"mbpp/MBPP/1\"\n    language \"en\"\n    variation \"mbpp\"\n    prompt \"one\"\n    engine_intent \"benchmark_failure\"\n  frontier_prompt\n    rank \"2\"\n    query \"mbpp/MBPP/2\"\n    language \"en\"\n    variation \"mbpp\"\n    prompt \"two\"\n    engine_intent \"benchmark_failure\"\n".to_string();
    let pages = formal_ai::external_benchmarks::learning::split_frontier_document(&wide, 10);
    assert!(pages.len() >= 2, "fixture must split");
    let written = formal_ai::external_benchmarks::learning::write_frontier_pages(
        &base,
        &pages,
        UPSTREAM_BENCHMARKS_FRONTIER_PARTS,
    )
    .expect("wide frontier must write");
    assert_eq!(
        written.len(),
        1 + UPSTREAM_BENCHMARKS_FRONTIER_PARTS,
        "base page plus every provisioned part"
    );
    for part in 1..=UPSTREAM_BENCHMARKS_FRONTIER_PARTS {
        let path = dir.join(format!("frontier-part{part}.lino"));
        assert!(path.exists(), "part {part} must exist after the wide write");
    }
    assert!(
        fs::read_to_string(dir.join("frontier-part1.lino"))
            .expect("part 1 must be readable")
            .contains("frontier_prompt")
    );

    let narrow = String::from(
        "learning_frontier\n  record_type \"learning_frontier_record\"\n  frontier_prompt\n    rank \"1\"\n    query \"mbpp/MBPP/1\"\n    language \"en\"\n    variation \"mbpp\"\n    prompt \"one\"\n    engine_intent \"benchmark_failure\"\n",
    );
    let pages = formal_ai::external_benchmarks::learning::split_frontier_document(&narrow, 1400);
    assert_eq!(pages.len(), 1, "fixture must fit one page");
    formal_ai::external_benchmarks::learning::write_frontier_pages(
        &base,
        &pages,
        UPSTREAM_BENCHMARKS_FRONTIER_PARTS,
    )
    .expect("narrow frontier must write");
    for part in 1..=UPSTREAM_BENCHMARKS_FRONTIER_PARTS {
        let text = fs::read_to_string(dir.join(format!("frontier-part{part}.lino")))
            .unwrap_or_else(|error| panic!("part {part} must survive the rewrite: {error}"));
        assert!(
            !text.contains("frontier_prompt"),
            "part {part} must carry no stale item after the shrinking rewrite"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}
