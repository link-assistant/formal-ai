//! Regression coverage for issue #1047: short tasks were blocking the critical
//! path.
//!
//! `cargo nextest --partition slice:N/D` splits by test *index*, which is
//! uncorrelated with how long a test takes. Measured across the eight macOS
//! slices of run 32591020809 -- 2895 tests, 4704s of work:
//!
//! ```text
//! index order  worst partition 870s  spread 468s
//! LPT          worst partition 588s  spread   0s   (= the ideal 4704/8)
//! ```
//!
//! 282s of the critical path spent waiting on one unlucky partition while the
//! other seven sat idle. The same pattern showed *inside* a partition: the
//! quarter of tests that finished last averaged 4.31s against 0.70s for the
//! quarter that finished first -- the long ones were being started last.
//!
//! The rule this pins, which `CONTRIBUTING.md` states for all future fan-outs:
//! **start the longest work first.** A long task started last runs alone on a
//! machine everything else has already finished waiting for, and that tail is
//! pure serial time. Sorting descending and packing short work behind it is
//! longest-processing-time-first; Graham proved it lands within
//! 4/3 - 1/(3m) of optimal, and on this workload it hits optimal exactly.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// Recorded durations, longest first.
fn recorded_durations() -> Vec<(String, f64)> {
    let text = repository_file("data/meta/test-durations.lino");
    // Canonical Links Notation: `test <name>` then an indented `seconds <n>`.
    let mut out: Vec<(String, f64)> = Vec::new();
    let mut pending: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("test ") {
            pending = name.contains("::").then(|| name.to_string());
        } else if let Some(seconds) = trimmed.strip_prefix("seconds ")
            && let Some(name) = pending.take()
            && let Ok(seconds) = seconds.parse::<f64>()
        {
            out.push((name, seconds));
        }
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Longest-first onto the emptiest bin -- the same algorithm the planner runs.
fn plan(tests: &[(String, f64)], partitions: usize) -> Vec<f64> {
    let mut load = vec![0.0f64; partitions];
    for (_, seconds) in tests {
        let lightest = load
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(index, _)| index);
        load[lightest] += seconds;
    }
    load
}

/// The planned partitions are balanced, and measurably better than index order.
#[test]
fn the_longest_tests_are_scheduled_first() {
    const PARTITIONS: usize = 8;
    /// The spread the CI gate allows. Kept in step with the planner.
    const MAX_SPREAD_PERCENT: f64 = 25.0;

    let tests = recorded_durations();
    assert!(
        tests.len() > 100,
        "the recorded durations look empty ({} entries); scheduling cannot be \
         balanced against data that is not there",
        tests.len()
    );

    let lpt = plan(&tests, PARTITIONS);
    let worst = lpt.iter().copied().fold(0.0f64, f64::max);
    let best = lpt.iter().copied().fold(f64::MAX, f64::min);
    let spread = (worst - best) / worst * 100.0;

    assert!(
        spread <= MAX_SPREAD_PERCENT,
        "the planned partitions differ by {spread:.1}%, above the \
         {MAX_SPREAD_PERCENT}% limit: the slowest carries {worst:.0}s against \
         {best:.0}s. The critical path then waits on one partition while the \
         others idle."
    );

    // The comparison that justifies the whole mechanism: index order, which is
    // what nextest does unaided.
    let mut by_name = tests;
    by_name.sort_by(|a, b| a.0.cmp(&b.0));
    let mut index_order = [0.0f64; PARTITIONS];
    for (position, (_, seconds)) in by_name.iter().enumerate() {
        index_order[position % PARTITIONS] += seconds;
    }
    let index_worst = index_order.iter().copied().fold(0.0f64, f64::max);

    assert!(
        worst < index_worst,
        "longest-first ({worst:.0}s) must beat index order ({index_worst:.0}s); \
         if it does not, the durations are stale and the plan is scheduling \
         against numbers that no longer describe the suite"
    );
}

/// The macOS lane consumes a planned filter rather than a slice of everything.
///
/// Issue #1059 replaced the eight-way split with a module list: macOS runs the
/// 139 tests whose behaviour can differ from Linux, not 2895 that cannot. The
/// longest-first planner still exists for any future fan-out -- the rule in
/// CONTRIBUTING.md outlives this one lane -- but there is no longer a partition
/// here for it to balance.
#[test]
fn the_macos_lane_runs_a_planned_filter() {
    let workflow = repository_file(".github/workflows/macos-core-tests.yml");

    assert!(
        workflow.contains("plan-test-partition.rs --macos-platform"),
        "the lane selects its tests from `data/meta/macos-platform-tests.lino`"
    );
    assert!(
        !workflow.contains("slice:${{ matrix.partition }}"),
        "sharding ten seconds of tests across eight machines costs a 916MB \
         download each and gains nothing"
    );
}

/// Parallelism is not throttled on CI.
///
/// A runner is billed for the minutes it is alive, so an idle core makes the
/// wait longer at the same cost. The half-CPU cap in `cargo-test.sh` is for a
/// shared laptop and is explicitly conditioned on `CI` being unset.
#[test]
fn ci_is_not_artificially_throttled() {
    for name in [
        "release.yml",
        "macos-core-tests.yml",
        "coverage.yml",
        "security.yml",
    ] {
        let workflow = repository_file(&format!(".github/workflows/{name}"));
        assert!(
            !workflow.contains("max-parallel"),
            "{name} caps its matrix with `max-parallel`, which leaves runners \
             idle while the critical path waits"
        );
    }

    let wrapper = repository_file("scripts/cargo-test.sh");
    assert!(
        wrapper.contains("jobs=$total_cpus"),
        "the wrapper must use every core on CI; the half-CPU cap is for a \
         shared laptop"
    );
}

/// The balance rule is written down for future fan-outs, not just this one.
#[test]
fn the_scheduling_rule_is_documented_for_contributors() {
    let contributing = repository_file("CONTRIBUTING.md");

    assert!(
        contributing.contains("Start the longest work first"),
        "the rule has to be stated where contributors read it, or the next \
         fan-out repeats the mistake"
    );
    assert!(
        contributing.contains("check_test_partition_balance"),
        "the rule should name the gate that enforces it, so a contributor \
         knows what will fail and why"
    );
}
