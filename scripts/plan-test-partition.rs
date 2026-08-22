#!/usr/bin/env rust-script
//! Assign tests to a partition by longest-processing-time-first, not by index.
//!
//! Issue #1047. `cargo nextest --partition slice:N/D` splits by *test index*,
//! which is uncorrelated with duration. Measured across the eight macOS slices
//! of run 32591020809 -- 2895 tests, 4704s of work:
//!
//! ```text
//! round-robin  worst 870s  spread 468s
//! LPT          worst 588s  spread   0s   (= the ideal 4704/8)
//! ```
//!
//! 282s of the critical path, spent waiting on one unlucky slice while others
//! sat idle. LPT is the standard fix: sort longest first and hand each task to
//! whichever bin is currently emptiest. Graham proved it finishes within
//! 4/3 - 1/(3m) of optimal; on this workload it hits optimal exactly, because
//! the longest single test is 271s against a 588s ideal slice and so cannot
//! dominate a bin on its own.
//!
//! The same rule applies at every level: start the long thing first. A long
//! task started last runs alone while everything else has finished, and that
//! tail is pure serial time added to the critical path.
//!
//! Durations come from `data/meta/test-durations.lino`, recorded from a real
//! run. A test with no recorded duration is treated as `DEFAULT_SECONDS` --
//! new tests are usually fast, and one mis-sized test costs a little balance,
//! never correctness: every test still lands in exactly one partition.
//!
//! Usage:
//!   plan-test-partition.rs --partition <n> --of <d> [--durations <path>]
//!       prints a nextest filter expression for that partition
//!   plan-test-partition.rs --unrecorded --of <d>
//!       prints a filter matching every test with no recorded duration
//!   plan-test-partition.rs --report --of <d>
//!       prints the predicted per-partition load, for the CI gate

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process;

/// Assumed duration for a test with no recording. Deliberately small: an
/// unrecorded test is almost always a new one, and new tests are usually fast.
const DEFAULT_SECONDS: f64 = 0.1;

/// The imbalance a partitioning may show before CI fails. LPT reaches 0% on the
/// recorded data; this leaves room for drift as tests change without letting a
/// regression back to index-order slip through unnoticed.
const MAX_SPREAD_PERCENT: f64 = 25.0;

fn durations(path: &str) -> BTreeMap<String, f64> {
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    // Canonical Links Notation: `test <name>` followed by an indented
    // `seconds <value>`. `data_files::lino_data_files_are_parseable...` rejects
    // anything else, so the format is not free to drift.
    let mut out = BTreeMap::new();
    let mut pending: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("test ") {
            pending = name.contains("::").then(|| name.to_string());
        } else if let Some(seconds) = trimmed.strip_prefix("seconds ") {
            // `rust-script` compiles this on the 2021 edition, where let-chains
            // are not available, so the three conditions nest.
            if let Some(name) = pending.take() {
                if let Ok(seconds) = seconds.parse::<f64>() {
                    out.insert(name, seconds);
                }
            }
        }
    }
    out
}

/// Longest first, each onto the emptiest bin.
fn plan(tests: &[(String, f64)], partitions: usize) -> Vec<Vec<String>> {
    let mut ordered: Vec<&(String, f64)> = tests.iter().collect();
    // Longest first. Ties broken by name so the plan is identical on every
    // machine -- a partition that depends on hash order would run different
    // tests on a rerun and make a failure irreproducible.
    ordered.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut load = vec![0.0f64; partitions];
    let mut bins: Vec<Vec<String>> = vec![Vec::new(); partitions];
    for (name, seconds) in ordered {
        let lightest = load
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap_or(0);
        load[lightest] += seconds;
        bins[lightest].push(name.clone());
    }
    bins
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let value = |flag: &str| -> Option<String> {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    let path = value("--durations").unwrap_or_else(|| "data/meta/test-durations.lino".to_string());
    let recorded = durations(&path);
    let of: usize = value("--of")
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("--of <partitions> is required");
            process::exit(2)
        });

    let tests: Vec<(String, f64)> = recorded
        .iter()
        .map(|(name, seconds)| (name.clone(), *seconds))
        .collect();

    if args.iter().any(|a| a == "--report") {
        let bins = plan(&tests, of);
        let loads: Vec<f64> = bins
            .iter()
            .map(|bin| {
                bin.iter()
                    .map(|name| recorded.get(name).copied().unwrap_or(DEFAULT_SECONDS))
                    .sum()
            })
            .collect();
        let worst = loads.iter().cloned().fold(0.0f64, f64::max);
        let best = loads.iter().cloned().fold(f64::MAX, f64::min);
        let spread = if worst > 0.0 {
            (worst - best) / worst * 100.0
        } else {
            0.0
        };
        for (index, load) in loads.iter().enumerate() {
            println!("partition {}/{of}: {load:.0}s", index + 1);
        }
        println!("spread: {spread:.1}% (limit {:.0}%)", MAX_SPREAD_PERCENT);
        if spread > MAX_SPREAD_PERCENT {
            eprintln!(
                "::error title=Test partitions are unbalanced::The slowest \
                 partition carries {spread:.1}% more than the fastest, above \
                 the {:.0}% limit. The long tests are no longer starting \
                 first, so the critical path waits on one slice while the \
                 others idle. Re-record data/meta/test-durations.lino.",
                MAX_SPREAD_PERCENT
            );
            process::exit(1);
        }
        return;
    }

    if args.iter().any(|a| a == "--unrecorded") {
        // The complement of every recorded name. The workflow intersects this
        // with nextest's own `--partition`, so unrecorded tests are still split
        // across machines by index -- balanced no worse than they are today,
        // and never run twice.
        let all: Vec<String> = tests
            .iter()
            .map(|(name, _)| format!("test(={name})"))
            .collect();
        if all.is_empty() {
            println!("all()");
        } else {
            println!("not ({})", all.join(" + "));
        }
        return;
    }

    let partition: usize = value("--partition")
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("--partition <n> is required");
            process::exit(2)
        });
    let bins = plan(&tests, of);
    let bin = bins.get(partition - 1).cloned().unwrap_or_default();

    // Only the tests this partition owns. The caller pairs this with nextest's
    // own `--partition`, which covers every *unrecorded* test by index -- so a
    // brand-new test still runs exactly once without a re-recording, and no
    // test can run twice: a name is either recorded here or left to nextest,
    // never both.
    //
    // Printing the remainder into every partition instead would look
    // equivalent and silently run every unrecorded test on all eight machines.
    let named: Vec<String> = bin.iter().map(|name| format!("test(={name})")).collect();
    println!("{}", named.join(" + "));
}
