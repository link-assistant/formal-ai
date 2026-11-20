//! CodeQL coverage and extractor prerequisites.

use std::fs;

use super::{job_block, repository_file};

/// The `CodeQL` run emitted parse diagnostics on every single invocation because
/// the Rust extractor, in `build-mode: none`, parses every `.rs` file on disk --
/// including a deliberately truncated documentation excerpt.
#[test]
fn codeql_skips_archived_evidence_but_still_analyses_compiled_code() {
    let security = repository_file(".github/workflows/security.yml");
    assert!(
        security.contains("config-file: ./.github/codeql/codeql-config.yml"),
        "the CodeQL init step must load the shared config"
    );

    let config = repository_file(".github/codeql/codeql-config.yml");
    let excluded_paths: Vec<&str> = config
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.trim().strip_prefix("- "))
        .collect();

    for excluded in ["docs/**", "dev/**", "experiments/**"] {
        assert!(
            excluded_paths.contains(&excluded),
            "{excluded} holds archived evidence and scratch reproductions; \
             parsing it only produces diagnostics (issue #1017)"
        );
    }
    assert!(
        !excluded_paths
            .iter()
            .any(|path| path.starts_with("examples")),
        "`examples/` are real Cargo targets the workspace compiles, so they \
         must stay in scope: {excluded_paths:?}"
    );

    // The file that produced the diagnostics is still archived, so the
    // exclusion above is load-bearing rather than historical.
    let excerpt = "docs/case-studies/issue-96/raw-data/link-calculator-lib-excerpt.rs";
    assert!(
        fs::metadata(format!("{}/../{excerpt}", env!("CARGO_MANIFEST_DIR"))).is_ok(),
        "{excerpt} is the truncated excerpt the exclusion exists for"
    );
}

/// A `CodeQL` run that reports success while 1,023 files were "extracted with
/// errors" is the worst kind of false negative: the dashboard is green because
/// nothing was analysed. Run 31937348308 emitted 20,725 `macro expansion
/// failed` diagnostics, every one of them for a macro defined in `std`/`core`
/// or in a dependency and none defined here, because the extractor had no
/// sysroot override and the ambient `std` outran the rust-analyzer the `CodeQL`
/// bundle vendors (github/codeql#19982).
///
/// Both variables are load-bearing. Setting only `_SYSROOT_SRC` takes
/// rust-analyzer's `discover_with_src_override` path, which keeps the
/// discovered binary sysroot and leaves the failures in place — so a future
/// edit that drops one of them would silently restore the false negative.
#[test]
fn codeql_rust_lane_pins_the_extractor_sysroot() {
    let security = repository_file(".github/workflows/security.yml");
    let job = job_block(&security, "codeql");

    for variable in [
        "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=",
        "CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=",
    ] {
        assert!(
            job.contains(variable),
            "the CodeQL job must export {variable} into $GITHUB_ENV, or `std` \
             macros stay unexpanded and their call sites go unanalysed \
             (issue #1017, github/codeql#19982)"
        );
    }

    assert!(
        job.contains("CODEQL_RUST_SYSROOT_TOOLCHAIN:"),
        "the pinned toolchain must be declared once, as a named variable, so \
         raising it as CodeQL vendors a newer rust-analyzer is a one-line edit"
    );
    assert!(
        job.contains("if: matrix.language == 'rust'"),
        "the sysroot pin costs a toolchain install and only affects the Rust \
         extractor, so it must not run in the `actions` lane"
    );
    assert!(
        job.contains("--component rust-src"),
        "`_SYSROOT_SRC` points into `rust-src`; without the component the \
         directory does not exist and the pin silently does nothing"
    );
    assert!(
        job.contains("::error title=CodeQL sysroot pin unavailable::"),
        "losing the pin must fail before extraction: falling back to an \
         incompatible sysroot would let the 20,725 diagnostics return unnoticed"
    );
}
