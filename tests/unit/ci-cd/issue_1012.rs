//! Regression coverage for issue #1012's full CI warning/error audit.

use std::fs;
use std::process::Command;

use super::workflow_fixtures::{job_block, release_workflow};

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
        .replace("\r\n", "\n")
}

fn action_step<'a>(workflow: &'a str, action: &str) -> Vec<&'a str> {
    workflow
        .split("\n      - ")
        .filter(|step| step.contains(&format!("uses: {action}")))
        .collect()
}

#[test]
fn macos_core_tests_are_sliced_and_warn_before_the_job_timeout() {
    let workflow = release_workflow();
    let test = job_block(&workflow, "test");
    let macos_call = job_block(&workflow, "macos-core-tests");
    let macos = repository_file(".github/workflows/macos-core-tests.yml");

    assert_eq!(test.matches("os: macos-15-intel").count(), 1);
    assert!(test.contains("test-suite: specification"));
    assert!(macos_call.contains("uses: ./.github/workflows/macos-core-tests.yml"));
    for shard in 1..=3 {
        assert!(
            macos.contains(&format!("- {{ partition: {shard} }}")),
            "missing macOS core shard {shard}"
        );
    }
    assert_eq!(macos.matches("cargo nextest archive").count(), 1);
    assert!(macos.contains("--partition \"slice:${{ matrix.partition }}/3\""));
    assert!(macos.contains("timeout-minutes: 25"));
    assert!(macos.contains("timeout-minutes: 10"));
    assert!(!macos.contains("2100"));
    assert!(macos.contains("taiki-e/install-action@nextest"));
    assert!(macos.contains("scripts/run-with-budget-warning.sh"));
}

#[test]
fn budget_warning_is_live_instead_of_post_process() {
    let script = format!(
        "{}/scripts/run-with-budget-warning.sh",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new("bash")
        .arg(script)
        .args(["1", "Slow regression test", "bash", "-c", "sleep 2"])
        .output()
        .expect("run budget warning wrapper");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("::warning title=Slow regression test is approaching its timeout::"),
        "warning should be emitted while the wrapped command is still alive: {stderr}"
    );
}

#[test]
fn download_artifact_deprecation_is_suppressed_only_on_affected_steps() {
    for path in [
        ".github/workflows/agentic-cli-matrix.yml",
        ".github/workflows/desktop-release.yml",
        ".github/workflows/macos-core-tests.yml",
        ".github/workflows/release.yml",
    ] {
        let workflow = repository_file(path);
        let steps = action_step(&workflow, "actions/download-artifact@v8");
        assert!(
            !steps.is_empty(),
            "{path} should exercise download-artifact v8"
        );
        for step in steps {
            assert!(
                step.contains("NODE_OPTIONS: --disable-warning=DEP0005"),
                "{path} has an unsuppressed download-artifact v8 step:\n{step}"
            );
        }
    }
}

#[test]
fn agentic_matrix_consumes_search_output_and_tolerates_closed_tui_input() {
    let library = repository_file("experiments/agentic_cli_matrix/lib.sh");

    for old in ["grep -qF", "grep -qiF", "grep -qiE"] {
        assert!(
            !library.contains(old),
            "{old} closes the process substitution early and makes sed report a broken pipe"
        );
    }
    for search in ["grep -F", "grep -iF", "grep -iE"] {
        assert!(library.contains(&format!("{search} -- \"$2\" > /dev/null < <(")));
    }
    assert!(library.contains("printf '\\r' 2> /dev/null || return 0"));
    assert!(library.contains("printf '%b' \"${step#*:}\" 2> /dev/null || return 0"));
}

#[test]
fn third_party_cli_noise_has_narrow_documented_mitigations() {
    let installer = repository_file("experiments/agentic_cli_matrix/install_client.sh");
    assert!(installer.contains("NODE_OPTIONS=--disable-warning=DEP0169"));
    assert!(installer.contains("microsoft/vscode/issues/319867"));

    let gemini = repository_file("experiments/agent_cli_e2e/run_issue_907.sh");
    assert!(gemini.contains("\"model\":{\"name\":\"formal-ai\"}"));
    assert!(gemini.contains("\"tools\":{\"useRipgrep\":false}"));
    assert!(gemini.contains("export TERM=xterm-256color"));
}

#[test]
fn stock_rust_workflow_uses_a_quiet_dependency_probe_and_installed_path() {
    let workflow = repository_file(".github/workflows/stock-rust-install.yml");

    assert!(!workflow.contains("cargo tree --locked -i openssl-sys"));
    assert!(workflow.contains("cargo tree --locked --prefix none --format '{p}'"));
    assert!(workflow.contains("> /tmp/formal-ai-dependency-tree.txt"));
    assert!(!workflow.contains("<<<"));
    assert!(workflow.contains("CARGO_INSTALL_ROOT: /tmp/formal-ai-install"));
    assert!(workflow.contains("export PATH=\"$CARGO_INSTALL_ROOT/bin:$PATH\""));
    assert!(workflow.contains("formal-ai --version"));
}

#[test]
fn box_language_matrix_reuses_one_release_binary() {
    let workflow = release_workflow();
    let build = job_block(&workflow, "build-box-language-binary");
    let matrix = job_block(&workflow, "box-language-projects");

    assert!(build.contains("cargo build --release --bin formal-ai"));
    assert!(build.contains("actions/upload-artifact@v7"));
    assert!(matrix.contains("needs: [detect-changes, build-box-language-binary]"));
    assert!(matrix.contains("actions/download-artifact@v8"));
    assert!(!matrix.contains("actions/cache@v5"));
    assert!(!matrix.contains("cargo build --release --bin formal-ai"));
}

#[test]
fn sccache_backend_diagnostics_are_available_but_off_by_default() {
    let workflow = release_workflow();
    assert!(workflow.contains("FORMAL_AI_CI_VERBOSE: ${{ vars.FORMAL_AI_CI_VERBOSE || 'false' }}"));

    let setup = repository_file(".github/actions/setup-sccache/action.yml");
    assert!(setup.contains("if: env.FORMAL_AI_CI_VERBOSE == 'true'"));
    assert!(setup.contains("SCCACHE_LOG=debug"));
    assert!(setup.contains("SCCACHE_LOG_MILLIS=1"));
}

#[test]
fn audited_warning_band_sources_stay_below_their_limits() {
    let solver_lines = repository_file("src/solver.rs").lines().count();
    assert!(
        solver_lines <= 900,
        "src/solver.rs has {solver_lines} lines; keep it below the 900-line warning band"
    );
    let workflow_lines = release_workflow().lines().count();
    assert!(
        workflow_lines <= 1_500,
        ".github/workflows/release.yml has {workflow_lines} lines"
    );
}

#[test]
fn solver_configuration_keeps_every_registered_response_language() {
    let registered_languages = formal_ai::language::registered_languages();
    assert!(
        !registered_languages.is_empty(),
        "the response-language registry must not be empty"
    );

    for language in registered_languages {
        let config = formal_ai::SolverConfig {
            forced_response_language: Some(language.slug()),
            ..formal_ai::SolverConfig::default()
        };
        assert_eq!(config.forced_response_language, Some(language.slug()));
    }
}

#[test]
fn formal_ai_and_real_agent_cli_authored_two_of_nine_requirement_leaves() {
    const FORMAL_AI_AUTHORED_LEAVES: usize = 2;
    const SMALLEST_REQUIREMENT_LEAVES: usize = 9;
    let cases = [
        (
            "Create file ci-diagnostic-audit-invariant.md containing CI diagnostics are classified, actionable causes are fixed, and uncertain cache backends expose opt-in tracing",
            "CI diagnostics are classified, actionable causes are fixed, and uncertain cache backends expose opt-in tracing",
            "ci-diagnostic-audit-invariant.md",
            "ses_ffb20640bffeX0VahcuKR9bfXj",
            "diagnostic-audit",
        ),
        (
            "Create file ci-template-comparison-invariant.md containing Template audits compare immutable workflow and script trees and report shared defects upstream",
            "Template audits compare immutable workflow and script trees and report shared defects upstream",
            "ci-template-comparison-invariant.md",
            "ses_ffb1ee9b4ffedIt26xRyh8FEDy",
            "template-comparison",
        ),
    ];

    const {
        assert!(FORMAL_AI_AUTHORED_LEAVES * 100 / SMALLEST_REQUIREMENT_LEAVES >= 20);
    }
    for (task, expected_leaf, target, native_session, evidence_dir) in cases {
        let base = format!("docs/case-studies/issue-1012/self-hosting-authorship/{evidence_dir}");
        assert_eq!(repository_file(&format!("{base}/{target}")), expected_leaf);
        assert!(repository_file(&format!("{base}/agent-cli.log")).contains(native_session));

        let server_log = repository_file(&format!("{base}/formal-ai.log"));
        for marker in ["planned ToolCalls", "tool=write", "planned Final", target] {
            assert!(
                server_log.contains(marker),
                "missing {marker:?} from {evidence_dir} server trace"
            );
        }

        let fresh = formal_ai::agentic_coding::run_agentic_task(task).expect("replay agent task");
        let rendered = format!(
            "{}\n",
            serde_json::to_string_pretty(&fresh.session_json()).expect("render session JSON")
        );
        assert_eq!(repository_file(&format!("{base}/session.json")), rendered);

        let written_leaf = fresh
            .steps
            .iter()
            .find(|step| {
                step.tool == "write_file"
                    && serde_json::from_str::<serde_json::Value>(&step.arguments)
                        .is_ok_and(|arguments| arguments["path"] == target)
            })
            .expect("Agent CLI writes the claimed leaf");
        let arguments: serde_json::Value =
            serde_json::from_str(&written_leaf.arguments).expect("write arguments JSON");
        assert_eq!(arguments["content"], expected_leaf);
    }
}
