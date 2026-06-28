use std::fs;
use std::path::Path;

#[test]
fn issue_492_release_badge_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #492 Release Badge Stability",
            "| R360 ",
            "| R361 ",
            "| R362 ",
            "| R363 ",
            "| R364 ",
            "| R365 ",
            "| R366 ",
            "| R367 ",
            "| R368 ",
            "| R369 ",
            "crate_release_badges",
            "readme_keeps_traditional_ci_and_artifact_badges",
            "rust-ai-driven-development-pipeline-template#85",
            "docs/case-studies/issue-492",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "actions/workflows/release.yml/badge.svg?branch=main",
            "actions/workflows/desktop-release.yml/badge.svg?branch=main",
            "img.shields.io/crates/v/formal-ai?label=crates.io&style=flat",
            "img.shields.io/docsrs/formal-ai?label=docs.rs&style=flat",
            "img.shields.io/badge/rust-1.96%2B-blue.svg",
            "codecov.io/gh/link-assistant/formal-ai/branch/main/graph/badge.svg",
            "img.shields.io/badge/license-Unlicense-blue.svg",
        ],
    );

    let release_script = read(root.join("scripts/create-github-release.rs"));
    assert_contains_all(
        "scripts/create-github-release.rs",
        &release_script,
        &[
            "fn crate_release_badges",
            "img.shields.io/badge/crates.io-",
            "img.shields.io/badge/docs.rs-",
            "https://crates.io/crates/{crate_name}/{version}",
            "https://docs.rs/{crate_name}/{version}",
            "crate_release_badges_use_static_artifact_links_not_live_status",
        ],
    );
    assert!(
        !release_script.contains("https://docs.rs/{crate_name}/badge.svg"),
        "release script should not render live docs.rs badge SVGs for release notes"
    );
    assert!(
        !release_script.contains("https://img.shields.io/crates/v/{crate_name}"),
        "release script should not render live crates.io version badges for release notes"
    );

    let case_study = read(root.join("docs/case-studies/issue-492/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-492/README.md",
        &case_study,
        &[
            "# Issue 492 Case Study",
            "## 2. Collected Data",
            "## 3. Timeline",
            "## 4. Requirements",
            "## 5. Root Cause",
            "## 6. Implemented Design",
            "## 7. Template Comparison",
            "## 8. Prior Art And Online Research",
            "## 9. Verification",
            "R360",
            "R369",
            "release-v0.205.0.json",
            "desktop-release-27572474798.log",
            "repro-crate-release-badges-before.log",
            "focused-badge-tests.log",
            "cargo-test-unit.log",
            "manual-file-size-scan.log",
            "template-badge-release-patterns.txt",
            "rust-ai-driven-development-pipeline-template#85",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-492/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-492/raw-data/online-research.md",
        &research,
        &[
            "docs.rs badge documentation",
            "Shields.io badge documentation",
            "GitHub Actions workflow status badge documentation",
            "https://img.shields.io/badge/<label>-<message>-<color>",
        ],
    );

    for relative in [
        "docs/case-studies/issue-492/assets/issue-screenshot.png",
        "docs/case-studies/issue-492/raw-data/issue-492.json",
        "docs/case-studies/issue-492/raw-data/issue-492-comments.json",
        "docs/case-studies/issue-492/raw-data/pr-583.json",
        "docs/case-studies/issue-492/raw-data/pr-583-conversation-comments.json",
        "docs/case-studies/issue-492/raw-data/pr-583-review-comments.json",
        "docs/case-studies/issue-492/raw-data/pr-583-reviews.json",
        "docs/case-studies/issue-492/raw-data/release-list-summary.json",
        "docs/case-studies/issue-492/raw-data/release-v0.205.0.json",
        "docs/case-studies/issue-492/raw-data/release-v0.205.0-peeled-commit-runs.json",
        "docs/case-studies/issue-492/raw-data/desktop-release-27572474798.log",
        "docs/case-studies/issue-492/raw-data/repro-crate-release-badges-before.log",
        "docs/case-studies/issue-492/raw-data/repro-crate-release-badges-after.log",
        "docs/case-studies/issue-492/raw-data/focused-badge-tests.log",
        "docs/case-studies/issue-492/raw-data/cargo-fmt-check.log",
        "docs/case-studies/issue-492/raw-data/cargo-test-unit.log",
        "docs/case-studies/issue-492/raw-data/check-file-size-unit-tests.log",
        "docs/case-studies/issue-492/raw-data/manual-file-size-scan.log",
        "docs/case-studies/issue-492/raw-data/template-js-head.txt",
        "docs/case-studies/issue-492/raw-data/template-rust-head.txt",
        "docs/case-studies/issue-492/raw-data/template-python-head.txt",
        "docs/case-studies/issue-492/raw-data/template-csharp-head.txt",
        "docs/case-studies/issue-492/raw-data/template-workflow-files.txt",
        "docs/case-studies/issue-492/raw-data/template-script-files.txt",
        "docs/case-studies/issue-492/raw-data/template-badge-release-patterns.txt",
        "docs/case-studies/issue-492/raw-data/reported-rust-template-issue-85.json",
        "docs/case-studies/issue-492/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #492 traceability"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
