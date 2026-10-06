//! Issue #1168: generated code must use the latest versions discovered at
//! generation time -- the resolution machinery lives in
//! `rust/src/version_resolution.rs` (its tests run with the crate), and this
//! file pins the data-side half: the `data/` gate that fails on any memorized
//! third-party action SHA or toolchain version.
//!
//! The gate is drafted red-by-design against the two template files that
//! still carry literals (`data/meta/stdout-program-contracts.lino`,
//! `data/meta/work-item-steps.lino`): the same change that rewrites them to
//! `{placeholders}` (R6) turns both the gate and the pinning test green. The
//! test here therefore asserts the findings are confined to those two known
//! templates -- zero elsewhere -- rather than zero everywhere, so it stays
//! honest before and after the rewrite.

#[path = "../../../scripts/check-generated-version-literals.rs"]
mod check_generated_version_literals;

use std::fs;
use std::path::{Path, PathBuf};

use check_generated_version_literals::scan_data;

fn repository_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join("data")
}

fn fixture_tree(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "issue-1168-gate-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    for (name, body) in files {
        let path = root.join("data").join(name);
        fs::create_dir_all(path.parent().expect("a directory")).expect("fixture dirs");
        fs::write(path, body).expect("fixture file");
    }
    root.join("data")
}

#[test]
fn the_gate_fails_on_a_memorized_action_sha() {
    let data = fixture_tree(
        "sha",
        &[(
            "meta/template.lino",
            "  workflow_template \"      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262\n\"\n",
        )],
    );
    let findings = scan_data(&data);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].path.ends_with("data/meta/template.lino"));
    assert_eq!(findings[0].line, 1);
}

#[test]
fn the_gate_fails_on_a_memorized_toolchain_version() {
    let data = fixture_tree(
        "version",
        &[(
            "meta/contract.lino",
            "    ci_setup \"        with:\n          version: '2.3.10'\n\"\n",
        )],
    );
    let findings = scan_data(&data);
    assert!(
        findings.iter().any(|f| f.detail.contains("version")),
        "{findings:?}"
    );
}

#[test]
fn placeholder_templates_and_the_baseline_seed_scan_clean() {
    let data = fixture_tree(
        "clean",
        &[
            (
                "meta/template.lino",
                "  workflow_template \"      - uses: actions/checkout@{checkout_ref} # {checkout_tag}\n\"\n",
            ),
            (
                "seed/toolchains.lino",
                "generated_version actions_checkout\n  latest_tag \"v7.0.1\"\n  latest_sha \"3d3c42e5aac5ba805825da76410c181273ba90b1\"\n",
            ),
        ],
    );
    assert!(scan_data(&data).is_empty());
}

#[test]
fn captured_evidence_under_cache_is_not_generated_code() {
    let data = fixture_tree(
        "cache",
        &[(
            "cache/wiktionary/en/example.lino",
            "  etymology \"pinned in a captured page: something@0123456789abcdef0123456789abcdef01234567\"\n",
        )],
    );
    assert!(scan_data(&data).is_empty());
}

#[test]
fn repository_findings_are_confined_to_the_two_unrewritten_templates() {
    let known = [
        "data/meta/stdout-program-contracts.lino",
        "data/meta/work-item-steps.lino",
    ];
    let data = repository_data();
    let findings = scan_data(&data);
    for finding in &findings {
        let path = finding.path.display().to_string();
        assert!(
            known.contains(&path.as_str()),
            "unexpected memorized pin outside the two templates awaiting the R6 rewrite: {path}: {}",
            finding.detail
        );
    }
}
