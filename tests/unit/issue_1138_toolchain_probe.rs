//! Issue #1138 B6 (plan 06, L1–L3): availability is probed, never asserted.
//!
//! Today every catalogue row states availability as a constant a human typed.
//! A probe is the only thing that may set it, and there are three verdicts, not
//! two: "we have not looked" is not a synonym for "it does not work".

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::prerequisite::probe::{ProbeVerdict, ToolchainProbe, probe_command};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn probe(program: &str, argv: &[&str]) -> ToolchainProbe {
    ToolchainProbe {
        program: program.to_owned(),
        argv: argv.iter().map(|a| (*a).to_owned()).collect(),
        expect: None,
        requires: Vec::new(),
    }
}

/// A program that is not on the path yields `Missing` with the exit code and
/// standard error that were actually observed.
#[test]
fn probe_reports_missing_with_the_observed_exit_code() {
    let verdict = probe_command(
        &probe("formal-ai-no-such-program-1138", &["--version"]),
        &repo_root(),
    );
    match verdict {
        ProbeVerdict::Missing { exit_code, stderr } => {
            assert_eq!(
                exit_code,
                Some(127),
                "a program that is not on the path exits 127"
            );
            assert!(
                !stderr.trim().is_empty(),
                "the observed standard error is recorded, not discarded"
            );
        }
        other => panic!("a missing program must yield Missing, got {other:?}"),
    }
}

/// A program that is present yields the version line that was printed — never a
/// pinned string this repository chose.
#[test]
fn probe_reports_present_with_the_observed_version() {
    let verdict = probe_command(&probe("python3", &["--version"]), &repo_root());
    match verdict {
        ProbeVerdict::Present { version } => assert!(
            !version.trim().is_empty(),
            "the observed version line is recorded"
        ),
        ProbeVerdict::NotProbed { reason } => panic!(
            "python3 is probeable in this environment; NotProbed was reported instead: {reason}"
        ),
        other => panic!("python3 must probe Present here, got {other:?}"),
    }
}

/// "We did not look" is a real answer. It is a distinct verdict, and the
/// catalogue's status type must be able to carry it.
#[test]
fn an_unprobed_toolchain_is_not_reported_as_unavailable() {
    let verdict = probe_command(&probe("", &[]), &repo_root());
    assert!(
        matches!(verdict, ProbeVerdict::NotProbed { .. }),
        "a probe with nothing to run has not looked, got {verdict:?}"
    );
    assert!(
        !matches!(verdict, ProbeVerdict::Missing { .. }),
        "not having looked may never be rendered as missing"
    );

    let types = fs::read_to_string(repo_root().join("src/coding/catalog/types.rs"))
        .expect("src/coding/catalog/types.rs should be readable");
    assert!(
        types.contains("NotProbed"),
        "ExecutionStatus must carry a third variant for the unprobed case"
    );
    assert!(
        types.contains("fn from_verdict"),
        "the catalogue status must be derivable from a live probe verdict"
    );
}

/// No catalogue row may state its own availability. The literals go to seed and
/// the status comes from a probe.
#[test]
fn catalog_status_comes_from_a_probe_not_a_constant() {
    let languages = fs::read_to_string(repo_root().join("src/coding/catalog/languages.rs"))
        .expect("src/coding/catalog/languages.rs should be readable");
    let asserted: Vec<&str> = languages
        .lines()
        .filter(|line| {
            let code = line.split("//").next().unwrap_or(line);
            code.contains("ExecutionStatus::Verified")
                || code.contains("ExecutionStatus::Unavailable")
        })
        .collect();
    assert!(
        asserted.is_empty(),
        "no catalogue row may assert its own availability; {} rows still do: {asserted:?}",
        asserted.len()
    );
}
