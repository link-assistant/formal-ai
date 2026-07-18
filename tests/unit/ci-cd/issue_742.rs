//! Regression coverage for issue #742's documentation publishing contract.

use std::fs;

fn manifest() -> String {
    fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .expect("Cargo.toml should be readable")
}

fn release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("release workflow should be readable")
}

#[test]
fn docs_rs_profile_excludes_the_broken_lindera_build_script() {
    let manifest = manifest();

    assert!(
        manifest.contains("meta-language = { version = \"0.45.0\", optional = true }"),
        "the transitive lindera build script can only be avoided when meta-language is optional"
    );
    assert!(manifest.contains("default = [\"doublets-native\", \"meta-language\"]"));
    assert!(manifest.contains("[package.metadata.docs.rs]"));
    assert!(manifest.contains("no-default-features = true"));
}

#[test]
fn pull_requests_validate_the_same_dependency_profile_docs_rs_uses() {
    let workflow = release_workflow();

    assert!(workflow.contains("DOCS_RS: 1"));
    assert!(workflow.contains("cargo doc --no-deps --lib --no-default-features"));
}

#[test]
fn generated_api_docs_are_published_below_the_site_docs_route() {
    let workflow = release_workflow();

    assert!(workflow.contains("mkdir -p src/web/docs/api"));
    assert!(workflow.contains("cp -R target/doc/. src/web/docs/api/"));
    assert!(workflow.contains("uses: actions/upload-pages-artifact@v5"));
    assert!(workflow.contains("path: src/web"));
    assert!(workflow.contains("uses: actions/deploy-pages@v5"));
}

#[test]
fn whole_docs_task_is_fail_closed_from_validation_through_deployment() {
    let manifest = manifest();
    let workflow = release_workflow();

    let validate = workflow
        .find("cargo doc --no-deps --lib --no-default-features")
        .expect("missing docs.rs-compatible validation");
    let build = workflow
        .find("  build:\n")
        .expect("missing package build job");
    let deploy = workflow
        .find("  deploy-pages:\n")
        .expect("missing Pages deployment job");

    assert!(manifest.contains("no-default-features = true"));
    assert!(
        validate < build,
        "documentation must be validated before release packaging"
    );
    assert!(
        build < deploy,
        "Pages deployment must wait for the verified package build"
    );
    assert!(workflow.contains("needs.build.result == 'success'"));
    assert!(workflow.contains("needs.deploy-pages.result == 'success'"));
}
