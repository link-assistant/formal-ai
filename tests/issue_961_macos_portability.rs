//! Cross-platform source contracts for the four macOS portability regressions
//! reported in issue #961.

const PACKAGE_WRAPPER: &str = include_str!("../desktop/scripts/package-macos-with-retry.sh");
const SESSION_FILE_TEST: &str = include_str!("issue_757_session_files.rs");
const TUI_ISOLATION_TEST: &str = include_str!("integration/issue_819_tui_isolation.rs");
const PTY_HELPER: &str = include_str!("integration/pty.rs");
const WITH_FORMAL_AI_TEST: &str = include_str!("integration/with_formal_ai.rs");
const SYNC_SEED: &str = include_str!("../scripts/sync-seed.sh");
const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");

fn seed_array_guard_precedes_expansion() -> bool {
    let guard = SYNC_SEED.find("if [[ ${#dests[@]} -gt 0 ]]");
    let expansion = SYNC_SEED.find("for dst in \"${dests[@]}\"");
    matches!((guard, expansion), (Some(guard), Some(expansion)) if guard < expansion)
}

fn macos_portability_failures() -> Vec<&'static str> {
    let mut failures = Vec::new();

    if PACKAGE_WRAPPER.contains("formal-ai-macos-package.XXXXXX.log")
        || !PACKAGE_WRAPPER.contains("formal-ai-macos-package.log.XXXXXX")
    {
        failures.push("the package log mktemp placeholder must be the template suffix");
    }
    if !SESSION_FILE_TEST.contains("proxy_log.canonicalize()") {
        failures.push("the expected proxy log path must be canonicalized");
    }
    if TUI_ISOLATION_TEST.contains(".args([\"-qfec\"")
        || WITH_FORMAL_AI_TEST.contains(".args([\"-qfec\"")
        || !PTY_HELPER.contains("ScriptDialect::Bsd")
        || !PTY_HELPER.contains("command.args([\"-q\", \"/dev/null\", program])")
    {
        failures.push("PTY tests must select BSD script syntax on macOS");
    }
    if !seed_array_guard_precedes_expansion() {
        failures.push("the empty destination array must be guarded before expansion");
    }
    if !RELEASE_WORKFLOW.contains("os: [ubuntu-latest, macos-15-intel]") {
        failures.push("the full test matrix must include a supported macOS runner");
    }

    failures
}

#[test]
fn package_log_uses_a_bsd_portable_mktemp_template() {
    assert!(
        PACKAGE_WRAPPER.contains("formal-ai-macos-package.log.XXXXXX"),
        "BSD mktemp only replaces trailing X characters"
    );
    assert!(!PACKAGE_WRAPPER.contains("formal-ai-macos-package.XXXXXX.log"));
}

#[test]
fn proxy_log_expectation_matches_the_canonicalized_product_path() {
    assert!(SESSION_FILE_TEST.contains("proxy_log.canonicalize()"));
}

#[test]
fn pty_tests_do_not_embed_util_linux_only_script_flags() {
    assert!(!TUI_ISOLATION_TEST.contains(".args([\"-qfec\""));
    assert!(!WITH_FORMAL_AI_TEST.contains(".args([\"-qfec\""));
    assert!(PTY_HELPER.contains("ScriptDialect::Bsd"));
    assert!(PTY_HELPER.contains("command.args([\"-q\", \"/dev/null\", program])"));
}

#[test]
fn seed_sync_guards_an_empty_destination_array() {
    assert!(seed_array_guard_precedes_expansion());
}

#[test]
fn full_test_matrix_runs_on_a_supported_macos_image() {
    assert!(RELEASE_WORKFLOW.contains("os: [ubuntu-latest, macos-15-intel]"));
}

#[cfg(unix)]
#[test]
fn seed_sync_reaches_the_orphan_pass_with_an_empty_destination() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-961-sync-seed-{}-{nonce}",
        std::process::id()
    ));
    let script_dir = root.join("scripts");
    let source_dir = root.join("data/seed");
    let destination_dir = root.join("src/web/seed");
    std::fs::create_dir_all(&script_dir).expect("script directory");
    std::fs::create_dir_all(&source_dir).expect("seed source directory");
    std::fs::create_dir_all(&destination_dir).expect("empty seed destination directory");
    std::fs::write(source_dir.join("canary.lino"), "canary\n").expect("source canary");
    std::fs::write(script_dir.join("sync-seed.sh"), SYNC_SEED).expect("sandbox script");

    let output = std::process::Command::new("/bin/bash")
        .arg(script_dir.join("sync-seed.sh"))
        .arg("--check")
        .output()
        .expect("run seed sync with the platform Bash");
    let stderr = String::from_utf8_lossy(&output.stderr);
    std::fs::remove_dir_all(&root).expect("remove seed sync sandbox");

    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.contains("sync-seed: out of sync"),
        "stderr:\n{stderr}"
    );
    assert!(!stderr.contains("unbound variable"), "stderr:\n{stderr}");
}

#[test]
fn complete_macos_portability_contract_holds() {
    assert_eq!(macos_portability_failures(), Vec::<&str>::new());
}
