//! Regression coverage for the CI/CD audit in issue #717.

use std::fs;

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn checksum_attestations_use_the_lf_safe_current_action() {
    let desktop = workflow("desktop-release.yml");

    assert_eq!(
        desktop.matches("uses: actions/attest@v4").count(),
        2,
        "desktop and VS Code checksum manifests must use actions/attest v4, whose parser accepts LF on Windows"
    );
    assert!(
        !desktop.contains("actions/attest-build-provenance@v2"),
        "the v2 wrapper splits checksums with the host EOL and treats an LF-only Windows manifest as one oversized subject"
    );
}

#[test]
fn coverage_upload_uses_a_node_24_compatible_action() {
    let release = workflow("release.yml");

    assert!(release.contains("uses: codecov/codecov-action@v7"));
    assert!(!release.contains("uses: codecov/codecov-action@v5"));
}

#[test]
fn workflows_suppress_git_default_branch_hints_at_the_source() {
    for name in ["release.yml", "desktop-release.yml"] {
        let contents = workflow(name);
        assert!(contents.contains("GIT_CONFIG_COUNT: '1'"), "{name}");
        assert!(
            contents.contains("GIT_CONFIG_KEY_0: init.defaultBranch"),
            "{name}"
        );
        assert!(contents.contains("GIT_CONFIG_VALUE_0: main"), "{name}");
    }
}

#[test]
fn artifact_download_and_expected_adhoc_signing_are_warning_free() {
    let desktop = workflow("desktop-release.yml");

    assert!(desktop.contains("uses: actions/download-artifact@v8"));
    assert!(!desktop.contains("uses: actions/download-artifact@v7"));
    assert!(desktop.contains("::notice::macOS signing/notarization secrets are not configured:"));
    assert!(!desktop.contains("::warning::Missing required macOS signing/notarization secrets:"));
}

#[test]
fn vscode_packaging_copies_the_repository_license() {
    let script = fs::read_to_string(format!(
        "{}/vscode/scripts/prepare-resources.mjs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(script.contains("fs.copyFileSync(sourceLicense, outputLicense)"));
}

#[test]
fn file_size_warning_scope_uses_the_event_diff_base() {
    let release = workflow("release.yml");

    assert!(release.contains(
        "FILE_SIZE_WARNING_BASE: ${{ github.event.pull_request.base.sha || github.event.before }}"
    ));
}
