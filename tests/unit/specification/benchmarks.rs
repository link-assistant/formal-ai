//! Issue #304 benchmark-suite coverage.
//!
//! The imported fixtures are allowed to expose current capability gaps. This
//! test keeps the suite deterministic and runnable while reporting the pass
//! and fail counts for reviewers and future E26 work.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};
use lino_objects_codec::format::parse_indented;

const BENCHMARK_FIXTURE: &str = "data/benchmarks/industry-suite.lino";
const LICENSE_NOTE: &str = "data/benchmarks/LICENSES.md";
const RESEARCH_NOTE: &str = "docs/case-studies/issue-244/raw-data/online-research.md";
const REQUIRED_DOMAINS: [&str; 3] = ["general_problem_solving", "math", "programming"];
const PERMISSIVE_LICENSES: [&str; 3] = ["Apache-2.0", "CC-BY-4.0", "MIT"];

#[derive(Debug)]
struct LinoRecord {
    kind: String,
    id: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
struct BenchmarkSource {
    id: String,
    domain: String,
    license: String,
    source_ref: String,
}

#[derive(Debug)]
struct BenchmarkCase {
    id: String,
    source: String,
    domain: String,
    prompt: String,
    expected_contains: Vec<String>,
    allow_current_failure: bool,
}

#[derive(Debug)]
struct BenchmarkSuite {
    sources: BTreeMap<String, BenchmarkSource>,
    cases: Vec<BenchmarkCase>,
}

#[derive(Debug, PartialEq, Eq)]
struct BenchmarkReport {
    passed: usize,
    failed: usize,
    failures: Vec<String>,
}

#[test]
fn issue_304_benchmark_suite_reports_pass_fail_counts() {
    let suite = load_suite();

    assert_eq!(suite.cases.len(), 5, "the initial imported slice is small");
    assert_required_domains_are_covered(&suite);
    assert_every_case_has_a_permissive_source(&suite);

    let report = run_suite(&suite.cases);
    assert_eq!(report.passed + report.failed, suite.cases.len());

    let rendered = render_report(&report);
    println!("{rendered}");
    assert!(rendered.contains("benchmark pass/fail counts"));
}

#[test]
fn issue_314_numeric_benchmark_cases_compute_with_trace() {
    let suite = load_suite();
    let solver = benchmark_solver();
    let cases = [
        (
            "gsm8k_test_0_duck_eggs",
            "18",
            &[
                "sub_result:",
                "composition:remainder:",
                "composition:evaluation:",
                "trace:",
            ][..],
        ),
        (
            "math_train_7_algebra_substitution",
            "11",
            &[
                "sub_result:",
                "composition:substitution:",
                "composition:evaluation:",
                "trace:",
            ][..],
        ),
        (
            "bigbench_object_counting_instruments",
            "3",
            &[
                "sub_result:",
                "composition:category:",
                "composition:count:",
                "trace:",
            ][..],
        ),
    ];

    for (case_id, expected, required_prefixes) in cases {
        let case = suite
            .cases
            .iter()
            .find(|case| case.id == case_id)
            .unwrap_or_else(|| panic!("missing benchmark case {case_id}"));
        let response = solver.solve(&case.prompt);
        assert!(
            response.answer.contains(expected),
            "{} should contain {expected:?}, got {}",
            case.id,
            response.answer
        );
        for prefix in required_prefixes {
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with(prefix)),
                "{} missing trace prefix {prefix:?}: {:?}",
                case.id,
                response.evidence_links
            );
        }
        assert!(
            response.links_notation.contains("composition:"),
            "{} should serialize composition steps: {}",
            case.id,
            response.links_notation
        );
    }
}

#[test]
fn issue_304_benchmark_research_note_records_provenance() {
    let root = repo_root();
    let fixture = fs::read_to_string(root.join(BENCHMARK_FIXTURE)).expect("benchmark fixture");
    let license_note = fs::read_to_string(root.join(LICENSE_NOTE)).expect("license note");
    let research = fs::read_to_string(root.join(RESEARCH_NOTE)).expect("research note");

    for expected in [
        "HumanEval",
        "Mostly Basic Python Problems",
        "GSM8K",
        "MATH",
        "BIG-bench object_counting",
        "MIT",
        "Apache-2.0",
        "source_ref",
    ] {
        assert!(
            fixture.contains(expected)
                || license_note.contains(expected)
                || research.contains(expected),
            "missing benchmark provenance marker `{expected}`",
        );
    }
}

#[test]
fn issue_304_benchmark_fixture_parses_with_windows_line_endings() {
    let text = fs::read_to_string(repo_root().join(BENCHMARK_FIXTURE)).expect("benchmark fixture");
    let windows_text = text.replace('\n', "\r\n");

    validate_lino_syntax(&windows_text);
    let suite = parse_suite(&windows_text);

    assert_eq!(suite.cases.len(), 5);
    assert_required_domains_are_covered(&suite);
    assert_every_case_has_a_permissive_source(&suite);
}

fn load_suite() -> BenchmarkSuite {
    let text = fs::read_to_string(repo_root().join(BENCHMARK_FIXTURE)).expect("benchmark fixture");
    validate_lino_syntax(&text);
    parse_suite(&text)
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn validate_lino_syntax(text: &str) {
    for record in split_records(text) {
        parse_indented(&record).expect("benchmark record should be valid Links Notation");
    }
}

fn parse_suite(text: &str) -> BenchmarkSuite {
    let mut sources = BTreeMap::new();
    let mut cases = Vec::new();
    for record in parse_records(text) {
        match record.kind.as_str() {
            "benchmark_source" => {
                let source = BenchmarkSource {
                    id: record.id,
                    domain: field_value(&record.fields, "domain"),
                    license: field_value(&record.fields, "license"),
                    source_ref: field_value(&record.fields, "source_ref"),
                };
                sources.insert(source.id.clone(), source);
            }
            "benchmark_case" => cases.push(BenchmarkCase {
                id: record.id,
                source: field_value(&record.fields, "source"),
                domain: field_value(&record.fields, "domain"),
                prompt: field_value(&record.fields, "prompt"),
                expected_contains: field_values(&record.fields, "expected_contains"),
                allow_current_failure: field_value(&record.fields, "allow_current_failure")
                    == "true",
            }),
            _ => {}
        }
    }
    BenchmarkSuite { sources, cases }
}

fn parse_records(text: &str) -> Vec<LinoRecord> {
    split_records(text)
        .into_iter()
        .map(|record| parse_record(&record))
        .collect()
}

fn split_records(text: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                records.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line.trim_end().to_owned());
        }
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(block: &str) -> LinoRecord {
    let mut lines = block.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().expect("record header");
    let fields = lines
        .map(parse_lino_line)
        .filter(|(name, _)| !name.is_empty())
        .collect::<Vec<_>>();
    let kind = field_value(&fields, "record_type");
    let id = field_value(&fields, "id");
    assert!(!kind.is_empty(), "record `{header}` is missing record_type");
    assert!(!id.is_empty(), "record `{header}` is missing id");
    LinoRecord { kind, id, fields }
}

fn parse_lino_line(line: &str) -> (String, String) {
    let content = line.trim();
    if let Some((name, raw_value)) = content.split_once(' ') {
        (name.to_owned(), unescape_quoted(raw_value.trim()))
    } else {
        (content.to_owned(), String::new())
    }
}

fn unescape_quoted(raw: &str) -> String {
    let inner = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn field_value(fields: &[(String, String)], name: &str) -> String {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then(|| value.clone()))
        .unwrap_or_default()
}

fn field_values(fields: &[(String, String)], name: &str) -> Vec<String> {
    fields
        .iter()
        .filter(|(field_name, _)| field_name == name)
        .map(|(_, value)| value.clone())
        .collect()
}

fn assert_required_domains_are_covered(suite: &BenchmarkSuite) {
    let domains = suite
        .cases
        .iter()
        .map(|case| case.domain.as_str())
        .collect::<BTreeSet<_>>();
    for domain in REQUIRED_DOMAINS {
        assert!(
            domains.contains(domain),
            "missing benchmark domain `{domain}` in {domains:?}",
        );
    }
}

fn assert_every_case_has_a_permissive_source(suite: &BenchmarkSuite) {
    for case in &suite.cases {
        assert!(
            case.allow_current_failure,
            "benchmark case {} should explicitly allow current capability gaps",
            case.id,
        );
        assert!(
            !case.expected_contains.is_empty(),
            "benchmark case {} needs deterministic checks",
            case.id,
        );
        let source = suite
            .sources
            .get(&case.source)
            .unwrap_or_else(|| panic!("missing source {} for case {}", case.source, case.id));
        assert_eq!(
            source.domain, case.domain,
            "case {} should use the same domain as source {}",
            case.id, source.id,
        );
        assert!(
            PERMISSIVE_LICENSES.contains(&source.license.as_str()),
            "source {} has non-permissive license `{}`",
            source.id,
            source.license,
        );
        assert!(
            !source.source_ref.is_empty(),
            "source {} must record exact upstream revision",
            source.id,
        );
    }
}

fn run_suite(cases: &[BenchmarkCase]) -> BenchmarkReport {
    let solver = benchmark_solver();
    let mut passed = 0;
    let mut failures = Vec::new();
    for case in cases {
        let response = solver.solve(&case.prompt);
        let missing = case
            .expected_contains
            .iter()
            .filter(|expected| !response.answer.contains(expected.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if missing.is_empty() {
            passed += 1;
        } else {
            failures.push(format!("{} missing {:?}", case.id, missing));
        }
    }
    BenchmarkReport {
        passed,
        failed: cases.len() - passed,
        failures,
    }
}

fn benchmark_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

fn render_report(report: &BenchmarkReport) -> String {
    let mut out = format!(
        "benchmark pass/fail counts: passed={} failed={} total={}\n",
        report.passed,
        report.failed,
        report.passed + report.failed,
    );
    for failure in &report.failures {
        out.push_str("FAIL ");
        out.push_str(failure);
        out.push('\n');
    }
    out
}
