//! Regression coverage for the failures and warning-policy gaps in issue #798.

use std::fs;

use super::workflow_fixtures::desktop_release_workflow;

fn read(path: &str) -> String {
    fs::read_to_string(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path))
        .unwrap_or_else(|error| panic!("{path} must be readable: {error}"))
}

#[test]
fn direct_wasm_rustc_build_rejects_warnings() {
    let build = read("src/web/wasm-worker/build.sh");

    assert!(
        build.contains("-D warnings"),
        "RUSTFLAGS does not affect a direct rustc invocation; the worker build must deny warnings explicitly"
    );
}

#[test]
fn desktop_release_rejects_rust_warnings_on_every_platform() {
    let workflow = desktop_release_workflow();
    let env = workflow
        .split_once("\nenv:\n")
        .map(|(_, rest)| rest.split_once("\njobs:\n").map_or(rest, |(env, _)| env))
        .expect("desktop release workflow must have a global env block");

    assert!(
        env.contains("RUSTFLAGS: -Dwarnings"),
        "desktop cargo builds must not silently accept compiler warnings"
    );
}

#[test]
fn wasm_only_partial_modules_document_their_intentional_dead_code() {
    let worker = read("src/web/wasm-worker/src/lib.rs");

    for module in ["language", "arithmetic", "web_engine_core"] {
        let marker = format!("#[allow(dead_code)]\nmod {module};");
        assert!(
            worker.contains(&marker),
            "the partially reused `{module}` module must scope its intentional wasm-only dead-code allowance"
        );
    }
}

#[test]
fn platform_specific_shared_memory_state_is_platform_gated() {
    let shared_memory = read("src/shared_memory.rs");

    assert!(
        shared_memory.contains("#[cfg(unix)]\n        let existed = parent.exists();"),
        "the state used only by the Unix permissions branch must not warn on Windows"
    );
}

#[test]
fn pull_request_ci_runs_the_macos_signing_regression() {
    let workflow = read(".github/workflows/release.yml");

    assert!(
        workflow.contains("node --test desktop/scripts/adhoc-sign-mac.test.cjs"),
        "the macOS signer regression must run before changes reach the release-only workflow"
    );
}

#[test]
fn pull_request_ci_rejects_an_unsynchronized_cargo_lock() {
    let workflow = read(".github/workflows/release.yml");

    assert!(
        workflow.contains("cargo metadata --locked --format-version 1"),
        "binary releases must be built only from a synchronized Cargo.lock"
    );
}

#[test]
fn parallel_e2e_jobs_do_not_race_to_save_the_bun_cache() {
    let workflow = read(".github/workflows/release.yml");

    assert_eq!(
        workflow.matches("no-cache: true").count(),
        2,
        "both parallel E2E jobs must leave the shared setup-bun cache write to lint"
    );
}

#[test]
fn both_release_paths_smoke_test_the_registry_artifact() {
    let workflow = read(".github/workflows/release.yml");
    let invocation = "scripts/smoke-test-published-crate.sh";

    assert_eq!(
        workflow.matches(invocation).count(),
        2,
        "automatic and manual releases must both execute the published crate"
    );

    let smoke = read("scripts/smoke-test-published-crate.sh");
    for contract in [
        "cargo install formal-ai",
        "--version \"=${version}\"",
        "--locked",
        "--help",
    ] {
        assert!(
            smoke.contains(contract),
            "published-crate smoke test must contain `{contract}`"
        );
    }
}
