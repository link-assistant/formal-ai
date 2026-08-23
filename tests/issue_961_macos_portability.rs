//! Cross-platform source contracts for the four macOS portability regressions
//! reported in issue #961.

const PACKAGE_WRAPPER: &str = include_str!("../desktop/scripts/package-macos-with-retry.sh");
const SESSION_FILE_TEST: &str = include_str!("issue_757_session_files.rs");
const TUI_ISOLATION_TEST: &str = include_str!("integration/issue_819_tui_isolation.rs");
const PTY_HELPER: &str = include_str!("integration/pty.rs");
const WITH_FORMAL_AI_TEST: &str = include_str!("integration/with_formal_ai.rs");
const SYNC_SEED: &str = include_str!("../scripts/sync-seed.sh");
const RUNNER_DISK_CLEANUP: &str = include_str!("../scripts/free-runner-disk.sh");
const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");
const MACOS_CORE_WORKFLOW: &str = include_str!("../.github/workflows/macos-core-tests.yml");
const REQUIREMENTS: &str = include_str!("../REQUIREMENTS.md");
const ISSUE_CASE_STUDY: &str = include_str!("../docs/case-studies/issue-961/README.md");
const PR_CASE_STUDY: &str = include_str!("../docs/case-studies/pull-request-987/README.md");

fn seed_array_guard_precedes_expansion() -> bool {
    let guard = SYNC_SEED.find("if [[ ${#dests[@]} -gt 0 ]]");
    let expansion = SYNC_SEED.find("for dst in \"${dests[@]}\"");
    matches!((guard, expansion), (Some(guard), Some(expansion)) if guard < expansion)
}

/// The denominator the slice step actually passes to `cargo nextest
/// --partition`. Issue #961 froze it at twelve; issue #1017 raised it to
/// sixteen because the round-robin partitioner balances by test index and never
/// by duration. Reading it from the step instead of repeating a literal keeps
/// #961's contract -- every supported shard runs -- true at any slice count,
/// and still catches the failure it was written for: a matrix that disagrees
/// with the denominator silently drops those tests while CI stays green.
fn macos_core_slice_denominator() -> usize {
    MACOS_CORE_WORKFLOW
        .split("--partition \"slice:${{ matrix.partition }}/")
        .nth(1)
        .expect("the slice step must declare a denominator")
        .split('"')
        .next()
        .unwrap()
        .parse()
        .expect("numeric slice denominator")
}

fn supported_macos_test_shards_are_complete() -> bool {
    let slices = macos_core_slice_denominator();
    RELEASE_WORKFLOW.matches("os: macos-15-intel").count() == 1
        && RELEASE_WORKFLOW.contains("uses: ./.github/workflows/macos-core-tests.yml")
        && MACOS_CORE_WORKFLOW.matches("- { partition:").count() == slices
        && (1..=slices)
            .all(|shard| MACOS_CORE_WORKFLOW.contains(&format!("- {{ partition: {shard} }}")))
        && RELEASE_WORKFLOW.contains("test-suite: specification")
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
    if !supported_macos_test_shards_are_complete() {
        failures.push("the test matrix must include every supported macOS core slice");
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
fn pty_input_waits_for_tui_readiness_before_writing() {
    assert!(PTY_HELPER.contains("interact_after_ready"));
    assert!(WITH_FORMAL_AI_TEST.contains("pty::interact_after_ready"));
}

#[test]
fn seed_sync_guards_an_empty_destination_array() {
    assert!(seed_array_guard_precedes_expansion());
}

#[test]
fn full_test_matrix_runs_on_a_supported_macos_image() {
    assert!(supported_macos_test_shards_are_complete());
    // Issue #961 needed 25 minutes for the archive build and froze that literal;
    // issue #1017 raised it to 30 so the 1200s step budget expires before the
    // job cap rather than after it. The floor is the contract -- the archive
    // build must still be allowed at least the 25 minutes it was measured to
    // need -- so assert the floor instead of an exact value that a later
    // headroom increase would break again.
    let longest_cap = MACOS_CORE_WORKFLOW
        .split("timeout-minutes: ")
        .skip(1)
        .filter_map(|rest| rest.split('\n').next()?.trim().parse::<u64>().ok())
        .max()
        .expect("the macOS core workflow must declare job timeouts");
    assert!(
        longest_cap >= 25,
        "the archive build needs at least 25 minutes (issue #961); found {longest_cap}"
    );
    // Issue #1055: a floor, for the reason the comment above gives -- pinning
    // the exact number is what broke this when the budget grew from 1200s to
    // 1400s to absorb a cold rebuild after a profile change.
    let archive_budget = MACOS_CORE_WORKFLOW
        .split("TEST_BUDGET_SECONDS: ")
        .skip(1)
        .filter_map(|rest| rest.split('\n').next()?.trim().parse::<u64>().ok())
        .max()
        .expect("the macOS core workflow must budget its archive build");
    assert!(
        archive_budget >= 1200,
        "the archive build needs at least 1200s (issue #961); found {archive_budget}"
    );
    assert!(MACOS_CORE_WORKFLOW.contains("taiki-e/install-action@nextest"));
    assert_eq!(
        MACOS_CORE_WORKFLOW.matches("cargo nextest archive").count(),
        1
    );
}

#[test]
fn macos_test_bootstrap_uses_portable_df_syntax() {
    assert!(!RUNNER_DISK_CLEANUP.contains("--output=avail"));
    assert!(RUNNER_DISK_CLEANUP.contains("df -Pk /"));
}

#[cfg(unix)]
#[test]
fn runner_disk_cleanup_accepts_a_bsd_shaped_df() {
    use std::os::unix::fs::PermissionsExt as _;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-961-runner-disk-{}-{nonce}",
        std::process::id()
    ));
    let bin_dir = root.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("fake binary directory");
    let script = root.join("free-runner-disk.sh");
    std::fs::write(&script, RUNNER_DISK_CLEANUP).expect("sandbox cleanup script");

    for (name, body) in [
        (
            "df",
            "#!/bin/sh\ncase \"$1\" in\n  -Pk) printf 'Filesystem 1024-blocks Used Available Capacity Mounted on\\n/dev/mock 100000 50000 50000 50%% /\\n' ;;\n  -h) printf 'Filesystem Size Used Avail Capacity Mounted on\\n/dev/mock 100G 50G 50G 50%% /\\n' ;;\n  *) echo \"df: unrecognized option '$1'\" >&2; exit 64 ;;\nesac\n",
        ),
        ("sudo", "#!/bin/sh\nexit 0\n"),
    ] {
        let path = bin_dir.join(name);
        std::fs::write(&path, body).expect("fake command");
        let mut permissions = std::fs::metadata(&path)
            .expect("fake command metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("fake command permissions");
    }

    let mut paths = vec![bin_dir];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let output = std::process::Command::new("/bin/bash")
        .arg(&script)
        .env("PATH", std::env::join_paths(paths).expect("sandbox PATH"))
        .env("RUNNER_DISK_MIN_MIB", "0")
        .output()
        .expect("run cleanup against BSD-shaped df");
    let stderr = String::from_utf8_lossy(&output.stderr);
    std::fs::remove_dir_all(&root).expect("remove runner disk sandbox");

    assert!(output.status.success(), "stderr:\n{stderr}");
}

#[test]
fn requirement_matrix_and_case_studies_cover_the_complete_issue() {
    for requirement in ["R961-1", "R961-2", "R961-3", "R961-4", "R961-5", "R961-6"] {
        assert!(REQUIREMENTS.contains(requirement));
        assert!(ISSUE_CASE_STUDY.contains(requirement));
    }
    for section in [
        "## 1. Collected data",
        "## 2. Timeline",
        "## 3. Requirements",
        "## 4. Root causes",
        "## 5. Research and prior art",
        "## 6. Tests-first reproduction",
        "## 7. Implemented fix",
        "## 8. Formal AI / Agent CLI authorship",
        "## 9. Verification",
    ] {
        assert!(ISSUE_CASE_STUDY.contains(section));
    }
    assert!(PR_CASE_STUDY.contains("## Initial state and discussion audit"));
    assert!(PR_CASE_STUDY.contains("## Decisions made in this pull request"));
    assert!(PR_CASE_STUDY.contains("## Verification and CI"));
}

#[test]
fn formal_ai_and_the_real_agent_cli_authored_two_of_seven_smallest_leaves() {
    const FORMAL_AI_AUTHORED_LEAVES: usize = 2;
    const SMALLEST_REQUIREMENT_LEAVES: usize = 7;
    const GENERATED_CHANGELOG: &str = include_str!(
        "../docs/case-studies/issue-961/self-hosting-authorship/changelog-session/20260810_120000_issue_961_macos_ci_parity.md"
    );
    // Towncrier removes the canonical fragment after publishing it. Keep the
    // authorship check valid on release commits by verifying the generated
    // fragment against its durable canonical destination instead.
    const CANONICAL_CHANGELOG: &str = include_str!("../CHANGELOG.md");
    const GENERATED_DECOMPOSITION: &[u8] = include_bytes!(
        "../docs/case-studies/issue-961/self-hosting-authorship/decomposition-session/issue-961-task-decomposition.lino"
    );
    const CANONICAL_DECOMPOSITION: &[u8] =
        include_bytes!("../docs/case-studies/issue-961/issue-961-task-decomposition.lino");

    assert!(CANONICAL_CHANGELOG.contains(GENERATED_CHANGELOG.trim()));
    assert_eq!(GENERATED_DECOMPOSITION, CANONICAL_DECOMPOSITION);
    assert_eq!(
        FORMAL_AI_AUTHORED_LEAVES * 100 / SMALLEST_REQUIREMENT_LEAVES,
        28
    );

    for (agent_log, server_log, session, target) in [
        (
            include_str!(
                "../docs/case-studies/issue-961/self-hosting-authorship/changelog-session/agent-cli.log"
            ),
            include_str!(
                "../docs/case-studies/issue-961/self-hosting-authorship/changelog-session/formal-ai.log"
            ),
            "ses_014a75079ffewrf0nGhHCIG06f",
            "20260810_120000_issue_961_macos_ci_parity.md",
        ),
        (
            include_str!(
                "../docs/case-studies/issue-961/self-hosting-authorship/decomposition-session/agent-cli.log"
            ),
            include_str!(
                "../docs/case-studies/issue-961/self-hosting-authorship/decomposition-session/formal-ai.log"
            ),
            "ses_014a725dbffeJQOYwc4T0yJjFT",
            "issue-961-task-decomposition.lino",
        ),
    ] {
        assert!(agent_log.contains(session));
        for evidence in ["planned ToolCalls", "tool=write", "planned Final", target] {
            assert!(server_log.contains(evidence));
        }
    }
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
