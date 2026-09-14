//! Regression coverage for issue #1059: macOS re-ran tests that cannot differ.
//!
//! The lane ran the same 2895 tests as Linux, on the same `cfg(unix)` code.
//! No conditional in `src/` distinguishes macOS from Linux -- all eight are
//! `cfg(unix)`, true on both -- so the Rust logic could not behave differently
//! there. Every macOS-only failure this repository has recorded came from the
//! environment: `timeout` absent, bash 3.2 without `mapfile`, subprocess and
//! path handling.
//!
//! The cost was not theoretical. Each of eight slices downloaded a 916MB
//! archive -- 7GB per run -- and two slices of run 32706091539 could not finish
//! that transfer in three attempts while six did it in 20-81s, taking `main`
//! red for a reason no commit caused.
//!
//! The lane now runs the 139 tests named in
//! `data/meta/macos-platform-tests.lino`: about ten seconds, one runner, one
//! download.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// The list says why each module is on it.
///
/// A bare list of names invites the next person to add "just one more" until it
/// is the whole suite again. The file carries the reasoning that makes the list
/// reviewable.
#[test]
fn the_macos_test_list_records_its_own_rationale() {
    let listed = repository_file("data/meta/macos-platform-tests.lino");

    for key in ["purpose", "rationale", "cost", "extend"] {
        assert!(
            listed.contains(&format!("  {key} ")),
            "`data/meta/macos-platform-tests.lino` must state `{key}`; a list \
             with no reasoning cannot be reviewed, only copied"
        );
    }
}

/// One runner, because the work no longer justifies sharding.
#[test]
fn the_macos_lane_uses_a_single_runner() {
    let workflow = repository_file(".github/workflows/macos-core-tests.yml");

    assert_eq!(
        workflow.matches("- { partition:").count(),
        1,
        "sharding roughly ten seconds of tests multiplies checkout, toolchain \
         and a 916MB download across machines for no gain"
    );
}

/// Contributors are told the rule, and told how to extend it.
#[test]
fn the_macos_testing_rule_is_documented() {
    let contributing = repository_file("CONTRIBUTING.md");

    assert!(
        contributing.contains("macOS runs platform tests, not the whole suite"),
        "the rule has to be where contributors read it, or the next person \
         restores the full suite as an obvious improvement"
    );
    assert!(
        contributing.contains("data/meta/macos-platform-tests.lino"),
        "and it has to name the file to extend, or a genuine macOS difference \
         gets tested by widening the filter back to everything"
    );
}
