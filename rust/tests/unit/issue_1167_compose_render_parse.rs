//! Issue #1167 (E132), R1: the CST render legs report named gaps, never
//! silent skips.
//!
//! `formal_ai::meta_translate::render_cst_source` is the public render leg
//! every `program-cst-grammars.lino` entry owes. These tests pin its refusal
//! behaviour through the crate's public surface; the full per-language
//! compose→render→parse→CST-equal round trip (R4) lives in the source test
//! tree (`rust/tests/source/source_tests/coding/cst/tests.rs`), which
//! compiles the coding modules directly.

use formal_ai::meta_translate::{CstRenderGap, render_cst_source, try_render_cst_source};

#[test]
fn render_cst_source_reports_missing_language() {
    // The issue's own named test: an unregistered slug is None, so no caller
    // can confuse a missing leg with an empty program.
    assert!(render_cst_source("", "nonexistent").is_none());
}

#[test]
fn the_missing_language_gap_is_named() {
    let gap = try_render_cst_source("", "nonexistent").expect_err("an unregistered slug refuses");
    assert!(
        matches!(gap, CstRenderGap::NoGrammarEntry { .. }),
        "expected NoGrammarEntry, got {gap:?}"
    );
    assert!(
        gap.describe().contains("no cst_grammar entry"),
        "the gap must name what is missing: {}",
        gap.describe()
    );
}

#[test]
fn a_registered_target_refuses_the_document_not_the_language() {
    // Kotlin is a registered render target since this issue; a non-network
    // document fails for the *document* reason, which distinguishes a live
    // leg with bad input from a missing leg.
    let gap = try_render_cst_source("census_document\n  signature fn\n", "kotlin")
        .expect_err("a non-network document refuses");
    assert!(
        matches!(gap, CstRenderGap::NotNetworkLino { .. }),
        "expected NotNetworkLino, got {gap:?}"
    );
    assert!(
        gap.describe().contains("network serialization dialect"),
        "the gap must name the expected dialect: {}",
        gap.describe()
    );
}

#[test]
fn every_newly_registered_target_is_a_named_render_leg() {
    // The four grammars registered by this issue are reachable legs: each
    // refuses a foreign document with the dialect reason rather than the
    // no-entry reason, proving the grammar table entry is wired through.
    for slug in ["kotlin", "scala", "swift", "r"] {
        let gap = try_render_cst_source("not a network document", slug)
            .expect_err("a non-network document refuses");
        assert!(
            matches!(gap, CstRenderGap::NotNetworkLino { .. }),
            "`{slug}` must be a registered render leg refusing the document, got {gap:?}"
        );
    }
}
