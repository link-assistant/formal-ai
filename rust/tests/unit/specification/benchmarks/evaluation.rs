use std::sync::{Mutex, MutexGuard, OnceLock};

use super::{
    BTreeSet, BenchmarkCase, BenchmarkReport, BenchmarkSuite, ExecutionSurface, HELD_OUT_VARIANT,
    PERMISSIVE_LICENSES, REQUIRED_DOMAINS, SolverConfig, UniversalSolver,
};

pub(super) fn assert_required_domains_are_covered(suite: &BenchmarkSuite) {
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

pub(super) fn assert_every_case_has_a_permissive_source(suite: &BenchmarkSuite) {
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

pub(super) fn assert_pass_count_floor_is_recorded(suite: &BenchmarkSuite) {
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

pub(super) fn assert_pass_count_floor_is_met(report: &BenchmarkReport) {
    assert!(
        report.passed >= report.minimum_pass_count,
        "benchmark pass-count floor dropped: passed={} minimum_pass_count={}\n{}",
        report.passed,
        report.minimum_pass_count,
        render_report(report),
    );
}

pub(super) fn assert_held_out_variants_are_covered(suite: &BenchmarkSuite) {
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

pub(super) fn assert_case_passes_by_derivation(case: &BenchmarkCase, links_notation: &str) {
    let required_marker = match case.source.as_str() {
        "humaneval" | "mbpp" => "synthesis:verification tests_passed",
        "gsm8k" => "composition:remainder",
        "math" => "composition:substitution",
        "bigbench_object_counting" => "composition:count",
        "arithmetic_reachability_search" => "search:solution",
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

pub(super) fn run_suite(suite: &BenchmarkSuite) -> BenchmarkReport {
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

pub(super) fn benchmark_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

pub(super) fn benchmark_solver_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("benchmark solver lock should not be poisoned")
}

pub(super) fn render_report(report: &BenchmarkReport) -> String {
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
