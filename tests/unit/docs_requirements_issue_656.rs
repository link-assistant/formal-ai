use std::fs;
use std::path::Path;

/// Issue #656 (E37): the promotion protocol's documentation must stay pinned to
/// the live source. Citing a renamed section, a deleted file, or a stale
/// requirement fails here instead of drifting silently.
#[test]
fn issue_656_promotion_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #656 Benchmark-Gated Promotion Protocol",
            "| R445 ",
            "| R446 ",
            "| R447 ",
            "| R448 ",
            "| R449 ",
            "| R450 ",
            "| R451 ",
            "src/promotion.rs",
            "formal-ai improve --promote",
            // R385 must now point at issue #656, not the closed #558.
            "the benchmark-gated promotion of proposals into seed data is implemented by issue #656",
        ],
    );

    let meta_algorithm = read(root.join("docs/meta-algorithm.md"));
    assert_contains_all(
        "docs/meta-algorithm.md",
        &meta_algorithm,
        &[
            "The promotion meta-algorithm (issue #656)",
            "src/promotion.rs",
            "tests/unit/issue_656_promotion.rs",
            "tests/integration/issue_656_improve.rs",
            "formal-ai improve --promote",
            "cargo test promotion_protocol",
            "promotion_proposal",
            "promotion_evidence",
            "promotion_decision",
            "promotion_applied",
            "promotion_rejection",
            "never a direct push",
        ],
    );

    // Every file the docs cite must exist and be non-empty.
    for relative in [
        "src/promotion.rs",
        "src/cli_improve.rs",
        "tests/unit/issue_656_promotion.rs",
        "tests/integration/issue_656_improve.rs",
        "changelog.d/20260714_090000_issue_656_promotion.md",
    ] {
        let path = root.join(relative);
        assert!(
            path.is_file(),
            "{relative} should exist for issue #656 traceability",
        );
        assert!(
            path.metadata().map_or(0, |meta| meta.len()) > 0,
            "{relative} must not be empty for issue #656 traceability",
        );
    }

    // The protocol's public API cited above must still appear in the live source.
    let promotion_src = read(root.join("src/promotion.rs"));
    for identifier in [
        "pub struct PromotionRatchet",
        "pub struct PromotionProposal",
        "pub struct PromotionRun",
        "pub enum PromotionOutcome",
        "pub fn apply_promotions",
        "fn memory_events",
        "promotion_rejection",
    ] {
        assert!(
            promotion_src.contains(identifier),
            "src/promotion.rs should still define `{identifier}`",
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}",
        );
    }
}
