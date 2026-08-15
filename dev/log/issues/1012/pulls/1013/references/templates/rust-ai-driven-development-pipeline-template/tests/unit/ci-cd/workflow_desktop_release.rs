use std::fs;

fn workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/desktop-release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("desktop-release workflow should exist")
    .replace("\r\n", "\n")
}

#[test]
fn desktop_release_is_opt_in_and_runs_a_six_target_dry_run() {
    let workflow = workflow();
    assert!(workflow.contains("vars.DESKTOP_RELEASE_ENABLED == 'true'"));
    assert!(workflow.contains("pull_request:"));
    assert!(workflow.contains("fail-fast: false"));
    for target in [
        "linux-x64",
        "linux-arm64",
        "macos-x64",
        "macos-arm64",
        "windows-x64",
        "windows-arm64",
    ] {
        assert!(workflow.contains(target), "missing target {target}");
    }
    assert!(workflow.contains("scripts/package-desktop.sh"));
    assert!(workflow.contains("github.event_name != 'pull_request'"));
}

#[test]
fn published_assets_are_verified_attested_and_finalized() {
    let workflow = workflow();
    assert!(workflow.contains("find \"$OUTPUT_DIR\" -type f -size +0c"));
    assert!(
        !workflow.contains("mapfile -t assets"),
        "macOS ships Bash 3, which does not provide mapfile"
    );
    assert!(workflow.contains("actions/attest@v4"));
    assert!(workflow.contains("SHA256SUMS.txt"));
    assert!(workflow.contains("BUILD-PROVENANCE.txt"));
    assert!(workflow.contains("gh release upload"));
}

#[test]
fn download_page_uses_the_github_releases_api() {
    let page = fs::read_to_string(format!(
        "{}/docs/download/index.html",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("download page should exist");
    assert!(page.contains("api.github.com/repos/"));
    assert!(page.contains("/releases/latest"));
    assert!(page.contains("SHA256SUMS.txt"));
}
