//! Desktop Release workflow assertions, extracted from `workflow_release` when
//! that file crossed the 1000-line cap `scripts/check-file-size.rs` enforces
//! (PR #965 review: "All CI/CD warnings, and errors must be also fixed").
//! The `desktop-release` job is a self-contained surface -- it packages,
//! renames, checksums, and uploads artifacts for a *child* release -- so it
//! splits cleanly from the Pages/landing-page assertions left behind.
//! Shared helpers live in `workflow_fixtures`.

use std::fs;

use super::workflow_fixtures::*;

#[test]
fn desktop_release_does_not_archive_cargo_dependencies_after_packaging() {
    let workflow = desktop_release_workflow();
    let build = job_block(&workflow, "build");
    let install_sccache = workflow_step_block(build, "Cache Rust compiler outputs");
    let enable_sccache = workflow_step_block(build, "Enable Rust compiler cache");

    assert!(
        !build.contains("uses: actions/cache@"),
        "desktop builds must not register a post-job Cargo dependency archive: \
         a cold Windows x64 build already completed and uploaded every artifact, \
         then timed out compressing this redundant cache"
    );
    assert!(
        build.contains("mozilla-actions/sccache-action@"),
        "desktop builds should retain the compiler-output cache"
    );
    for step in [install_sccache, enable_sccache] {
        assert!(
            step.contains("if: runner.os != 'Windows'"),
            "desktop builds must bypass sccache on Windows: wrapping rustc makes \
             web-sys's generated feature-check command exceed CreateProcess's \
             command-line limit (os error 206)"
        );
    }
    assert!(
        install_sccache.contains("version: v0.16.0"),
        "desktop builds must pin the last known-good sccache version: v0.17.0 \
         expands Rust response files and exceeds the Windows command-line limit"
    );

    let shared_setup = fs::read_to_string(format!(
        "{}/.github/actions/setup-sccache/action.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read shared sccache setup action");
    assert!(
        shared_setup.contains("version: v0.16.0"),
        "the shared setup action must pin sccache instead of silently tracking its latest release"
    );
}

#[test]
fn desktop_release_workflow_run_resolves_child_release_not_head_sha_tag() {
    // Issue #479: the automated release tags a CHILD "chore: release vX.Y.Z"
    // commit (its first parent is the completed CI head SHA) and is pushed with
    // GITHUB_TOKEN, so GitHub never starts a CI run for it and suppresses the
    // `release` event. The previous resolve logic required a tag whose commit
    // EQUALS the workflow_run head SHA -- a match that could never happen -- so
    // the build was skipped and every /download entry read "Not available in
    // latest release".
    //
    // The corrected resolve logic lives in scripts/desktop-release-resolve.sh
    // (behaviorally covered by desktop_release_resolve.rs). This guard pins the
    // workflow wiring and the absence of the old, broken skip path.
    let workflow = desktop_release_workflow();
    let resolve = job_block(&workflow, "resolve");
    let pick = workflow_step_block(resolve, "Resolve tag and whether desktop assets are needed");
    let resolve_script = fs::read_to_string(format!(
        "{}/scripts/desktop-release-resolve.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n");

    assert!(
        pick.contains("WORKFLOW_RUN_HEAD_SHA: ${{ github.event.workflow_run.head_sha }}"),
        "workflow_run desktop builds should pass the completed CI run head SHA to the resolve script"
    );
    assert!(
        pick.contains("bash scripts/desktop-release-resolve.sh"),
        "the resolve step should delegate to the unit-tested resolve script"
    );
    assert!(
        resolve.contains("actions/checkout@v7"),
        "the resolve job must check out the repo so the resolve script is available"
    );

    // The old, broken behavior must not come back: never skip merely because no
    // tag points at the head SHA (the auto-release tag never does).
    assert!(
        !workflow.contains("No release tag points at workflow_run head SHA"),
        "issue #479 regression: must not skip when no tag matches the head SHA"
    );
    assert!(
        !resolve_script.contains("No release tag points at workflow_run head SHA"),
        "issue #479 regression: the resolve script must not reinstate the head-SHA skip"
    );

    // The corrected script must fall back to the latest release and keep the
    // self-healing idempotency guard.
    assert!(
        resolve_script.contains("latest_release_tag()"),
        "resolve script should fall back to the latest published release"
    );
    assert!(
        resolve_script.contains(r#"select(.commit.sha == \"$WORKFLOW_RUN_HEAD_SHA\")"#),
        "resolve script should keep the defensive exact-SHA tier"
    );
    assert!(
        resolve_script.contains("expected_desktop_assets()"),
        "resolve script should enumerate the complete required desktop asset set for the idempotency guard"
    );
    assert!(
        resolve_script.contains("missing required desktop assets"),
        "resolve script should report missing platform assets instead of treating a partial release as complete"
    );
    assert!(
        resolve_script.contains("already-has-all-assets"),
        "resolve script should skip only when every required desktop asset is present"
    );
    assert!(
        resolve_script.contains("::group::"),
        "resolve script should emit grouped verbose diagnostics for future debugging"
    );
}

#[test]
fn desktop_release_normalizes_linux_artifact_names_before_checksums() {
    // Electron Builder emits Linux x64 artifacts as x86_64 AppImage and amd64
    // .deb files. Normalize before checksums/upload so the release assets match
    // src/web/download and scripts/desktop-release-resolve.sh.
    let workflow = desktop_release_workflow();
    let build = job_block(&workflow, "build");
    let normalize = workflow_step_block(build, "Normalize desktop artifact names");
    let normalizer = fs::read_to_string(format!(
        "{}/desktop/scripts/normalize-artifacts.mjs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n");

    let package_pos = build
        .find("- name: Package desktop app")
        .expect("desktop build should package the app");
    let normalize_pos = build
        .find("- name: Normalize desktop artifact names")
        .expect("desktop build should normalize Electron Builder artifact aliases");
    let collect_pos = build
        .find("- name: Collect artifacts and checksums")
        .expect("desktop build should collect checksums");
    let upload_pos = build
        .find("- name: Upload assets to release")
        .expect("desktop build should upload release assets");

    assert!(
        package_pos < normalize_pos && normalize_pos < collect_pos && collect_pos < upload_pos,
        "desktop artifacts must be normalized after packaging but before checksum generation and release upload"
    );
    assert!(
        normalize.contains("working-directory: desktop")
            && normalize.contains("node scripts/normalize-artifacts.mjs"),
        "desktop workflow should run the normalizer from the desktop directory"
    );
    assert!(
        normalizer.contains("linux-x86_64")
            && normalizer.contains("linux-amd64")
            && normalizer.contains("linux-x64")
            && normalizer.contains("latest(?:-mac|-linux)?\\.yml"),
        "normalizer should map Electron Builder Linux x64 aliases to the x64 download and updater contracts"
    );
}

#[test]
fn desktop_release_uploads_auto_update_metadata() {
    let workflow = desktop_release_workflow();
    let build = job_block(&workflow, "build");
    let collect = workflow_step_block(build, "Collect artifacts and checksums");
    let upload = workflow_step_block(build, "Upload assets to release");
    let resolve_script = fs::read_to_string(format!(
        "{}/scripts/desktop-release-resolve.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    for step in [collect, upload] {
        assert!(
            step.contains("*.blockmap") && step.contains("release/latest.yml"),
            "desktop release should collect/upload updater blockmaps and latest.yml metadata"
        );
        assert!(
            !step.contains("*.blockmap|*.yml) continue"),
            "issue #548 regression: updater metadata must not be filtered out of release uploads"
        );
    }
    assert!(
        collect.contains("latest(-mac|-linux)?\\.yml"),
        "checksum fragments should include updater metadata for provenance"
    );
    assert!(
        resolve_script.contains("latest.yml")
            && resolve_script.contains("latest-mac.yml")
            && resolve_script.contains("latest-linux.yml")
            && resolve_script.contains("required desktop assets: 17"),
        "release resolver should require update metadata before skipping an automatic build"
    );
}

#[test]
fn desktop_release_runs_on_any_completed_main_pipeline_not_only_success() {
    // Issue #479 (root cause, take 2): PR #480 fixed the resolve *script* but
    // left the resolve *job* gated behind `workflow_run.conclusion == 'success'`.
    // The auto-release publishes the GitHub release in an EARLY pipeline job, so
    // any LATER job failing (e.g. the E2E Pages probe timing out) made the whole
    // pipeline conclude `failure`, the gate skipped, and no desktop assets were
    // ever built -- the fix stayed dormant and /download still read "Not
    // available in latest release". The gate must run on ANY completed main-branch
    // pipeline except cancelled/skipped, delegating the real build decision to the
    // self-healing resolve script + its idempotency guard.
    let workflow = desktop_release_workflow();
    let resolve = job_block(&workflow, "resolve");

    assert!(
        resolve.contains("github.event.workflow_run.head_branch == 'main'"),
        "desktop release should still only auto-build for main-branch pipelines"
    );
    assert!(
        resolve.contains("github.event.workflow_run.conclusion != 'cancelled'")
            && resolve.contains("github.event.workflow_run.conclusion != 'skipped'"),
        "desktop release should run on any completed main pipeline except cancelled/skipped"
    );
    assert!(
        !resolve.contains("github.event.workflow_run.conclusion == 'success'"),
        "issue #479 regression: desktop release must NOT gate on full-pipeline success -- a late \
         unrelated failure (e.g. E2E Pages timeout) would again suppress every desktop build"
    );
}
