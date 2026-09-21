//! Independent checks for the benchmark index authored by Formal AI.

use std::collections::BTreeMap;

use formal_ai::external_benchmarks::{Ledger, ResultEntry};

const SOURCE: &str = include_str!(
    "../../../docs/case-studies/issue-710/agent-cli-evidence/final-benchmark-index/final-benchmark-measurements.lino"
);
const INDEX: &str = include_str!("../../../data/meta/benchmark-release-capability-index.lino");

fn quoted_value(line: &str) -> &str {
    line.split_once('"')
        .and_then(|(_, rest)| rest.rsplit_once('"').map(|(value, _)| value))
        .expect("Links Notation value")
}

fn records(document: &str, record_name: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut records = BTreeMap::new();
    let mut current = None::<String>;
    for line in document.lines() {
        if let Some(rest) = line.strip_prefix(&format!("  {record_name} ")) {
            let identifier = rest.trim_matches('"').to_owned();
            records.insert(identifier.clone(), BTreeMap::new());
            current = Some(identifier);
        } else if let Some(field) = line.strip_prefix("    ")
            && let Some(identifier) = &current
            && let Some((name, _)) = field.split_once(' ')
        {
            records
                .get_mut(identifier)
                .expect("current record")
                .insert(name.to_owned(), quoted_value(field).to_owned());
        }
    }
    records
}

#[test]
fn formal_ai_index_is_a_digest_bound_projection_of_its_input() {
    let source = records(SOURCE, "measurement");
    let index = records(INDEX, "derived_formalization");

    assert_eq!(
        index, source,
        "every derived field must match the inspected input"
    );
    assert!(INDEX.starts_with("benchmark_release_index\n"));
    assert!(INDEX.contains(&format!(
        "sha256 \"{}\"",
        formal_ai::source_fetch::sha256_hex(SOURCE.as_bytes())
    )));
    assert!(INDEX.contains("root \"benchmark_release_evidence\""));
}

#[test]
fn release_candidate_measurements_match_the_latest_committed_ledger_rows() {
    let measurements = records(INDEX, "derived_formalization");
    let ledger = Ledger::parse(include_str!(
        "../../../data/benchmarks/external-results.lino"
    ))
    .expect("benchmark ledger");
    // The release candidate rests on complete floors of one slice size; the
    // ledger also carries honest partial scores at larger slices. Selecting
    // by the measurement's own suite and slice keeps the cross-check real --
    // the index must equal the newest committed row of exactly the run size
    // it records -- without demanding the whole suite's newest honest score
    // be a complete pass.
    let latest = ledger.results().into_iter().fold(
        BTreeMap::<(String, String), ResultEntry>::new(),
        |mut rows, row| {
            let key = (row.suite.clone(), row.slice.to_string());
            let replace = rows
                .get(&key)
                .is_none_or(|current| row.date.as_str() > current.date.as_str());
            if replace {
                rows.insert(key, row);
            }
            rows
        },
    );

    for identifier in ["humaneval_cold", "mbpp_live"] {
        let measurement = measurements.get(identifier).expect("release measurement");
        let row = latest
            .get(&(
                measurement["suite_id"].clone(),
                measurement["total_count"].clone(),
            ))
            .expect("latest suite row");
        assert_eq!(measurement["passed_count"], row.passed.to_string());
        assert_eq!(measurement["total_count"], row.total.to_string());
        assert_eq!(
            row.passed, row.total,
            "release coding floor must be complete"
        );
    }

    let cold = &measurements["mbpp_cold"];
    assert_eq!(cold["passed_count"], "18");
    assert_eq!(cold["total_count"], "20");
    assert_ne!(cold["source_requirement"], "none");
}
