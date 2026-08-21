//! Regression coverage for the complete CI diagnostic audit in issue #730.

use std::{fs, process::Command};

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn every_desktop_job_has_a_bounded_runtime_and_least_privilege() {
    let desktop = workflow("desktop-release.yml");
    assert!(
        desktop.contains("permissions:\n  contents: read"),
        "workflow default must be read-only"
    );
    assert!(
        !desktop
            .split("\njobs:\n")
            .next()
            .unwrap()
            .contains("\nconcurrency:\n")
    );
    for (job, next) in [
        ("resolve", "build"),
        ("build", "vscode"),
        ("vscode", "finalize"),
    ] {
        let body = desktop
            .split(&format!("  {job}:\n"))
            .nth(1)
            .and_then(|tail| tail.split(&format!("\n  {next}:\n")).next())
            .unwrap_or_else(|| panic!("missing {job} job"));
        assert!(body.contains("timeout-minutes:"), "{job} has no timeout");
        assert!(
            body.contains("    permissions:\n"),
            "{job} inherits broad permissions"
        );
    }
    let finalize = desktop.split("  finalize:\n").nth(1).expect("finalize job");
    assert!(finalize.contains("timeout-minutes:"));
    assert!(finalize.contains("    permissions:\n      actions: read\n      contents: write"));
}

#[test]
fn desktop_build_budget_covers_the_measured_windows_arm64_path() {
    let desktop = workflow("desktop-release.yml");
    let build = desktop
        .split("  build:\n")
        .nth(1)
        .and_then(|tail| tail.split("\n  vscode:\n").next())
        .expect("desktop build job");

    // Issue #1017 moved the cap into the matrix (`capmin`) so the packaging
    // retry guard can be derived from the same number instead of a second copy
    // of it. The guarantee this test exists for is unchanged and is asserted
    // against the values themselves rather than against one expression's
    // spelling: every target keeps headroom above the repeated 30-minute
    // Windows ARM64 path, and the three targets that were cancelled at 40
    // minutes keep the 50 they were raised to.
    assert!(
        build.contains("    timeout-minutes: ${{ matrix.capmin }}\n"),
        "the desktop build job must stay bounded by its matrix cap"
    );

    let mut seen = 0;
    for entry in build.lines().filter(|line| line.contains("capmin:")) {
        let label = entry
            .split("label: \"")
            .nth(1)
            .and_then(|tail| tail.split('"').next())
            .unwrap_or_else(|| panic!("matrix entry without a label: {entry}"));
        let capmin: u32 = entry
            .split("capmin:")
            .nth(1)
            .map(|tail| tail.trim_start().trim_end_matches([' ', '}']).trim())
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| panic!("matrix entry without a numeric capmin: {entry}"));
        let expected = if label == "macos-x64" || label.starts_with("windows-") {
            50
        } else {
            40
        };
        assert_eq!(
            capmin, expected,
            "{label} must keep its measured headroom above the 30-minute path"
        );
        seen += 1;
    }
    assert_eq!(seen, 6, "every packaged target must carry an explicit cap");
}

#[test]
fn main_pipeline_defaults_to_read_only_permissions() {
    let release = workflow("release.yml");
    assert!(
        release.contains("permissions:\n  contents: read"),
        "ordinary CI jobs must not inherit repository write permissions"
    );
    assert!(
        !release
            .split("\njobs:\n")
            .next()
            .unwrap()
            .contains("\nconcurrency:\n")
    );
}

#[test]
fn reviewed_dependency_warnings_are_narrowly_classified() {
    let root = env!("CARGO_MANIFEST_DIR");
    let script = format!("{root}/scripts/install-node-dependencies.sh");
    let contents = fs::read_to_string(&script).expect("npm classifier");

    for report in ["electron-builder/issues/10016", "vscode-vsce/issues/1290"] {
        assert!(contents.contains(report));
    }
    assert!(!contents.contains("--loglevel=error"));
    assert!(contents.contains("Unexpected npm stderr"));

    let desktop = workflow("desktop-release.yml");
    assert_eq!(
        desktop
            .matches("scripts/install-node-dependencies.sh")
            .count(),
        // Two install steps, plus the pull_request path filter that makes a
        // change to the script re-run the packaging dry run (issue #808).
        3
    );
    assert!(desktop.contains("NODE_OPTIONS: --disable-warning=DEP0005"));
    assert!(
        desktop.contains("actions/download-artifact/issues/484")
            || contents.contains("actions/download-artifact/issues/484")
    );

    let release = workflow("release.yml");
    assert!(release.contains("NODE_OPTIONS: --disable-warning=DEP0040"));
    assert!(release.contains("upstream issue #434"));

    let syntax = Command::new("bash").arg("-n").arg(script).status().unwrap();
    assert!(syntax.success());
}

#[test]
fn real_agent_cli_run_authored_the_requested_evidence_without_filing_an_issue() {
    let root = env!("CARGO_MANIFEST_DIR");
    let finding = fs::read_to_string(format!(
        "{root}/docs/case-studies/issue-730/agent-authored-finding.md"
    ))
    .expect("Agent CLI authored finding");
    assert_eq!(
        finding,
        "Attest release artifacts by path and keep checksum manifests portable"
    );

    let stream = fs::read_to_string(format!(
        "{root}/docs/case-studies/issue-730/agent-cli-final.jsonl"
    ))
    .expect("real Agent CLI stream");
    assert!(stream.contains("Completed the general change request"));
    assert!(stream.contains("agent-authored-finding.md"));
    assert!(!stream.contains("gh issue create"));

    let release = workflow("release.yml");
    assert!(release.contains("existing issue reference does not file a duplicate"));
    assert!(release.contains("For existing issue-730, create file issue-730-finding.md"));
    assert!(release.contains("EXPECT_TEXT: provenance paths are portable"));
}
