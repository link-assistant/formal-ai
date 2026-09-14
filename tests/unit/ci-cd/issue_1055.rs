//! Regression coverage for issue #1055: one build per platform, reused everywhere.
//!
//! The pipeline compiled the same project in four jobs, and link-time
//! optimization sat on the critical path of all of them. Measured with
//! `cargo test --release --no-run --bins --tests` from a touched `lib.rs`:
//!
//! ```text
//! lto = true,  codegen-units = 1    867s
//! lto = false, codegen-units unset  162s
//! ```
//!
//! 705 seconds, 5.4x. LTO is the one stage of a Rust build that does not
//! parallelize -- it merges every crate into a single optimization unit and
//! links it on one thread -- so unlike compilation it does not shrink when the
//! runner has more cores.
//!
//! The shape this pins:
//!
//! ```text
//! detect-changes
//!   ├── lint, secrets-scan          (need no build, run immediately)
//!   ├── build-artifacts             (binary + test executables, one compile)
//!   │     ├── test                  (downloads the executables)
//!   │     ├── docker-build          (downloads the binary)
//!   │     └── test-agent-cli-e2e    (downloads the binary)
//!   └── macos-core-tests            (its own archive, one compile per platform)
//!         └── build (packaging)     ── after every check
//!               └── release
//! ```

use std::fs;

use super::workflow_fixtures::release_workflow;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// What a job waits for, however its `needs:` is written.
///
/// Issue #1081: this read the rest of the `needs:` line and stopped there, so
/// the moment a list grew long enough to be wrapped
///
/// ```yaml
///     needs:
///       [
///         lint,
///         test,
///       ]
/// ```
///
/// -- which is the same YAML, and what a formatter produces -- every assertion
/// below saw an empty dependency list and passed. A helper that answers "this
/// job waits for nothing" when it waits for six things is the false negative
/// this repository's whole test surface is built to avoid, so the value is
/// read as a value: the key's line plus every line indented under it.
fn job_needs(workflow: &str, job: &str) -> String {
    let block = workflow
        .split(&format!("\n  {job}:\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("workflow declares a `{job}` job"));
    let lines: Vec<&str> = block.lines().collect();
    let job_level = |line: &str| line.starts_with("    ") && !line.starts_with("     ");
    let Some(key) = lines
        .iter()
        .position(|line| job_level(line) && line.trim_start().starts_with("needs:"))
    else {
        return String::new();
    };

    let mut value = lines[key]
        .trim()
        .strip_prefix("needs:")
        .unwrap_or_default()
        .trim()
        .to_string();
    for line in &lines[key + 1..] {
        if !line.starts_with("     ") {
            break;
        }
        value.push(' ');
        value.push_str(line.trim());
    }
    value
}

/// The helper above is a parser, so it gets a parser's test.
///
/// Both forms are the same YAML and GitHub reads them the same way; a test
/// that can only read one of them reports "no dependencies" for a job that has
/// six, which is worse than failing.
#[test]
fn a_wrapped_dependency_list_reads_the_same_as_an_inline_one() {
    let inline = "\n  release:\n    needs: [lint, test, build]\n    runs-on: ubuntu-latest\n";
    let wrapped = "\n  release:\n    needs:\n      [\n        lint,\n        test,\n        build,\n      ]\n    runs-on: ubuntu-latest\n";
    let block = "\n  release:\n    needs:\n      - lint\n      - test\n      - build\n    runs-on: ubuntu-latest\n";

    for form in [inline, wrapped, block] {
        let needs = job_needs(form, "release");
        for dependency in ["lint", "test", "build"] {
            assert!(
                needs.contains(dependency),
                "`{dependency}` is missing from {needs:?}"
            );
        }
    }

    assert_eq!(
        job_needs("\n  release:\n    runs-on: ubuntu-latest\n", "release"),
        "",
        "a job that really waits for nothing still reads as nothing"
    );
    assert!(
        !job_needs(
            "\n  release:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./a.yml\n        needs: nonsense\n",
            "release"
        )
        .contains("nonsense"),
        "only a job-level `needs:` is the job's dependency list"
    );
}

/// Link-time optimization stays off, so no job waits on a serial stage.
#[test]
fn the_release_profile_does_not_serialize_the_build() {
    let manifest = repository_file("Cargo.toml");
    let profile = manifest
        .split("[profile.release]")
        .nth(1)
        .expect("Cargo.toml declares [profile.release]");
    let profile = profile.split("\n[").next().unwrap_or(profile);

    assert!(
        profile.contains("lto = false"),
        "LTO merges every crate into one optimization unit and links it on a \
         single thread: 867s against 162s here, and no number of cores shortens \
         it. Everything downstream waits on that. Profile:\n{profile}"
    );
    let settings: String = profile
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !settings.contains("codegen-units = 1"),
        "`codegen-units = 1` forbids the compiler from splitting work across \
         cores, which is the opposite of what a hosted runner is for"
    );
}

/// Exactly one job compiles; everything else downloads.
#[test]
fn one_job_builds_and_the_rest_download() {
    // Issue #1081 moved the agent CLI E2E steps into their own file and
    // left the call behind; `pipeline_workflows()` splices the called
    // workflow back in at the job that calls it, so this still asks
    // "what does the pipeline run?" rather than "what is in this file?".
    let workflow = crate::ci_gates::pipeline_workflows();

    let compiles = workflow
        .split("\n      - name: ")
        .filter(|step| step.contains("cargo test --release --no-run --bins --tests"))
        .count();
    assert_eq!(
        compiles, 1,
        "the binary and the test executables must come from one compile; \
         found {compiles} jobs running it"
    );

    for (job, artifact) in [
        ("test", "formal-ai-test-executables"),
        ("docker-build", "download-formal-ai-binary"),
        ("test-agent-cli-e2e", "download-formal-ai-binary"),
        ("build", "download-formal-ai-binary"),
    ] {
        let block = super::workflow_fixtures::job_block(&workflow, job);
        assert!(
            block.contains(artifact),
            "`{job}` must consume `{artifact}` rather than compile again"
        );
    }
}

/// The test lane gets the binary as well as the executables.
///
/// `CARGO_BIN_EXE_formal-ai` is resolved when a test is *compiled*, and it
/// expands to `target/release/formal-ai` -- a path baked into every executable
/// that spawns the CLI, which 109 call sites across 36 test files do. Shipping
/// the executables without the binary makes all of those fail with
/// `NotFound`, and the message names the CLI rather than the missing artifact,
/// so the cause is not obvious from the failure.
#[test]
fn the_test_lane_downloads_the_binary_its_executables_expect() {
    let workflow = release_workflow();
    let job = super::workflow_fixtures::job_block(&workflow, "test");

    assert!(
        job.contains("download-formal-ai-binary"),
        "the executables spawn `target/release/formal-ai` by a path fixed at \
         compile time, so that file has to be present here too"
    );

    // Every binary, not one by name. `src/bin/with-formal-ai.rs` is
    // auto-discovered by cargo rather than declared in Cargo.toml, so a
    // hardcoded copy of `formal-ai` alone silently omitted it and
    // `with_formal_ai::standalone_with_formal_ai_binary_uses_same_wrapper`
    // failed pointing at a file the artifact never carried.
    let collector = repository_file("scripts/collect-build-artifacts.sh");
    assert!(
        !collector.contains("cp target/release/formal-ai "),
        "collect every binary `--bins` produced; naming one leaves the next \
         auto-discovered binary out of the artifact"
    );
}

/// Consumers declare the dependency that makes the artifact exist.
///
/// A job that downloads an artifact without depending on its producer is a
/// race: it usually passes because the producer happens to be quick, and fails
/// when it is not.
#[test]
fn every_consumer_waits_for_the_build() {
    let workflow = release_workflow();

    for job in ["test", "docker-build", "test-agent-cli-e2e", "build"] {
        let needs = job_needs(&workflow, job);
        assert!(
            needs.contains("build-artifacts"),
            "`{job}` downloads what `build-artifacts` produces, so it must wait \
             for it; without the edge the download races the build. needs:{needs}"
        );
    }
}

/// Checks that need no build are not held behind it.
#[test]
fn independent_checks_do_not_wait_for_the_build() {
    let workflow = release_workflow();

    for job in ["lint", "secrets-scan"] {
        let needs = job_needs(&workflow, job);
        assert!(
            !needs.contains("build-artifacts"),
            "`{job}` compiles nothing, so making it wait for the build only \
             lengthens the critical path. needs:{needs}"
        );
    }
}

/// Packaging and release come last, after every check.
#[test]
fn packaging_runs_after_the_checks_it_gates_on() {
    let workflow = release_workflow();

    let packaging = job_needs(&workflow, "build");
    for gate in ["lint", "test", "macos-core-tests"] {
        assert!(
            packaging.contains(gate),
            "packaging must not run before `{gate}`; a crate that fails its \
             tests should never be packaged. needs:{packaging}"
        );
    }

    let release = job_needs(&workflow, "auto-release");
    assert!(
        release.contains("build"),
        "release must follow packaging. needs:{release}"
    );
}
