//! Regression coverage for issue #796.
//!
//! `scripts/install-node-dependencies.sh` classified reviewed npm deprecation
//! warnings by exact `name@version`. Transitive dependencies float without any
//! change on our side, so when `archiver-utils` resolved `glob` from `7.2.3` to
//! `10.5.0` the warning stopped matching, was treated as an unexpected
//! diagnostic, and failed both the `.vsix` packaging job and every desktop
//! build in the Desktop Release workflow.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn script() -> String {
    format!(
        "{}/scripts/install-node-dependencies.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Builds a sandbox holding a fake `npm` that emits `stderr_lines`, so the
/// classifier can be exercised without touching the network.
fn sandbox(stderr_lines: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-npm-classifier-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("sandbox must be created");
    let npm = dir.join("npm");
    fs::write(
        &npm,
        format!(
            "#!/usr/bin/env bash\n\
             if [ \"$1\" = \"--version\" ]; then echo 10.9.8; exit 0; fi\n\
             cat >&2 <<'STDERR'\n{stderr_lines}\nSTDERR\n\
             echo 'added 1 package'\nexit 0\n"
        ),
    )
    .expect("fake npm must be written");
    fs::set_permissions(&npm, fs::Permissions::from_mode(0o755))
        .expect("fake npm must be executable");
    dir
}

fn run_classifier(dir: &Path) -> std::process::Output {
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new("bash")
        .arg(script())
        .arg("ignored-directory")
        .env("PATH", path)
        .output()
        .expect("classifier must run")
}

#[test]
fn reviewed_deprecations_are_matched_by_package_name_not_pinned_version() {
    // The exact line that failed CI in run 29681451142.
    let dir = sandbox(
        "npm warn deprecated glob@10.5.0: Old versions of glob are not supported, and contain \
         widely publicized security vulnerabilities, which have been fixed in the current version.",
    );
    let output = run_classifier(&dir);
    fs::remove_dir_all(&dir).expect("sandbox must be removed");

    assert!(
        output.status.success(),
        "a floated version of a reviewed deprecation must not fail CI; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("::notice title="),
        "the warning must be surfaced as an annotation rather than swallowed"
    );
}

#[test]
fn unreviewed_diagnostics_still_fail_the_build() {
    let dir = sandbox("npm warn deprecated totally-unknown-package@1.0.0: not reviewed");
    let output = run_classifier(&dir);
    fs::remove_dir_all(&dir).expect("sandbox must be removed");

    assert!(
        !output.status.success(),
        "loosening the version match must not turn the classifier into a rubber stamp"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unexpected npm stderr"));
}

#[test]
fn scoped_package_names_are_parsed_without_losing_the_scope() {
    let dir = sandbox("npm warn deprecated @scope/pkg@2.0.0: scoped and unreviewed");
    let output = run_classifier(&dir);
    fs::remove_dir_all(&dir).expect("sandbox must be removed");

    // Reported verbatim: the scope must survive name extraction so that adding
    // "@scope/pkg" to the allowlist is what silences it.
    assert!(String::from_utf8_lossy(&output.stderr).contains("@scope/pkg@2.0.0"));
}

#[test]
fn verbose_tracing_exists_and_is_off_by_default() {
    let dir = sandbox("npm warn deprecated glob@10.5.0: reviewed");

    let quiet = run_classifier(&dir);
    assert!(
        !String::from_utf8_lossy(&quiet.stderr).contains("install-node-dependencies:"),
        "diagnostic tracing must stay off unless explicitly enabled"
    );

    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let verbose = Command::new("bash")
        .arg(script())
        .arg("ignored-directory")
        .env("PATH", path)
        .env("INSTALL_NODE_DEPENDENCIES_VERBOSE", "1")
        .output()
        .expect("classifier must run");
    fs::remove_dir_all(&dir).expect("sandbox must be removed");

    assert!(
        String::from_utf8_lossy(&verbose.stderr).contains("classified 'glob'"),
        "verbose mode must explain each classification decision"
    );
}

#[test]
fn the_classifier_does_not_pin_versions_in_its_allowlist() {
    let contents = fs::read_to_string(script()).expect("npm classifier must be readable");
    let allowlist = contents
        .split("reviewed_deprecations=(")
        .nth(1)
        .and_then(|tail| tail.split(')').next())
        .expect("allowlist must be present");
    assert!(
        !allowlist.contains('@'),
        "allowlist entries must name packages only; pinning a version reintroduces issue #796"
    );
}
