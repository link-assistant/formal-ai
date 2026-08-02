//! Regression coverage for the CI/CD audit in issue #717.

use std::{fs, process::Command};

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn attestations_digest_artifacts_without_parsing_cross_platform_checksum_text() {
    let desktop = workflow("desktop-release.yml");

    assert_eq!(
        desktop.matches("uses: actions/attest@v4").count(),
        2,
        "desktop and VS Code artifacts must use the current generic attestation action"
    );
    assert_eq!(desktop.matches("subject-path:").count(), 2);
    assert!(desktop.contains("desktop/release/formal-ai-desktop-*"));
    assert!(desktop.contains("desktop/release/latest*.yml"));
    assert!(desktop.contains("vscode/formal-ai-vscode-*.vsix"));
    assert!(
        !desktop.contains("subject-checksums:"),
        "released actions/attest v4 splits checksum text with the host EOL and fails on LF-only Windows manifests"
    );
    assert!(
        !desktop.contains("actions/attest-build-provenance@v2"),
        "the legacy wrapper has the same host-EOL checksum parsing failure"
    );
}

#[test]
fn coverage_is_retained_and_remote_upload_is_fail_closed_when_configured() {
    let release = workflow("release.yml");

    assert!(release.contains("uses: codecov/codecov-action@v7"));
    assert!(!release.contains("uses: codecov/codecov-action@v5"));
    let coverage = release
        .split("  coverage:\n")
        .nth(1)
        .and_then(|tail| tail.split("\n  build:\n").next())
        .expect("coverage job");
    assert!(!coverage.contains("    env:\n      CODECOV_TOKEN:"));
    assert!(coverage.contains("id: codecov-config"));
    assert!(coverage.contains("CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}"));
    assert!(coverage.contains("uses: actions/upload-artifact@v7"));
    assert!(coverage.contains("name: rust-lcov"));
    assert!(coverage.contains("path: lcov.info"));
    assert!(coverage.contains("if: steps.codecov-config.outputs.configured == 'true'"));
    assert!(coverage.contains("if: steps.codecov-config.outputs.configured != 'true'"));
    assert!(coverage.contains("          token: ${{ secrets.CODECOV_TOKEN }}"));
    assert!(coverage.contains("          fail_ci_if_error: true"));
    assert!(coverage.contains("          disable_search: true"));
    assert!(coverage.contains("          plugins: noop"));
    assert!(!coverage.contains("          use_oidc: true"));
    assert!(!coverage.contains("          fail_ci_if_error: false"));
}

#[test]
fn agent_cli_stderr_policy_accepts_only_reviewed_upstream_warnings() {
    let script = format!(
        "{}/scripts/classify-agent-cli-stderr.sh",
        env!("CARGO_MANIFEST_DIR")
    );
    let fixture = std::env::temp_dir().join(format!(
        "formal-ai-agent-stderr-{}-{}.log",
        std::process::id(),
        std::thread::current().name().unwrap_or("issue-717")
    ));
    fs::write(
        &fixture,
        concat!(
            "AI SDK Warning: System messages in the prompt or messages fields can be a security risk because they may enable prompt injection attacks. Use the system option instead when possible. Set allowSystemInMessages to true to suppress this warning, or false to throw an error.\n",
            "AI SDK Warning (opencode.chat / big-pickle): The feature \"specificationVersion\" is used in a compatibility mode. Using v2 specification compatibility mode. Some features may not be available.\n",
        ),
    )
    .unwrap();
    let accepted = Command::new(&script).arg(&fixture).output().unwrap();
    assert!(accepted.status.success());
    assert!(String::from_utf8_lossy(&accepted.stdout).contains("::notice"));

    fs::write(&fixture, "unexpected dependency diagnostic\n").unwrap();
    let rejected = Command::new(&script).arg(&fixture).output().unwrap();
    fs::remove_file(&fixture).unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("unexpected dependency diagnostic"));
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

#[test]
fn change_detection_covers_the_complete_pull_request() {
    let script = fs::read_to_string(format!(
        "{}/scripts/detect-code-changes.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(
        !script.contains("HEAD^2^ to HEAD^2 (per-commit diff of PR head)"),
        "per-commit detection can skip required tests for a multi-commit PR"
    );
    let release = workflow("release.yml");
    assert!(release.contains("GITHUB_EVENT_BEFORE: ${{ github.event.before }}"));
}

#[test]
fn generated_api_documentation_fails_on_every_rustdoc_warning() {
    let release = workflow("release.yml");
    let docs = release
        .split("      - name: Generate Rust API docs (cargo doc)\n")
        .nth(1)
        .and_then(|tail| {
            tail.split("      - name: Upload GitHub Pages artifact\n")
                .next()
        })
        .expect("cargo doc step");

    assert!(docs.contains("RUSTDOCFLAGS: -D warnings"));
    assert!(docs.contains("cargo doc --no-deps --lib"));
    assert!(!docs.contains("RUSTDOCFLAGS is unset"));
}
