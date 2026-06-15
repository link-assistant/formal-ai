//! Release artifact publishing (crate, Docker, electron) + Pages asset
//! version-stamping / cache-busting assertions. Shared helpers live in
//! `workflow_fixtures`.

use std::fs;
use std::process::Command;

use super::workflow_fixtures::*;

#[test]
fn github_pages_artifact_advertises_crate_version_from_cargo_toml() {
    // Issue #72: the deployed Pages site advertised `0.16.0` long after the
    // crate moved past it. The fix replaces a hardcoded literal with the
    // `__FORMAL_AI_VERSION__` placeholder and a stamp step that reads the
    // current `Cargo.toml` version during the Pages deploy. Without these
    // pieces the deploy can drift again.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index_html = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    let app_js = fs::read_to_string(format!("{manifest_dir}/src/web/app.js")).unwrap();
    let stamp_script =
        fs::read_to_string(format!("{manifest_dir}/scripts/stamp-pages-artifact.sh")).unwrap();
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");

    assert!(
        index_html.contains("__FORMAL_AI_VERSION__"),
        "index.html should advertise the formal-ai version via a placeholder, not a hardcoded literal"
    );
    assert!(
        !index_html.contains("content=\"0.16.0\""),
        "index.html should not pin the stale 0.16.0 version literal"
    );
    assert!(
        app_js.contains("formal-ai-version"),
        "app.js should still read the formal-ai-version meta tag"
    );
    assert!(
        !app_js.contains("\"0.16.0\"") && !app_js.contains("'0.16.0'"),
        "app.js should not hardcode the stale 0.16.0 version as a string literal"
    );
    assert!(
        stamp_script.contains("__FORMAL_AI_VERSION__"),
        "stamp script should substitute the formal-ai version placeholder"
    );
    assert!(
        stamp_script.contains("formal-ai-version"),
        "stamp script should validate the rendered meta tag content"
    );
    assert!(
        stamp_script.contains("formal_ai_version"),
        "stamp deployment.json should carry the formal-ai version for the e2e wait script"
    );
    assert!(
        deploy_demo.contains("Read formal-ai version from Cargo.toml"),
        "deploy-demo should detect the crate version before stamping the artifact"
    );
    assert!(
        deploy_demo.contains(
            "scripts/stamp-pages-artifact.sh src/web \"${{ github.sha }}\" \"${{ github.sha }}\" \"${{ steps.formal_ai_version.outputs.version }}\""
        ),
        "deploy-demo should forward the resolved crate version to the stamp script"
    );

    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();
    assert!(
        wait_script.contains("__FORMAL_AI_VERSION__"),
        "pages-deployment wait script should reject lingering version placeholders"
    );
}

#[test]
fn stamp_pages_artifact_replaces_formal_ai_version_placeholder() {
    // Issue #72: end-to-end smoke test for the stamp script. Copy
    // `src/web/index.html` into a scratch directory, run the script the
    // same way CI does, and assert the rendered file advertises the
    // requested formal-ai version. Catches regressions in either the
    // placeholder or the substitution logic.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script = format!("{manifest_dir}/scripts/stamp-pages-artifact.sh");
    if !std::path::Path::new("/bin/bash").exists() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let tmp = std::env::temp_dir().join(format!(
        "formal-ai-stamp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    ));
    let web_dir = tmp.join("web");
    fs::create_dir_all(&web_dir).expect("create scratch web dir");
    fs::create_dir_all(web_dir.join("tests")).expect("create scratch tests dir");
    let source_index = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    fs::write(web_dir.join("index.html"), &source_index).expect("seed index.html");
    fs::write(
        web_dir.join("tests/index.html"),
        r#"<meta name="formal-ai-version" content="__FORMAL_AI_VERSION__"><script src="connectivity.js?v=__FORMAL_AI_ASSET_VERSION__"></script>"#,
    )
    .expect("seed tests/index.html");

    let output = Command::new("/bin/bash")
        .arg(&script)
        .arg(web_dir.to_str().unwrap())
        .arg("deadbeef")
        .arg("deadbeef")
        .arg("9.9.9")
        .output()
        .expect("run stamp script");
    let status_ok = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let rendered = fs::read_to_string(web_dir.join("index.html")).unwrap_or_default();
    let rendered_tests = fs::read_to_string(web_dir.join("tests/index.html")).unwrap_or_default();
    let marker = fs::read_to_string(web_dir.join("deployment.json")).unwrap_or_default();
    let _ = fs::remove_dir_all(&tmp);

    assert!(
        status_ok,
        "stamp script exited with {status:?}\nstdout: {stdout}\nstderr: {stderr}",
        status = output.status
    );
    assert!(
        rendered.contains("content=\"9.9.9\""),
        "stamped index.html should advertise the supplied formal-ai version, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("__FORMAL_AI_VERSION__"),
        "stamped index.html should not retain the formal-ai version placeholder"
    );
    assert!(
        !rendered.contains("__FORMAL_AI_ASSET_VERSION__"),
        "stamped index.html should not retain the asset version placeholder"
    );
    assert!(
        rendered.contains("?v=deadbeef"),
        "issue #479: the stamped landing index must embed the deploy asset version (SHA) on its \
         asset refs so the Pages freshness probe and CDN cache-busting both see it, got:\n{rendered}"
    );
    assert!(
        rendered_tests.contains("content=\"9.9.9\""),
        "stamped tests/index.html should advertise the supplied formal-ai version, got:\n{rendered_tests}"
    );
    assert!(
        rendered_tests.contains("connectivity.js?v=deadbeef"),
        "stamp script should cache-bust nested test page assets, got:\n{rendered_tests}"
    );
    assert!(
        !rendered_tests.contains("__FORMAL_AI_VERSION__")
            && !rendered_tests.contains("__FORMAL_AI_ASSET_VERSION__"),
        "stamped tests/index.html should not retain placeholders, got:\n{rendered_tests}"
    );
    assert!(
        marker.contains("\"formal_ai_version\": \"9.9.9\""),
        "deployment.json should record the formal-ai version, got:\n{marker}"
    );
}

#[test]
fn static_demo_runtime_assets_are_cache_busted_by_deployment_version() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // The interactive app moved to /app/ (issue #479); its cache-busted runtime
    // assets now live in src/web/app/index.html. The site root index.html is the
    // landing-page chooser; it ALSO cache-busts its own assets with the deploy
    // asset version (so the stamped index embeds the SHA the Pages probe waits
    // for) but, unlike /app/, its strict CSP forbids an inline asset-version
    // script, so the SHA rides on ?v= refs rather than window.FORMAL_AI_ASSET_VERSION.
    let landing_html = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    let app_index_html =
        fs::read_to_string(format!("{manifest_dir}/src/web/app/index.html")).unwrap();
    let tests_index =
        fs::read_to_string(format!("{manifest_dir}/src/web/tests/index.html")).unwrap();
    let app_js = fs::read_to_string(format!("{manifest_dir}/src/web/app.js")).unwrap();
    let seed_loader_js =
        fs::read_to_string(format!("{manifest_dir}/src/web/seed_loader.js")).unwrap();
    let worker_js =
        fs::read_to_string(format!("{manifest_dir}/src/web/formal_ai_worker.js")).unwrap();
    let stamp_script =
        fs::read_to_string(format!("{manifest_dir}/scripts/stamp-pages-artifact.sh")).unwrap();
    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();

    for asset in [
        "styles.css?v=__FORMAL_AI_ASSET_VERSION__",
        "seed_loader.js?v=__FORMAL_AI_ASSET_VERSION__",
        "preferences.js?v=__FORMAL_AI_ASSET_VERSION__",
        "i18n.js?v=__FORMAL_AI_ASSET_VERSION__",
        "memory.js?v=__FORMAL_AI_ASSET_VERSION__",
        "app.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            app_index_html.contains(asset),
            "app/index.html should version local asset {asset}"
        );
    }
    assert!(app_index_html.contains("window.FORMAL_AI_ASSET_VERSION"));
    // The landing page (site root) advertises the stamped formal-ai version AND
    // cache-busts every one of its own assets with the deploy asset version, so
    // the stamped index embeds the SHA the Pages freshness probe waits for
    // (issue #479 regression guard).
    assert!(landing_html.contains("__FORMAL_AI_VERSION__"));
    for asset in [
        "landing.css?v=__FORMAL_AI_ASSET_VERSION__",
        "preferences.js?v=__FORMAL_AI_ASSET_VERSION__",
        "site-chrome.js?v=__FORMAL_AI_ASSET_VERSION__",
        "landing.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            landing_html.contains(asset),
            "landing index.html should version local asset {asset} so the stamped page carries the deploy SHA"
        );
    }
    assert!(app_js.contains("withAssetVersion(\"formal_ai_worker.js\")"));
    assert!(seed_loader_js.contains("fetchText(withAssetVersion(file))"));
    assert!(worker_js.contains("importScripts(withAssetVersion(\"seed_loader.js\"))"));
    assert!(worker_js.contains("fetch(withAssetVersion(\"formal_ai_worker.wasm\"))"));
    for asset in [
        "connectivity.css?v=__FORMAL_AI_ASSET_VERSION__",
        "connectivity.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            tests_index.contains(asset),
            "tests/index.html should version local asset {asset}"
        );
    }
    assert!(tests_index.contains("__FORMAL_AI_VERSION__"));
    assert!(stamp_script.contains("__FORMAL_AI_ASSET_VERSION__"));
    assert!(stamp_script.contains("find \"$artifact_dir\" -type f -name '*.html'"));
    assert!(stamp_script.contains("deployment.json"));
    assert!(wait_script.contains("deployment.json"));
    assert!(wait_script.contains("expected_sha"));
}

#[test]
fn crate_package_manifest_uses_publish_allowlist() {
    let manifest = fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
        .replace("\r\n", "\n");

    assert!(
        manifest.contains("include = ["),
        "Cargo.toml should explicitly allowlist files published to crates.io"
    );

    for required in [
        "\"/Cargo.lock\"",
        "\"/Cargo.toml\"",
        "\"/LICENSE\"",
        "\"/README.md\"",
        "\"/data/**\"",
        "\"/src/**\"",
    ] {
        assert!(
            manifest.contains(required),
            "Cargo.toml package include list should contain {required}"
        );
    }

    for excluded in [
        "\"/docs/**\"",
        "\"/tests/**\"",
        "\"/scripts/**\"",
        "\"/examples/**\"",
        "\"/experiments/**\"",
        "\"/.github/**\"",
    ] {
        assert!(
            !manifest.contains(excluded),
            "Cargo.toml should use an include allowlist instead of carrying explicit excluded repository artifacts"
        );
    }
}

#[test]
fn build_job_checks_generated_crate_archive_size() {
    let workflow = release_workflow();
    let build = job_block(&workflow, "build");
    let package_list = build
        .find("- name: Check package")
        .expect("build job should list package contents");
    let package_size = build
        .find("- name: Check crate package size")
        .expect("build job should check the generated crate archive size");
    let install_rust_script = build
        .find("- name: Install rust-script")
        .expect("build job should install rust-script before running script guards");

    assert!(
        package_list < package_size,
        "build job should list package contents before checking archive size"
    );
    assert!(
        install_rust_script < package_size,
        "build job should install rust-script before checking archive size"
    );
    assert!(
        build.contains("rust-script scripts/check-crate-package-size.rs"),
        "build job should run the crate size guard"
    );
}

#[test]
fn release_workflow_publishes_optional_docker_hub_image_after_crate_is_visible() {
    let workflow = release_workflow();

    assert!(
        workflow.contains("DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}"),
        "workflow should expose an opt-in Docker Hub image variable"
    );
    assert_eq!(
        workflow.matches("docker/login-action@v4").count(),
        2,
        "auto and manual release jobs should log in to Docker Hub when configured"
    );
    assert_eq!(
        workflow.matches("docker/metadata-action@v6").count(),
        2,
        "auto and manual release jobs should derive Docker tags for Docker Hub"
    );
    assert_eq!(
        workflow.matches("docker/build-push-action@v7").count(),
        2,
        "auto and manual release jobs should publish Docker Hub images when configured"
    );
    assert!(
        workflow.contains("password: ${{ env.DOCKERHUB_TOKEN }}"),
        "Docker Hub login should use DOCKERHUB_TOKEN"
    );

    let auto_release = job_block(&workflow, "auto-release");
    let auto_publish = auto_release
        .find("- name: Publish to Crates.io")
        .expect("auto release should publish the crate");
    let auto_wait = auto_release
        .find("- name: Wait for Crate availability on Crates.io")
        .expect("auto release should wait for the crate");
    let auto_docker = auto_release
        .find("- name: Publish Docker image to Docker Hub")
        .expect("auto release should publish the Docker image");
    let auto_github_release = auto_release
        .find("- name: Create GitHub Release")
        .expect("auto release should create a GitHub release");

    assert!(
        auto_publish < auto_wait && auto_wait < auto_docker && auto_docker < auto_github_release,
        "auto release should publish crates.io first, then Docker Hub, then GitHub release"
    );

    let manual_release = job_block(&workflow, "manual-release");
    let manual_publish = manual_release
        .find("- name: Publish to Crates.io")
        .expect("manual release should publish the crate");
    let manual_wait = manual_release
        .find("- name: Wait for Crate availability on Crates.io")
        .expect("manual release should wait for the crate");
    let manual_docker = manual_release
        .find("- name: Publish Docker image to Docker Hub")
        .expect("manual release should publish the Docker image");
    let manual_github_release = manual_release
        .find("- name: Create GitHub Release")
        .expect("manual release should create a GitHub release");

    assert!(
        manual_publish < manual_wait
            && manual_wait < manual_docker
            && manual_docker < manual_github_release,
        "manual release should publish crates.io first, then Docker Hub, then GitHub release"
    );
}

#[test]
fn release_workflow_defers_rate_limited_crates_publish_without_downstream_artifacts() {
    let workflow = release_workflow();
    let auto_release = job_block(&workflow, "auto-release");

    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(auto_release, step_name);
        assert!(
            step.contains("steps.check.outputs.crate_published == 'true'"),
            "auto-release {step_name} should still run when the crate was already published"
        );
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "auto-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains("steps.check.outputs.should_release == 'true'\n"),
            "auto-release {step_name} should not run solely because a release is needed"
        );
    }

    let manual_release = job_block(&workflow, "manual-release");
    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(manual_release, step_name);
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "manual-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains(
                "steps.version.outputs.version_committed == 'true' || steps.version.outputs.already_released == 'true'\n"
            ),
            "manual-release {step_name} should not run solely because a version step completed"
        );
    }
}

#[test]
fn desktop_release_lets_electron_builder_read_package_json_build_key() {
    let workflow = desktop_release_workflow();
    let package_json = fs::read_to_string(format!(
        "{}/desktop/package.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let smoke = fs::read_to_string(format!(
        "{}/desktop/scripts/smoke.mjs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(
        workflow.contains("npx --no-install electron-builder ${{ matrix.ebflag }} --publish never"),
        "desktop release workflow should invoke electron-builder without passing package.json as a config file"
    );
    assert!(
        !workflow.contains("--config package.json"),
        "package.json is an app manifest; passing it as --config makes electron-builder reject its top-level build key"
    );
    assert!(
        !package_json.contains("--config package.json"),
        "desktop npm build scripts should not pass package.json as an explicit electron-builder config file"
    );
    assert!(
        smoke.contains("--config package.json"),
        "desktop smoke checks should guard against reintroducing the invalid config flag"
    );
}

#[test]
fn release_scripts_check_configured_release_artifacts() {
    let release_check = fs::read_to_string(format!(
        "{}/scripts/check-release-needed.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let wait_for_crate = fs::read_to_string(format!(
        "{}/scripts/wait-for-crate.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let release_script = fs::read_to_string(format!(
        "{}/scripts/create-github-release.rs",
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
        release_script.contains("--docker-hub-url"),
        "GitHub release creation should accept a Docker Hub URL"
    );
    assert!(
        release_script.contains("fn docker_hub_badge"),
        "GitHub release notes should include Docker Hub badge support"
    );
}
