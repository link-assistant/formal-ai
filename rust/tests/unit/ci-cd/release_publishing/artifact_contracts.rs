use std::fs;
use std::process::Command;

pub(super) fn desktop_linux_artifact_names_are_normalized_for_download_contract() {
    // Electron Builder renders Linux x64 artifacts with target-specific aliases:
    // AppImage uses x86_64 and Debian uses amd64. The download page, checksum
    // tests, and release resolver all use the cross-platform x64 contract, so
    // the release workflow normalizes the files before checksums and upload.
    match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {}
        _ => {
            eprintln!("skipping: node not available");
            return;
        }
    }

    let manifest_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let script = format!("{manifest_dir}/desktop/scripts/normalize-artifacts.mjs");
    let tmp = std::env::temp_dir().join(format!(
        "formal-ai-desktop-artifacts-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    ));
    fs::create_dir_all(&tmp).expect("create scratch release dir");

    let source_names = [
        "formal-ai-desktop-linux-x86_64-9.9.9.AppImage",
        "formal-ai-desktop-linux-x86_64-9.9.9.AppImage.blockmap",
        "formal-ai-desktop-linux-amd64-9.9.9.deb",
        "formal-ai-desktop-linux-x64-9.9.9.tar.gz",
        "formal-ai-desktop-linux-arm64-9.9.9.deb",
    ];
    for name in source_names {
        fs::write(tmp.join(name), b"artifact").expect("seed scratch release artifact");
    }
    fs::write(
        tmp.join("latest-linux.yml"),
        "files:\n  - url: formal-ai-desktop-linux-x86_64-9.9.9.AppImage\n  - url: formal-ai-desktop-linux-amd64-9.9.9.deb\npath: formal-ai-desktop-linux-x86_64-9.9.9.AppImage\n",
    )
    .expect("seed scratch update metadata");

    let output = Command::new("node")
        .arg(&script)
        .arg(&tmp)
        .output()
        .expect("run desktop artifact normalizer");
    let status_ok = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let expected_names = [
        "formal-ai-desktop-linux-x64-9.9.9.AppImage",
        "formal-ai-desktop-linux-x64-9.9.9.AppImage.blockmap",
        "formal-ai-desktop-linux-x64-9.9.9.deb",
        "formal-ai-desktop-linux-x64-9.9.9.tar.gz",
        "formal-ai-desktop-linux-arm64-9.9.9.deb",
    ];
    let missing = expected_names
        .iter()
        .filter(|name| !tmp.join(name).exists())
        .copied()
        .collect::<Vec<_>>();
    let stale = [
        "formal-ai-desktop-linux-x86_64-9.9.9.AppImage",
        "formal-ai-desktop-linux-x86_64-9.9.9.AppImage.blockmap",
        "formal-ai-desktop-linux-amd64-9.9.9.deb",
    ]
    .iter()
    .filter(|name| tmp.join(name).exists())
    .copied()
    .collect::<Vec<_>>();
    let metadata =
        fs::read_to_string(tmp.join("latest-linux.yml")).expect("read normalized update metadata");
    let _ = fs::remove_dir_all(&tmp);

    assert!(
        status_ok,
        "normalizer exited with {status:?}\nstdout: {stdout}\nstderr: {stderr}",
        status = output.status
    );
    assert!(
        missing.is_empty(),
        "normalizer should create every download-contract artifact, missing {missing:?}\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stale.is_empty(),
        "normalizer should remove Electron Builder alias names, still present {stale:?}\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("x86_64") && stdout.contains("amd64") && stdout.contains("x64"),
        "normalizer should log the Linux x64 alias rewrites, got stdout:\n{stdout}"
    );
    assert!(
        metadata.contains("formal-ai-desktop-linux-x64-9.9.9.AppImage")
            && metadata.contains("formal-ai-desktop-linux-x64-9.9.9.deb")
            && !metadata.contains("linux-x86_64")
            && !metadata.contains("linux-amd64"),
        "normalizer should rewrite latest-linux.yml to the uploaded x64 asset names, got:\n{metadata}"
    );
}

pub(super) fn release_scripts_check_configured_release_artifacts() {
    let release_check = fs::read_to_string(format!(
        "{}/../scripts/check-release-needed.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let wait_for_crate = fs::read_to_string(format!(
        "{}/../scripts/wait-for-crate.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let release_script = fs::read_to_string(format!(
        "{}/../scripts/create-github-release.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(
        release_check.contains("check_docker_hub_tag"),
        "release-needed check should verify configured Docker Hub tags"
    );
    assert!(
        release_check.contains("check_docker_hub_tag(image, \"latest\")"),
        "release-needed check should verify Docker Hub latest tags as part of completeness"
    );
    assert!(
        release_check.contains("check_github_release"),
        "release-needed check should verify GitHub release artifacts"
    );
    assert!(
        release_check.contains("crate_published"),
        "release-needed check should output whether the crate already exists"
    );
    assert!(
        wait_for_crate.contains("crates.io/api/v1/crates"),
        "release workflow should wait for crates.io visibility before image publishing"
    );
    assert!(
        wait_for_crate.contains("example-sum-package-name")
            && wait_for_crate.contains("crate_available\", \"skipped\""),
        "crate availability wait should preserve template-safe publishing skips"
    );
    assert!(
        release_script.contains("--ghcr-url"),
        "GitHub release creation should accept a GHCR package URL"
    );
    assert!(
        release_script.contains("fn ghcr_badge"),
        "GitHub release notes should include GHCR badge support"
    );
    assert!(
        release_script.contains("--docker-hub-url"),
        "GitHub release creation should accept a Docker Hub URL"
    );
    assert!(
        release_script.contains("fn docker_hub_badge"),
        "GitHub release notes should include Docker Hub badge support"
    );
}
