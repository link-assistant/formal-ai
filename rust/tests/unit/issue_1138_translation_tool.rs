//! Plan 16 L2's translation tool, first rung (issue #1138): the any-direction
//! dispatcher through the meta pivot, and its honest-gap contract — a leg that
//! is not materialized names the plan-16 leaf that owes it instead of
//! silently producing nothing.

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

#[test]
fn every_translate_cli_text_lives_in_the_seed() {
    for intent in [
        "translate_needs_from_to",
        "translate_unknown_from",
        "translate_unknown_to",
        "translate_same_roots",
        "translate_pending",
        "translate_pending_docs",
        "translate_needs_input",
        "translate_leg_live",
        "translate_leg_pending",
        "translate_refused",
    ] {
        assert!(
            formal_ai::response_for(intent, "en").is_some(),
            "intent {intent} must live in data/seed/multilingual-responses-translate lino"
        );
    }
    assert_eq!(
        formal_ai::render_response(
            "translate_leg_live",
            "en",
            &[("from", "rust"), ("to", "meta")]
        ),
        Some("rust → meta  live".to_owned())
    );
    assert_eq!(
        formal_ai::render_response(
            "translate_leg_pending",
            "en",
            &[("from", "rust"), ("to", "js"), ("leaf", "L5")]
        ),
        Some("rust → js  pending (plan 16 L5)".to_owned())
    );
}

#[test]
fn every_distinct_pair_is_listed_exactly_once() {
    let directions = meta_translate::directions();
    assert_eq!(
        directions.len(),
        12,
        "four roots give twelve directed pairs, and every pair is a real direction"
    );
    let mut seen = std::collections::HashSet::new();
    for (from, to, pending) in &directions {
        assert_ne!(from, to);
        assert!(
            seen.insert((*from, *to)),
            "duplicate direction {from:?} → {to:?}"
        );
        assert_ne!(
            *pending,
            Some("L0"),
            "a listed direction is never a same-root non-direction"
        );
    }
}

#[test]
fn rust_to_meta_is_the_live_signature_leg() {
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Rust, SourceRoot::Meta),
        None
    );
    let source = "fn answer() -> u32 {\n    41 + 1\n}\n";
    match meta_translate::translate(SourceRoot::Rust, SourceRoot::Meta, "probe.rs", source) {
        TranslationOutcome::Rendered { target, .. } => {
            assert!(target.contains("self_ast"));
            assert!(target.contains("target probe.rs"));
            assert!(target.contains("language rust"));
            assert!(target.contains("engine meta_language"));
            assert!(target.contains("named_node_count"));
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("rust → meta must render the self-AST document; got {other:?}")
        }
    }
}

#[test]
fn pending_legs_name_their_plan_leaf() {
    for (from, to, pending) in meta_translate::directions() {
        let Some(leaf) = pending else { continue };
        assert!(
            leaf.starts_with('L') && leaf.len() >= 2 && leaf[1..].chars().all(char::is_numeric),
            "leaf {leaf:?} for {from:?} → {to:?} must name a plan 16 leaf"
        );
    }
    // Plan 16 L2 opened the ES quadrant: js/ts ↔ meta and js ↔ ts carry
    // through the token-tree pivot. The dogfood back-translation into rust
    // is owed by L3, and the meta → rust inverse by L5.
    for (from, to) in [
        (SourceRoot::JavaScript, SourceRoot::Meta),
        (SourceRoot::Meta, SourceRoot::JavaScript),
        (SourceRoot::TypeScript, SourceRoot::Meta),
        (SourceRoot::Meta, SourceRoot::TypeScript),
        (SourceRoot::JavaScript, SourceRoot::TypeScript),
        (SourceRoot::TypeScript, SourceRoot::JavaScript),
    ] {
        assert_eq!(
            meta_translate::pending_leg(from, to),
            None,
            "{from:?} → {to:?} is a plan 16 L2 leg and must be live"
        );
    }
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::JavaScript, SourceRoot::Rust),
        Some("L3")
    );
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Meta, SourceRoot::Rust),
        Some("L5")
    );
    // A same-root call is a non-direction, not a leaf.
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Rust, SourceRoot::Rust),
        Some("L0")
    );
}

#[test]
fn js_to_ts_carries_through_the_pivot() {
    match meta_translate::translate(
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        "app.js",
        "export const x = 1;\n",
    ) {
        TranslationOutcome::Rendered { target, carried } => {
            assert!(carried >= 5, "five tokens must cross: {carried}");
            assert!(
                target.contains("export const x = 1"),
                "the token-faithful carry renders the tokens: {target}"
            );
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("js → ts must carry through the pivot; got {other:?}")
        }
    }
}

#[test]
fn source_root_names_round_trip() {
    for root in [
        SourceRoot::Rust,
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        SourceRoot::Meta,
    ] {
        assert_eq!(SourceRoot::parse(root.name()), Some(root));
    }
    assert_eq!(SourceRoot::parse("rs"), Some(SourceRoot::Rust));
    assert_eq!(
        SourceRoot::parse("javascript"),
        Some(SourceRoot::JavaScript)
    );
    assert_eq!(
        SourceRoot::parse("typescript"),
        Some(SourceRoot::TypeScript)
    );
    assert_eq!(SourceRoot::parse("lino"), Some(SourceRoot::Meta));
    assert_eq!(SourceRoot::parse(""), None);
    assert_eq!(SourceRoot::parse("python"), None);
}
