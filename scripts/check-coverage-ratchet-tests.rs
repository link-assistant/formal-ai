//! Tests for `check-coverage-ratchet.rs`.
//!
//! Issue #895 asks for regression tests over *threshold enforcement and
//! baseline updates* specifically: the gate itself is the deliverable, so a
//! silent break in it would put the repository back where it started — an LCOV
//! file nobody reads. These live beside the script (see
//! `version-and-commit-tests.rs` for the same split) and run in CI via
//! `rust-script --test scripts/check-coverage-ratchet.rs`.

use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("coverage-ratchet-{name}-{nanos}"));
    fs::create_dir_all(&path).unwrap();
    path
}

/// An LCOV record with `hit` of `found` lines covered and one function.
fn lcov_record(path: &str, hit: usize, found: usize, function_hit: bool) -> String {
    let mut out = format!(
        "TN:\nSF:{path}\nFN:1,demo\nFNDA:{},demo\n",
        u8::from(function_hit)
    );
    for line in 1..=found {
        out.push_str(&format!("DA:{line},{}\n", u8::from(line <= hit)));
    }
    out.push_str("end_of_record\n");
    out
}

fn denominator(lines: f64, functions: f64) -> Denominator {
    Denominator {
        label: "Demo".to_string(),
        lcov: "coverage/demo.info".to_string(),
        include: vec!["src/".to_string()],
        exclude: vec!["src/generated/".to_string()],
        lines_percent: lines,
        functions_percent: functions,
        tolerance_percent: 0.5,
        reviewed: "2026-08-05".to_string(),
        evidence: "unit test".to_string(),
        lowered_reason: None,
        inventory: None,
    }
}

fn baseline_with(denominator: Denominator) -> Baseline {
    Baseline {
        policy: "Coverage may not decrease.".to_string(),
        denominators: BTreeMap::from([("demo".to_string(), denominator)]),
    }
}

/// Lay out a repository with a baseline file and an LCOV report.
fn fixture(name: &str, denominator: Denominator, lcov: &str) -> PathBuf {
    let repo = temp_dir(name);
    fs::create_dir_all(repo.join("coverage")).unwrap();
    write_baseline(
        &repo.join("coverage/baseline.json"),
        &baseline_with(denominator),
    )
    .unwrap();
    fs::write(repo.join("coverage/demo.info"), lcov).unwrap();
    repo
}

fn options() -> Options {
    Options {
        only: vec!["demo".to_string()],
        ..Options::default()
    }
}

// --- LCOV parsing -----------------------------------------------------------

#[test]
fn parses_lines_and_functions_from_data_records() {
    let repo = Path::new("/repo");
    let files = parse_lcov(&lcov_record("src/a.rs", 3, 4, true), repo);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "src/a.rs");
    assert_eq!(files[0].lines_found, 4);
    assert_eq!(files[0].lines_hit, 3);
    assert_eq!(files[0].functions_found, 1);
    assert_eq!(files[0].functions_hit, 1);
}

#[test]
fn declares_never_executed_functions_in_the_denominator() {
    let repo = Path::new("/repo");
    let files = parse_lcov("SF:src/a.rs\nFN:1,cold\nDA:1,0\nend_of_record\n", repo);

    assert_eq!(files[0].functions_found, 1, "an FN row without FNDA counts");
    assert_eq!(files[0].functions_hit, 0);
}

#[test]
fn normalizes_absolute_and_dot_relative_source_paths() {
    let repo = Path::new("/repo");
    let files = parse_lcov(
        "SF:/repo/src/a.rs\nDA:1,1\nend_of_record\nSF:./src/b.rs\nDA:1,0\nend_of_record\n",
        repo,
    );

    assert_eq!(
        files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/a.rs", "src/b.rs"],
        "both LCOV producers must land on the same repo-relative path"
    );
}

#[test]
fn merges_repeated_records_by_best_hit_count() {
    let repo = Path::new("/repo");
    // `cargo llvm-cov` can emit one record per test binary for the same file.
    let files = parse_lcov(
        "SF:src/a.rs\nDA:1,0\nDA:2,0\nend_of_record\nSF:src/a.rs\nDA:1,7\nDA:2,0\nend_of_record\n",
        repo,
    );

    assert_eq!(files.len(), 1, "the file is counted once, not twice");
    assert_eq!(files[0].lines_found, 2);
    assert_eq!(files[0].lines_hit, 1);
}

#[test]
fn filters_by_include_and_exclude_prefixes() {
    let repo = Path::new("/repo");
    let lcov = format!(
        "{}{}{}",
        lcov_record("src/a.rs", 1, 2, true),
        lcov_record("src/generated/bundle.js", 10, 10, true),
        lcov_record("tests/helper.rs", 10, 10, true),
    );
    let measurement = measure("demo", &denominator(0.0, 0.0), parse_lcov(&lcov, repo));

    assert_eq!(
        measurement
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/a.rs"],
        "generated and test files must stay out of the denominator"
    );
    assert_eq!(measurement.lines_percent, 50.0);
}

// --- Threshold enforcement --------------------------------------------------

#[test]
fn classifies_regression_improvement_and_hold() {
    assert_eq!(
        classify("lines", 59.0, 60.0, 0.5).status,
        RatchetStatus::Regressed,
        "a drop past the tolerance fails"
    );
    assert_eq!(
        classify("lines", 59.5, 60.0, 0.5).status,
        RatchetStatus::Held,
        "a drop exactly at the tolerance is noise, not a regression"
    );
    assert_eq!(
        classify("lines", 60.0, 60.0, 0.5).status,
        RatchetStatus::Held
    );
    assert_eq!(
        classify("lines", 60.5, 60.0, 0.5).status,
        RatchetStatus::Held
    );
    assert_eq!(
        classify("lines", 61.0, 60.0, 0.5).status,
        RatchetStatus::Improved,
        "a real gain asks for a baseline update"
    );
}

#[test]
fn fails_the_run_when_line_coverage_drops_below_the_baseline() {
    let repo = fixture(
        "drop",
        denominator(80.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );

    let report = run(&repo, &options()).unwrap();

    assert!(report.failed(), "50% against an 80% baseline must fail");
    let outcome = &report.outcomes[0];
    assert_eq!(outcome.measurement.lines_percent, 50.0);
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.starts_with("::error::")
                && message.contains("Coverage ratchet broken")),
        "the failure must be annotated for the run log: {:?}",
        outcome.messages
    );
}

#[test]
fn passes_when_coverage_holds_at_the_baseline() {
    let repo = fixture(
        "hold",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );

    let report = run(&repo, &options()).unwrap();

    assert!(!report.failed());
    assert!(
        report.outcomes[0].messages.is_empty(),
        "holding the line is quiet"
    );
}

#[test]
fn notices_an_improvement_without_failing() {
    let repo = fixture(
        "gain",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 9, 10, true),
    );

    let report = run(&repo, &options()).unwrap();

    assert!(!report.failed(), "more coverage never fails the build");
    assert!(
        report.outcomes[0]
            .messages
            .iter()
            .any(|message| message.starts_with("::notice::")),
        "a gain asks for a baseline update: {:?}",
        report.outcomes[0].messages
    );
}

#[test]
fn fails_when_no_file_matches_the_denominator() {
    // An LCOV report that only contains test files would otherwise measure 0
    // of 0 lines and silently "pass" at 0%.
    let repo = fixture(
        "empty",
        denominator(50.0, 100.0),
        &lcov_record("tests/only.rs", 10, 10, true),
    );

    let report = run(&repo, &options()).unwrap();

    assert!(report.failed());
    assert!(report.outcomes[0]
        .messages
        .iter()
        .any(|message| message.contains("No file in")));
}

#[test]
fn missing_lcov_report_is_an_error_not_a_pass() {
    let repo = fixture(
        "missing",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );
    fs::remove_file(repo.join("coverage/demo.info")).unwrap();

    let error = run(&repo, &options()).unwrap_err();

    assert!(error.contains("Could not read the LCOV report"), "{error}");
}

#[test]
fn unknown_denominator_is_rejected() {
    let repo = fixture(
        "unknown",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );
    let options = Options {
        only: vec!["nope".to_string()],
        ..Options::default()
    };

    let error = run(&repo, &options).unwrap_err();

    assert!(error.contains("Unknown denominator `nope`"), "{error}");
}

// --- Published reports ------------------------------------------------------

#[test]
fn writes_human_and_machine_readable_reports() {
    let repo = fixture(
        "reports",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );

    run(&repo, &options()).unwrap();

    let markdown = fs::read_to_string(repo.join("coverage/summary-demo.md")).unwrap();
    assert!(
        markdown.contains("## Coverage — Demo (`demo`)"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| lines | 5 | 10 | 50.00% | 50.00% | +0.00 pp | held |"),
        "{markdown}"
    );
    assert!(markdown.contains("### Least-covered files"), "{markdown}");

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join("coverage/summary-demo.json")).unwrap())
            .unwrap();
    assert_eq!(json["denominator"], "demo");
    assert_eq!(json["status"], "held");
    assert_eq!(json["lines"]["hit"], 5);
    assert_eq!(json["lines"]["found"], 10);
    assert_eq!(json["lines"]["percent"], 50.0);
    assert_eq!(json["files"][0]["path"], "src/a.rs");
}

#[test]
fn reports_are_written_even_when_the_ratchet_breaks() {
    // The artifacts are how a reviewer diagnoses the failure; withholding them
    // on failure is the state issue #895 is fixing.
    let repo = fixture(
        "reports-fail",
        denominator(90.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );

    let report = run(&repo, &options()).unwrap();

    assert!(report.failed());
    assert!(repo.join("coverage/summary-demo.md").exists());
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join("coverage/summary-demo.json")).unwrap())
            .unwrap();
    assert_eq!(json["status"], "failed");
    assert_eq!(json["lines"]["delta_percent"], -40.0);
}

// --- Baseline updates -------------------------------------------------------

#[test]
fn update_baseline_records_the_measured_numbers() {
    let repo = fixture(
        "raise",
        denominator(50.0, 0.0),
        &lcov_record("src/a.rs", 9, 10, true),
    );
    let options = Options {
        update_baseline: true,
        reviewed: Some("2026-09-01".to_string()),
        evidence: Some("run 42".to_string()),
        ..options()
    };

    let report = run(&repo, &options).unwrap();
    assert!(report.baseline_updated);

    let baseline = load_baseline(&repo.join("coverage/baseline.json")).unwrap();
    let updated = &baseline.denominators["demo"];
    assert_eq!(updated.lines_percent, 90.0);
    assert_eq!(updated.functions_percent, 100.0);
    assert_eq!(updated.reviewed, "2026-09-01");
    assert_eq!(updated.evidence, "run 42");
    assert_eq!(
        updated.lowered_reason, None,
        "raising the floor needs no justification"
    );
}

#[test]
fn lowering_the_baseline_requires_an_explicit_justification() {
    let repo = fixture(
        "lower-refused",
        denominator(90.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );
    let options = Options {
        update_baseline: true,
        ..options()
    };

    let error = run(&repo, &options).unwrap_err();

    assert!(
        error.contains("Refusing to lower the reviewed baseline"),
        "{error}"
    );
    let baseline = load_baseline(&repo.join("coverage/baseline.json")).unwrap();
    assert_eq!(
        baseline.denominators["demo"].lines_percent, 90.0,
        "the committed floor must survive a refused update"
    );
}

#[test]
fn a_justified_decrease_is_recorded_in_the_baseline_file() {
    let repo = fixture(
        "lower-allowed",
        denominator(90.0, 100.0),
        &lcov_record("src/a.rs", 5, 10, true),
    );
    let options = Options {
        update_baseline: true,
        justification: Some("reviewed: removed the vendored parser and its tests".to_string()),
        ..options()
    };

    let report = run(&repo, &options).unwrap();
    assert!(report.baseline_updated);
    assert!(
        !report.failed(),
        "an explicit reviewed update is the sanctioned way down"
    );

    let baseline = load_baseline(&repo.join("coverage/baseline.json")).unwrap();
    let updated = &baseline.denominators["demo"];
    assert_eq!(updated.lines_percent, 50.0);
    assert_eq!(
        updated.lowered_reason.as_deref(),
        Some("reviewed: removed the vendored parser and its tests"),
        "the reason travels with the number in the reviewable diff"
    );
}

#[test]
fn a_later_raise_clears_a_recorded_decrease() {
    let mut lowered = denominator(50.0, 100.0);
    lowered.lowered_reason = Some("reviewed: earlier removal".to_string());
    let repo = fixture(
        "clear-reason",
        lowered,
        &lcov_record("src/a.rs", 9, 10, true),
    );
    let options = Options {
        update_baseline: true,
        ..options()
    };

    run(&repo, &options).unwrap();

    let baseline = load_baseline(&repo.join("coverage/baseline.json")).unwrap();
    assert_eq!(baseline.denominators["demo"].lowered_reason, None);
}

#[test]
fn baseline_round_trips_without_losing_fields() {
    let repo = temp_dir("round-trip");
    let path = repo.join("baseline.json");
    let mut source = denominator(61.5, 58.25);
    source.inventory = Some(Inventory {
        roots: vec!["src/web".to_string()],
        extensions: vec!["js".to_string()],
        unmeasured_list: "coverage/unmeasured.txt".to_string(),
    });
    let baseline = baseline_with(source);

    write_baseline(&path, &baseline).unwrap();

    assert_eq!(load_baseline(&path).unwrap(), baseline);
}

#[test]
fn an_unknown_baseline_field_is_rejected() {
    let repo = temp_dir("unknown-field");
    let path = repo.join("baseline.json");
    fs::write(
        &path,
        r#"{"policy":"p","denominators":{},"minimum_percent":90}"#,
    )
    .unwrap();

    let error = load_baseline(&path).unwrap_err();

    assert!(error.contains("Could not parse baseline"), "{error}");
}

// --- Unmeasured-file inventory ---------------------------------------------

fn inventory_fixture(name: &str) -> (PathBuf, Inventory) {
    let repo = temp_dir(name);
    fs::create_dir_all(repo.join("src/web/worker")).unwrap();
    fs::write(repo.join("src/web/measured.js"), "// measured\n").unwrap();
    fs::write(repo.join("src/web/worker/mirror.js"), "// mirror\n").unwrap();
    fs::write(repo.join("src/web/notes.md"), "not code\n").unwrap();
    (
        repo,
        Inventory {
            roots: vec!["src/web".to_string()],
            extensions: vec!["js".to_string()],
            unmeasured_list: "coverage/browser-unmeasured.txt".to_string(),
        },
    )
}

#[test]
fn inventory_accepts_measured_and_declared_files() {
    let (repo, inventory) = inventory_fixture("inventory-clean");
    let measured = BTreeSet::from(["src/web/measured.js".to_string()]);
    let declared = parse_unmeasured_list(
        "# comment\nsrc/web/worker/mirror.js\tRuns only under the Playwright suite\n",
    );

    let report = check_inventory(&repo, &inventory, &[], &measured, &declared);

    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn inventory_rejects_a_new_unmeasured_file() {
    let (repo, inventory) = inventory_fixture("inventory-new");
    let measured = BTreeSet::from(["src/web/measured.js".to_string()]);

    let report = check_inventory(&repo, &inventory, &[], &measured, &[]);

    assert_eq!(
        report.undeclared,
        vec!["src/web/worker/mirror.js".to_string()],
        "a browser file with neither a test nor a declared reason must fail"
    );
    assert!(!report.is_clean());
}

#[test]
fn inventory_rejects_stale_and_missing_rows() {
    let (repo, inventory) = inventory_fixture("inventory-stale");
    let measured = BTreeSet::from([
        "src/web/measured.js".to_string(),
        "src/web/worker/mirror.js".to_string(),
    ]);
    let declared = parse_unmeasured_list(
        "src/web/measured.js\tstale row\nsrc/web/worker/mirror.js\tstale row\nsrc/web/gone.js\tdeleted file\n",
    );

    let report = check_inventory(&repo, &inventory, &[], &measured, &declared);

    assert_eq!(
        report.stale,
        vec![
            "src/web/measured.js".to_string(),
            "src/web/worker/mirror.js".to_string()
        ],
        "rows for files that are covered now must be pruned"
    );
    assert_eq!(report.missing, vec!["src/web/gone.js".to_string()]);
}

#[test]
fn inventory_requires_a_reason_for_every_row() {
    let (repo, inventory) = inventory_fixture("inventory-reason");
    let measured = BTreeSet::from(["src/web/measured.js".to_string()]);
    let declared = parse_unmeasured_list("src/web/worker/mirror.js\n");

    let report = check_inventory(&repo, &inventory, &[], &measured, &declared);

    assert_eq!(
        report.unexplained,
        vec!["src/web/worker/mirror.js".to_string()]
    );
}

#[test]
fn inventory_skips_excluded_prefixes() {
    let (repo, inventory) = inventory_fixture("inventory-exclude");
    let measured = BTreeSet::from(["src/web/measured.js".to_string()]);
    let exclude = vec!["src/web/worker/".to_string()];

    let report = check_inventory(&repo, &inventory, &exclude, &measured, &[]);

    assert!(
        report.is_clean(),
        "an excluded directory is out of the denominator entirely: {report:?}"
    );
}

#[test]
fn an_inventory_violation_fails_the_run_and_is_published() {
    let repo = temp_dir("inventory-run");
    fs::create_dir_all(repo.join("coverage")).unwrap();
    fs::create_dir_all(repo.join("src/web")).unwrap();
    fs::write(repo.join("src/web/measured.js"), "// measured\n").unwrap();
    fs::write(repo.join("src/web/unmeasured.js"), "// new file\n").unwrap();
    let mut browser = denominator(50.0, 100.0);
    browser.include = vec!["src/web/".to_string()];
    browser.inventory = Some(Inventory {
        roots: vec!["src/web".to_string()],
        extensions: vec!["js".to_string()],
        unmeasured_list: "coverage/browser-unmeasured.txt".to_string(),
    });
    write_baseline(
        &repo.join("coverage/baseline.json"),
        &baseline_with(browser),
    )
    .unwrap();
    fs::write(
        repo.join("coverage/demo.info"),
        lcov_record("src/web/measured.js", 5, 10, true),
    )
    .unwrap();
    fs::write(repo.join("coverage/browser-unmeasured.txt"), "").unwrap();

    let report = run(&repo, &options()).unwrap();

    assert!(
        report.failed(),
        "an undeclared browser file fails the build"
    );
    assert!(report.outcomes[0]
        .messages
        .iter()
        .any(|message| message.contains("src/web/unmeasured.js")));
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join("coverage/summary-demo.json")).unwrap())
            .unwrap();
    assert_eq!(json["inventory"]["undeclared"][0], "src/web/unmeasured.js");
}

// --- Command line -----------------------------------------------------------

#[test]
fn parses_the_documented_flags() {
    let parsed = parse_args(
        [
            "--only",
            "rust",
            "--lcov",
            "rust=lcov.info",
            "--out-dir",
            "coverage",
            "--update-baseline",
            "--justification",
            "reviewed",
        ]
        .into_iter()
        .map(str::to_string),
    )
    .unwrap();

    assert_eq!(parsed.only, vec!["rust".to_string()]);
    assert_eq!(parsed.lcov_overrides["rust"], "lcov.info");
    assert_eq!(parsed.out_dir.as_deref(), Some("coverage"));
    assert!(parsed.update_baseline);
    assert_eq!(parsed.justification.as_deref(), Some("reviewed"));
}

#[test]
fn rejects_a_malformed_lcov_override() {
    let error = parse_args(["--lcov", "lcov.info"].into_iter().map(str::to_string)).unwrap_err();

    assert!(error.contains("--lcov expects"), "{error}");
}

#[test]
fn an_lcov_override_wins_over_the_baseline_path() {
    let repo = fixture(
        "override",
        denominator(50.0, 100.0),
        &lcov_record("src/a.rs", 1, 10, true),
    );
    fs::write(
        repo.join("other.info"),
        lcov_record("src/a.rs", 5, 10, true),
    )
    .unwrap();
    let options = Options {
        lcov_overrides: BTreeMap::from([("demo".to_string(), "other.info".to_string())]),
        ..options()
    };

    let report = run(&repo, &options).unwrap();

    assert_eq!(report.outcomes[0].measurement.lines_percent, 50.0);
    assert!(!report.failed());
}

// --- The committed baseline -------------------------------------------------

#[test]
fn the_committed_baseline_is_valid_and_covers_both_denominators() {
    // `rust-script --test` compiles the script into a generated cargo project
    // and runs the binary from *there*, so the working directory is not the
    // repository. `file!()` is the absolute path of this file, which is inside
    // the real `scripts/` directory, so the repository root is two levels up.
    let repo = Path::new(file!())
        .parent()
        .and_then(Path::parent)
        .expect("the tests file lives in <repo>/scripts/");
    let path = repo.join(DEFAULT_BASELINE);
    assert!(
        path.exists(),
        "the committed baseline is missing: {}",
        path.display()
    );

    let baseline = load_baseline(&path).unwrap();

    assert!(
        baseline.denominators.contains_key("rust"),
        "the Rust production path needs its own honest denominator"
    );
    assert!(
        baseline.denominators.contains_key("browser"),
        "the browser production path needs its own honest denominator"
    );
    for (name, denominator) in &baseline.denominators {
        assert!(
            !denominator.include.is_empty(),
            "`{name}` must say what it measures"
        );
        assert!(
            denominator.lines_percent > 0.0,
            "`{name}` must carry a real, measured floor"
        );
        assert!(
            denominator.tolerance_percent < 5.0,
            "`{name}` tolerance must stay small enough to catch a real drop"
        );
    }
}
