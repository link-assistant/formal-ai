//! Regression coverage for issue #1051: the same binary was compiled four
//! times in one pull request.
//!
//! Measured on run 32598625222:
//!
//! ```text
//! E2E Tests (agent CLI)        10.8m   cargo build --release --bin formal-ai
//! Build Box Language Binary     4.2m   cargo build --release --bin formal-ai
//! Docker Image Build           33.0m   cargo build --release, inside `BuildKit`
//! Build Package                 5.7m   cargo build --release --verbose
//! ```
//!
//! The first two are byte-identical invocations -- same command, same profile
//! overrides, same binary -- and the second already uploaded its result as an
//! artifact for another job to download. The mechanism for sharing was in the
//! repository and simply unused between them.
//!
//! Docker was the worst of the four. It compiled 510 crates with **no
//! sccache**, because `RUSTC_WRAPPER` and the GitHub Actions cache token live
//! on the runner and `BuildKit` cannot reach them -- so the one job that could
//! least afford to recompile was the only one doing it cold. At 33 of the pull
//! request's 42.8 minutes it gated the entire run by itself.
//!
//! The invariant pinned here: **compile once, then reuse.** What ships is
//! exempt -- the publish jobs still build the image from source, so the
//! Dockerfile stays proven able to build the project standalone.

use std::fs;

use super::workflow_fixtures::release_workflow;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// Only one job compiles the release binary; the rest download it.
#[test]
fn the_release_binary_is_compiled_once_per_pipeline() {
    let workflow = release_workflow();

    let builders: Vec<&str> = workflow
        .split("\n      - name: ")
        .filter(|step| step.contains("cargo test --release --no-run --bins --tests"))
        .collect();

    assert_eq!(
        builders.len(),
        1,
        "the binary and the test executables must be compiled once, in one \
         job, and downloaded everywhere else; found {} jobs running that \
         build. Each duplicate costs its own minutes on the critical path \
         for a byte-identical result.",
        builders.len()
    );

    let consumers = workflow
        .matches("uses: ./.github/actions/download-formal-ai-binary")
        .count();
    assert!(
        consumers >= 3,
        "the one build should feed at least the E2E, box-language and Docker \
         jobs; found {consumers} jobs downloading the shared artifact"
    );
}

/// The Docker check reuses the binary rather than recompiling it.
#[test]
fn the_pull_request_image_reuses_the_prebuilt_binary() {
    let workflow = release_workflow();

    let step = workflow
        .split("- name: Build image")
        .nth(1)
        .expect("the Docker check builds an image");
    let step = step.split("- name:").next().unwrap_or(step);

    assert!(
        step.contains("BINARY_SOURCE=prebuilt"),
        "compiling inside BuildKit cannot use sccache -- the wrapper and the \
         cache token live on the runner -- so it recompiled 510 crates cold for \
         33 minutes. Step:\n{step}"
    );
}

/// What ships is still built from source.
///
/// The prebuilt path trades the Dockerfile's standalone-build guarantee for
/// speed. That trade is acceptable for a check that only verifies the runtime
/// contract, and unacceptable for the image users install -- so the publish
/// jobs must never pass the override.
#[test]
fn published_images_are_still_built_from_source() {
    let workflow = release_workflow();

    for step in workflow.split("\n      - name: ") {
        let name = step.lines().next().unwrap_or_default();
        if !name.contains("Publish Docker image") {
            continue;
        }
        assert!(
            !step.contains("BINARY_SOURCE"),
            "`{name}` ships to users, so it must build the image from source \
             and keep proving the Dockerfile can do so unaided. Step:\n{step}"
        );
    }
}

/// The prebuilt binary can actually reach the build context.
///
/// `.dockerignore` excludes `target`, which is right -- a local tree of build
/// artifacts is gigabytes and must never be uploaded. But it also blocked the
/// one file the prebuilt path needs, and the failure is not obvious from the
/// message `BuildKit` prints:
///
/// ```text
/// failed to compute cache key: "/target/release/formal-ai": not found
/// ```
///
/// Verified against real Docker that the exception re-includes that single file
/// and nothing else: a context with `target/debug/junk` alongside it copies
/// only `release/formal-ai`.
#[test]
fn the_prebuilt_binary_is_not_excluded_from_the_build_context() {
    let ignore = repository_file(".dockerignore");

    assert!(
        ignore.contains("target"),
        "`target/` must stay excluded; it is gigabytes of build artifacts"
    );
    assert!(
        ignore.contains("!target/release/formal-ai"),
        "the prebuilt path copies exactly this file, so it needs an exception \
         -- without it the image build fails with a `not found` that reads like \
         a Dockerfile bug rather than an ignore rule"
    );
}

/// The Dockerfile still knows how to build from source.
#[test]
fn the_dockerfile_keeps_a_from_source_path() {
    let dockerfile = repository_file("Dockerfile");

    assert!(
        dockerfile.contains("ARG BINARY_SOURCE=compile"),
        "the default must stay `compile`, so anyone building this image by \
         hand gets a self-contained build rather than a failure about a \
         missing artifact"
    );
    assert!(
        dockerfile.contains("AS compile-binary") && dockerfile.contains("AS prebuilt-binary"),
        "both paths have to exist for the selection to mean anything"
    );
    assert!(
        dockerfile.contains("cargo build --release --locked --bins"),
        "the from-source path must still actually compile the project"
    );
}
