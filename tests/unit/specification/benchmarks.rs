//! Issue #304 benchmark-suite coverage.
//!
//! The imported fixtures are allowed to expose current capability gaps. This
//! test keeps the suite deterministic and runnable while reporting the pass
//! and fail counts for reviewers and future synthesis work.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};
use lino_objects_codec::format::parse_indented;

const BENCHMARK_FIXTURE: &str = "data/benchmarks/industry-suite.lino";
const LICENSE_NOTE: &str = "data/benchmarks/LICENSES.md";
const RESEARCH_NOTE: &str = "docs/case-studies/issue-244/raw-data/online-research.md";
const REQUIRED_DOMAINS: [&str; 3] = ["general_problem_solving", "math", "programming"];
const PERMISSIVE_LICENSES: [&str; 3] = ["Apache-2.0", "CC-BY-4.0", "MIT"];
const HELD_OUT_VARIANT: &str = "held_out";

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
    variant: String,
}

#[derive(Debug)]
struct BenchmarkSuite {
    sources: BTreeMap<String, BenchmarkSource>,
    cases: Vec<BenchmarkCase>,
    minimum_pass_count: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct BenchmarkReport {
    passed: usize,
    failed: usize,
    minimum_pass_count: usize,
    failures: Vec<String>,
}

#[test]
fn issue_304_benchmark_suite_reports_pass_fail_counts() {
    let _solver_guard = benchmark_solver_lock();
    let suite = load_suite();

    assert!(
        suite.cases.len() >= 5,
        "the imported benchmark slice should remain populated",
    );
    assert_required_domains_are_covered(&suite);
    assert_every_case_has_a_permissive_source(&suite);

    let report = run_suite(&suite);
    assert_eq!(report.passed + report.failed, suite.cases.len());
    assert_pass_count_floor_is_met(&report);

    let rendered = render_report(&report);
    println!("{rendered}");
    assert!(rendered.contains("benchmark pass/fail counts"));
    assert!(rendered.contains("minimum_pass_count"));
}

#[test]
fn issue_317_benchmark_suite_grows_with_held_out_ratchet() {
    let suite = load_suite();

    assert!(
        suite.cases.len() > 5,
        "issue 317 requires the benchmark suite to grow beyond the issue 304 five-case seed",
    );
    assert_pass_count_floor_is_recorded(&suite);
    assert_held_out_variants_are_covered(&suite);
}

#[test]
fn issue_317_held_out_benchmark_variants_pass_by_derivation() {
    let _solver_guard = benchmark_solver_lock();
    let suite = load_suite();
    let solver = benchmark_solver();
    let held_out_cases = suite
        .cases
        .iter()
        .filter(|case| case.variant == HELD_OUT_VARIANT)
        .collect::<Vec<_>>();

    assert!(
        !held_out_cases.is_empty(),
        "issue 317 requires held-out/paraphrased benchmark variants",
    );

    for case in held_out_cases {
        let response = solver.solve(&case.prompt);
        let missing = case
            .expected_contains
            .iter()
            .filter(|expected| !response.answer.contains(expected.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "{} should pass deterministic checks, missing {:?}; answer: {}",
            case.id,
            missing,
            response.answer,
        );
        assert_case_passes_by_derivation(case, &response.links_notation);
    }
}

#[test]
fn issue_314_numeric_benchmark_cases_compute_with_trace() {
    let _solver_guard = benchmark_solver_lock();
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
fn issue_315_programming_benchmark_cases_synthesize_and_verify() {
    let _solver_guard = benchmark_solver_lock();
    let suite = load_suite();
    let solver = benchmark_solver();
    let cases = [
        (
            "humaneval_0_has_close_elements",
            &[
                "```python",
                "def has_close_elements",
                "return False",
                "Execution status: tests passed",
            ][..],
        ),
        (
            "mbpp_2_similar_elements",
            &[
                "```python",
                "def similar_elements",
                "set",
                "Execution status: tests passed",
            ][..],
        ),
    ];

    for (case_id, expected_fragments) in cases {
        let case = suite
            .cases
            .iter()
            .find(|case| case.id == case_id)
            .unwrap_or_else(|| panic!("missing benchmark case {case_id}"));
        let response = solver.solve(&case.prompt);
        assert_eq!(response.intent, "write_program");
        for expected in expected_fragments {
            assert!(
                response.answer.contains(expected),
                "{} should contain {expected:?}, got {}",
                case.id,
                response.answer
            );
        }
        for expected in [
            "synthesis:candidate",
            "synthesis:candidate_execution",
            "synthesis:verification tests_passed",
            "action_log:run_command",
        ] {
            assert!(
                response.links_notation.contains(expected),
                "{} should record {expected:?} in the trace, got {}",
                case.id,
                response.links_notation
            );
        }
        assert!(
            !response.links_notation.contains("legacy_intent"),
            "{} must not be satisfied by the hello-world seed path: {}",
            case.id,
            response.links_notation
        );
    }
}

#[test]
fn issue_315_unseen_python_function_synthesizes_without_seed_hit() {
    let _solver_guard = benchmark_solver_lock();
    let solver = benchmark_solver();
    let response = solver.solve(
        "Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
    );

    assert_eq!(response.intent, "write_program");
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("def count_vowels"));
    assert!(response.answer.contains("sum("));
    assert!(response.answer.contains("Execution status: tests passed"));
    assert!(response
        .links_notation
        .contains("synthesis:verification tests_passed"));
    assert!(
        !response.links_notation.contains("legacy_intent"),
        "unseen synthesis must not be recorded as a seed hit: {}",
        response.links_notation
    );
}

#[test]
fn issue_315_program_synthesis_accepts_supported_language_wrappers() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let _solver_guard = benchmark_solver_lock();
    let solver = benchmark_solver();
    let cases = [
        Case {
            language: "en",
            prompt: "English request: Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
        },
        Case {
            language: "ru",
            prompt: "Русский запрос: Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
        },
        Case {
            language: "hi",
            prompt: "हिंदी अनुरोध: Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
        },
        Case {
            language: "zh",
            prompt: "中文请求: Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
        },
    ];

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "write_program",
            "{} wrapper should still route to synthesis",
            case.language
        );
        assert!(
            response.answer.contains("def count_vowels"),
            "{} wrapper should synthesize the expected function, got {}",
            case.language,
            response.answer
        );
        assert!(
            response
                .links_notation
                .contains("synthesis:verification tests_passed"),
            "{} wrapper should verify the synthesized candidate, got {}",
            case.language,
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

    assert!(suite.cases.len() > 5);
    assert_pass_count_floor_is_recorded(&suite);
    assert_required_domains_are_covered(&suite);
    assert_every_case_has_a_permissive_source(&suite);
    assert_held_out_variants_are_covered(&suite);
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
    let mut minimum_pass_count = 0;
    for record in parse_records(text) {
        match record.kind.as_str() {
            "benchmark_suite" => {
                let raw_floor = field_value(&record.fields, "minimum_pass_count");
                if !raw_floor.is_empty() {
                    minimum_pass_count = raw_floor.parse::<usize>().unwrap_or_else(|err| {
                        panic!("invalid minimum_pass_count `{raw_floor}`: {err}")
                    });
                }
            }
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
                variant: field_value(&record.fields, "variant"),
            }),
            _ => {}
        }
    }
    BenchmarkSuite {
        sources,
        cases,
        minimum_pass_count,
    }
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

fn assert_pass_count_floor_is_recorded(suite: &BenchmarkSuite) {
    assert!(
        suite.minimum_pass_count > 0,
        "benchmark suite must record a monotonic minimum_pass_count",
    );
    assert!(
        suite.minimum_pass_count <= suite.cases.len(),
        "minimum_pass_count={} cannot exceed suite size {}",
        suite.minimum_pass_count,
        suite.cases.len(),
    );
}

fn assert_pass_count_floor_is_met(report: &BenchmarkReport) {
    assert!(
        report.passed >= report.minimum_pass_count,
        "benchmark pass-count floor dropped: passed={} minimum_pass_count={}\n{}",
        report.passed,
        report.minimum_pass_count,
        render_report(report),
    );
}

fn assert_held_out_variants_are_covered(suite: &BenchmarkSuite) {
    let expected_sources = suite.sources.keys().cloned().collect::<BTreeSet<_>>();
    let held_out_sources = suite
        .cases
        .iter()
        .filter(|case| case.variant == HELD_OUT_VARIANT)
        .map(|case| case.source.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        held_out_sources, expected_sources,
        "each benchmark source needs a held-out/paraphrased anti-memorization variant",
    );
}

fn assert_case_passes_by_derivation(case: &BenchmarkCase, links_notation: &str) {
    let required_marker = match case.source.as_str() {
        "humaneval" | "mbpp" => "synthesis:verification tests_passed",
        "gsm8k" => "composition:remainder",
        "math" => "composition:substitution",
        "bigbench_object_counting" => "composition:count",
        source => panic!("missing derivation marker rule for benchmark source {source}"),
    };
    assert!(
        links_notation.contains(required_marker),
        "{} should pass via derivation marker `{}`; links:\n{}",
        case.id,
        required_marker,
        links_notation,
    );
    assert!(
        !links_notation.contains("legacy_intent"),
        "{} should not be satisfied by a legacy seed lookup; links:\n{}",
        case.id,
        links_notation,
    );
}

fn run_suite(suite: &BenchmarkSuite) -> BenchmarkReport {
    let solver = benchmark_solver();
    let mut passed = 0;
    let mut failures = Vec::new();
    for case in &suite.cases {
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
        failed: suite.cases.len() - passed,
        minimum_pass_count: suite.minimum_pass_count,
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

fn benchmark_solver_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("benchmark solver lock should not be poisoned")
}

fn render_report(report: &BenchmarkReport) -> String {
    let mut out = format!(
        "benchmark pass/fail counts: passed={} failed={} total={} minimum_pass_count={}\n",
        report.passed,
        report.failed,
        report.passed + report.failed,
        report.minimum_pass_count,
    );
    for failure in &report.failures {
        out.push_str("FAIL ");
        out.push_str(failure);
        out.push('\n');
    }
    out
}
