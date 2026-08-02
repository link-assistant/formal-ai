//! The monotonic upstream ratchet (issue #698, requirement 4).
//!
//! The repository-local suites already refuse to let a `minimum_pass_count`
//! fall. This extends the same discipline to upstream suites: a pull request
//! may neither lower a suite's floor nor rewrite a recorded upstream pass count
//! downwards, and every recorded run must still clear the floor.

use std::collections::BTreeMap;

use super::ledger::{Ledger, ResultEntry};

/// Check a ledger on its own: internally consistent rows that clear their floor.
#[must_use]
pub fn violations(ledger: &Ledger) -> Vec<String> {
    let suites = ledger.suites();
    let results = ledger.results();
    let mut violations = Vec::new();

    for result in &results {
        let Some(suite) = suites.get(&result.suite) else {
            violations.push(format!(
                "result row for `{}` has no external_benchmark_suite record",
                result.suite
            ));
            continue;
        };
        if result.passed + result.failed != result.total {
            violations.push(format!(
                "{} {}: passed={} + failed={} does not equal total={}",
                result.suite, result.date, result.passed, result.failed, result.total
            ));
        }
        if result.total != result.slice {
            violations.push(format!(
                "{} {}: total={} does not equal slice={}",
                result.suite, result.date, result.total, result.slice
            ));
        }
        if result.slice == suite.ratchet_slice && result.passed < suite.minimum_pass_count {
            violations.push(format!(
                "{} {}: passed={} is below the recorded minimum_pass_count={}",
                result.suite, result.date, result.passed, suite.minimum_pass_count
            ));
        }
    }

    for (suite_id, suite) in &suites {
        let best = best_pass_count(&results, suite_id, suite.ratchet_slice);
        if let Some(best) = best {
            if suite.minimum_pass_count < best {
                violations.push(format!(
                    "{suite_id}: minimum_pass_count={} is below the best recorded pass count {best} at slice {}",
                    suite.minimum_pass_count, suite.ratchet_slice
                ));
            }
        }
    }

    violations.extend(non_monotonic_history(&results));
    violations
}

/// Check a ledger against its previous revision: nothing recorded may shrink.
#[must_use]
pub fn regressions(previous: &Ledger, current: &Ledger) -> Vec<String> {
    let mut regressions = Vec::new();
    let previous_suites = previous.suites();
    let current_suites = current.suites();

    for (suite_id, previous_suite) in &previous_suites {
        let Some(current_suite) = current_suites.get(suite_id) else {
            regressions.push(format!(
                "{suite_id}: the external_benchmark_suite record was removed"
            ));
            continue;
        };
        if current_suite.ratchet_slice == previous_suite.ratchet_slice
            && current_suite.minimum_pass_count < previous_suite.minimum_pass_count
        {
            regressions.push(format!(
                "{suite_id}: minimum_pass_count fell from {} to {}",
                previous_suite.minimum_pass_count, current_suite.minimum_pass_count
            ));
        }
    }

    let current_results = index_results(&current.results());
    for (key, previous_result) in index_results(&previous.results()) {
        match current_results.get(&key) {
            None => regressions.push(format!(
                "{} {}: the recorded result row at slice {} was removed",
                key.0, key.1, key.2
            )),
            Some(current_result) if current_result.passed < previous_result.passed => {
                regressions.push(format!(
                    "{} {}: recorded pass count fell from {} to {}",
                    key.0, key.1, previous_result.passed, current_result.passed
                ));
            }
            Some(_) => {}
        }
    }

    regressions
}

fn index_results(results: &[ResultEntry]) -> BTreeMap<(String, String, usize), ResultEntry> {
    results
        .iter()
        .map(|result| {
            (
                (result.suite.clone(), result.date.clone(), result.slice),
                result.clone(),
            )
        })
        .collect()
}

fn best_pass_count(results: &[ResultEntry], suite: &str, slice: usize) -> Option<usize> {
    results
        .iter()
        .filter(|result| result.suite == suite && result.slice == slice)
        .map(|result| result.passed)
        .max()
}

/// At a fixed slice size the recorded history may never fall over time.
fn non_monotonic_history(results: &[ResultEntry]) -> Vec<String> {
    let mut grouped: BTreeMap<(String, usize), Vec<&ResultEntry>> = BTreeMap::new();
    for result in results {
        grouped
            .entry((result.suite.clone(), result.slice))
            .or_default()
            .push(result);
    }

    let mut violations = Vec::new();
    for ((suite, slice), mut rows) in grouped {
        rows.sort_by(|left, right| left.date.cmp(&right.date));
        for pair in rows.windows(2) {
            if pair[1].passed < pair[0].passed {
                violations.push(format!(
                    "{suite}: pass count at slice {slice} fell from {} ({}) to {} ({})",
                    pair[0].passed, pair[0].date, pair[1].passed, pair[1].date
                ));
            }
        }
    }
    violations
}
