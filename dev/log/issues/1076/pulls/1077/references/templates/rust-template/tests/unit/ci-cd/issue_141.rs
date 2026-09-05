//! Regression tests for issue #141.
//!
//! The multi-architecture manifest step built its digest list with a
//! single-quoted `printf` format, so `${DOCKERHUB_IMAGE}` was passed through
//! literally and `docker buildx imagetools create` received an invalid image
//! reference. `shellcheck` reports it as SC2016, but no workflow ran
//! `actionlint`, so the defect was never surfaced.

use std::fs;

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|error| panic!("{name} should exist: {error}"))
    .replace("\r\n", "\n")
}

/// The digest list must expand `DOCKERHUB_IMAGE`, so the `printf` format has to
/// be double quoted.
#[test]
fn manifest_digest_list_expands_dockerhub_image() {
    let release = workflow("release.yml");

    assert!(
        release.contains(r#"printf "${DOCKERHUB_IMAGE}@sha256:%s\n" *"#),
        "release.yml must build the digest list with a double-quoted printf format"
    );
    assert!(
        !release.contains(r"printf '${DOCKERHUB_IMAGE}@sha256:%s\n' *"),
        "release.yml still uses the single-quoted printf format (shellcheck SC2016)"
    );
}

/// A dedicated workflow keeps `actionlint` (and the `shellcheck` it bundles)
/// running on every change under `.github/`.
#[test]
fn workflows_are_linted_by_actionlint() {
    let workflows = workflow("workflows.yml");

    assert!(
        workflows.contains("docker://rhysd/actionlint:"),
        "workflows.yml must run actionlint from its Docker image, which bundles shellcheck; \
         a native binary without shellcheck on PATH silently skips the shell checks"
    );
    assert!(
        workflows.contains("paths: ['.github/**']"),
        "the actionlint check must run for changes under .github/"
    );
}

/// `macos-15-intel` and `windows-11-arm` are real hosted runners that
/// actionlint 1.7.7 does not know about. Without this configuration the check
/// would fail on false positives and get disabled.
#[test]
fn actionlint_knows_the_runner_labels_used_by_desktop_release() {
    let config = fs::read_to_string(format!(
        "{}/.github/actionlint.yaml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("actionlint configuration should exist");
    let desktop = workflow("desktop-release.yml");

    for label in ["macos-15-intel", "windows-11-arm"] {
        if desktop.contains(label) {
            assert!(
                config.contains(&format!("- {label}")),
                "actionlint.yaml must declare the {label} runner label used by desktop-release.yml"
            );
        }
    }
}
