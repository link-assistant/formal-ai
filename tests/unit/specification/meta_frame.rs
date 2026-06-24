//! Issue #559 Phase 1A: problem-frame construction and trace.
//!
//! These tests pin the explicit, link-serializable problem frame (R330): a
//! single prompt produces a frame that enumerates every detected need (R7) and
//! serializes to Links Notation (R311), while single-intent prompts still carry
//! exactly one need so routing behavior is unchanged (R13). Backward
//! compatibility of the *answer* is covered by the existing reasoning-loop and
//! handler tests; here we assert the frame itself is correct and stable.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{AtomicityReason, NeedLedger, NeedStatus, ProblemFrame, WorkUnit};
use formal_ai::translation::formalize_prompt;
use formal_ai::IntentKind;

fn frame_for(prompt: &str) -> ProblemFrame {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    ProblemFrame::from_formalization(&formalization)
}

fn frame_for_lang(prompt: &str, language: &str) -> ProblemFrame {
    let candidate = formalize_prompt(prompt, language);
    let formalization = formalize_intent(prompt, language, Some(&candidate));
    ProblemFrame::from_formalization(&formalization)
}

fn work_unit_for(prompt: &str, max_depth: u8) -> WorkUnit {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    WorkUnit::from_formalization(&formalization, max_depth)
}

fn collect_leaves(unit: &WorkUnit, out: &mut Vec<String>) {
    if unit.atomic {
        out.push(unit.source_span.clone());
    } else {
        for child in &unit.children {
            collect_leaves(child, out);
        }
    }
}

fn deepest_depth(unit: &WorkUnit) -> u8 {
    unit.children
        .iter()
        .map(deepest_depth)
        .max()
        .unwrap_or(unit.depth)
}

fn ledger_for(prompt: &str) -> (ProblemFrame, NeedLedger) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    let root = WorkUnit::from_formalization(&formalization, 4);
    let ledger = NeedLedger::resolve(&frame, &root);
    (frame, ledger)
}

#[test]
fn single_intent_prompt_yields_exactly_one_need() {
    let frame = frame_for("translate apple to Russian");
    assert_eq!(
        frame.need_count(),
        1,
        "a single-intent prompt must collapse to one need so routing is unchanged"
    );
    let need = &frame.needs[0];
    assert_eq!(need.source_span, "translate apple to Russian");
    assert_eq!(need.kind, frame.kind);
    assert_eq!(need.route, frame.route);
    assert_eq!(need.status, NeedStatus::Pending);
}

#[test]
fn frame_mirrors_formalization_classification() {
    let candidate = formalize_prompt("translate apple to Russian", "en");
    let formalization = formalize_intent("translate apple to Russian", "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);

    assert_eq!(frame.impulse_id, formalization.impulse_id);
    assert_eq!(frame.language, "en");
    assert_eq!(frame.kind, IntentKind::Task);
    assert_eq!(frame.route.as_deref(), Some("translation"));
}

#[test]
fn conjunction_prompt_detects_multiple_needs() {
    let frame = frame_for("translate apple to Russian and write a hello world program in Python");
    assert!(
        frame.need_count() >= 2,
        "a conjunction prompt must surface more than one need, got {}: {:?}",
        frame.need_count(),
        frame
            .needs
            .iter()
            .map(|n| &n.source_span)
            .collect::<Vec<_>>()
    );
    let spans: Vec<&str> = frame.needs.iter().map(|n| n.source_span.as_str()).collect();
    assert!(
        spans.iter().any(|span| span.contains("translate apple")),
        "the translate clause should be its own need: {spans:?}"
    );
    assert!(
        spans.iter().any(|span| span.contains("hello world")),
        "the write-program clause should be its own need: {spans:?}"
    );
}

#[test]
fn multi_sentence_prompt_detects_a_question_and_a_task() {
    let frame = frame_for("What is the capital of France? Translate apple to Russian.");
    assert!(
        frame.need_count() >= 2,
        "a question followed by a task must yield two needs: {:?}",
        frame
            .needs
            .iter()
            .map(|n| &n.source_span)
            .collect::<Vec<_>>()
    );
    assert!(
        frame
            .needs
            .iter()
            .any(|need| need.kind == IntentKind::Question),
        "the interrogative sentence must classify as a question"
    );
    assert!(
        frame.needs.iter().any(|need| need.kind == IntentKind::Task),
        "the imperative sentence must classify as a task"
    );
}

#[test]
fn decimals_are_not_split_into_separate_needs() {
    let frame = frame_for("calculate 3.14 times 2");
    assert_eq!(
        frame.need_count(),
        1,
        "a decimal must not be split on its period: {:?}",
        frame
            .needs
            .iter()
            .map(|n| &n.source_span)
            .collect::<Vec<_>>()
    );
    assert!(frame.needs[0].source_span.contains("3.14"));
}

#[test]
fn frame_serializes_to_grounded_links_notation() {
    let frame = frame_for("translate apple to Russian and write a hello world program in Python");
    let lino = frame.to_links_notation();

    assert!(
        lino.contains("record_type \"problem_frame\""),
        "frame record must declare its record_type:\n{lino}"
    );
    assert!(
        lino.contains(&format!("impulse_id \"{}\"", frame.impulse_id)),
        "frame must serialize its impulse id:\n{lino}"
    );
    assert!(
        lino.contains("record_type \"problem_need\""),
        "every need must serialize as its own record:\n{lino}"
    );
    assert!(
        lino.contains(&format!("need_count \"{}\"", frame.need_count())),
        "frame must record how many needs it found:\n{lino}"
    );
    // Each need id referenced by the frame must also head a need record.
    for need in &frame.needs {
        assert!(
            lino.contains(&need.need_id),
            "need id {} must appear in the trace:\n{lino}",
            need.need_id
        );
    }
}

#[test]
fn single_intent_prompt_is_an_atomic_root() {
    let root = work_unit_for("translate apple to Russian", 4);
    assert_eq!(root.depth, 0);
    assert!(
        root.atomic,
        "a single-intent prompt must be an atomic root so its leaf routes as today: {root:?}"
    );
    assert!(
        root.children.is_empty(),
        "an atomic root must have no children"
    );
    assert_eq!(
        root.reason,
        AtomicityReason::DirectMethod,
        "a recognized route must record a direct-method leaf"
    );
    assert_eq!(root.unit_count(), 1);
    assert_eq!(root.leaf_count(), 1);
}

#[test]
fn conjunction_prompt_decomposes_into_atomic_leaves() {
    let root = work_unit_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    assert!(
        !root.atomic,
        "a conjunction root must decompose into children: {root:?}"
    );
    assert_eq!(root.reason, AtomicityReason::NotAtomic);
    assert!(
        root.children.len() >= 2,
        "a conjunction must yield at least two children: {root:?}"
    );
    let mut leaves = Vec::new();
    collect_leaves(&root, &mut leaves);
    assert!(
        leaves.iter().any(|span| span.contains("translate apple")),
        "the translate clause must reach a leaf: {leaves:?}"
    );
    assert!(
        leaves.iter().any(|span| span.contains("hello world")),
        "the write-program clause must reach a leaf: {leaves:?}"
    );
    assert_eq!(
        root.leaf_count(),
        leaves.len(),
        "leaf_count must match the collected leaves"
    );
}

#[test]
fn recursion_is_bounded_by_max_depth() {
    let prompt = "translate apple to Russian and write a hello world program in Python";
    let shallow = work_unit_for(prompt, 1);
    assert!(
        deepest_depth(&shallow) <= 1,
        "no unit may exceed the configured max depth: {shallow:?}"
    );
}

#[test]
fn depth_bounded_leaf_records_its_reason() {
    // With max_depth 0 the root is forced to a leaf immediately, even though it
    // is a multi-need conjunction, so the recursion is always bounded.
    let root = work_unit_for(
        "translate apple to Russian and write a hello world program in Python",
        0,
    );
    assert!(root.atomic, "max_depth 0 must force the root to a leaf");
    assert_eq!(root.reason, AtomicityReason::DepthBound);
    assert!(root.children.is_empty());
}

#[test]
fn work_unit_tree_serializes_to_grounded_links_notation() {
    let root = work_unit_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let lino = root.to_links_notation();
    assert!(
        lino.contains("record_type \"work_unit\""),
        "every unit must declare its record_type:\n{lino}"
    );
    assert!(
        lino.contains("atomicity_reason \"not_atomic\""),
        "the non-atomic root must record its reason:\n{lino}"
    );
    assert!(
        lino.contains(&format!("unit_id \"{}\"", root.unit_id)),
        "the root unit id must serialize:\n{lino}"
    );
    for child in &root.children {
        assert!(
            lino.contains(&child.unit_id),
            "child unit {} must appear in the trace:\n{lino}",
            child.unit_id
        );
    }
}

#[test]
fn ledger_has_exactly_one_row_per_need() {
    let (frame, ledger) =
        ledger_for("translate apple to Russian and write a hello world program in Python");
    assert_eq!(
        ledger.rows.len(),
        frame.needs.len(),
        "the ledger must account for every detected need exactly once"
    );
    for need in &frame.needs {
        assert!(
            ledger.rows.iter().any(|row| row.need_id == need.need_id),
            "need {} must appear in the ledger",
            need.need_id
        );
    }
}

#[test]
fn every_need_is_accounted_for_with_a_non_pending_status() {
    let (_frame, ledger) =
        ledger_for("translate apple to Russian and write a hello world program in Python");
    assert!(
        ledger.every_need_accounted_for(),
        "no need may stay pending in the resolved ledger: {ledger:?}"
    );
    assert!(
        ledger
            .rows
            .iter()
            .all(|row| row.status != NeedStatus::Pending),
        "every row must carry an explicit status"
    );
}

#[test]
fn routed_need_is_satisfied_and_unroutable_need_is_blocked() {
    let (_frame, ledger) = ledger_for("translate apple to Russian");
    assert_eq!(ledger.rows.len(), 1);
    assert_eq!(
        ledger.rows[0].status,
        NeedStatus::Satisfied,
        "a need that maps to a known method must be satisfiable"
    );
    assert_eq!(
        ledger.rows[0].leaf_reason,
        Some(AtomicityReason::DirectMethod)
    );

    let (_frame, blocked) = ledger_for("zzqqx unfathomable gibberish token");
    assert_eq!(blocked.rows.len(), 1);
    assert_eq!(
        blocked.rows[0].status,
        NeedStatus::Blocked,
        "a need with no recognized method must be recorded as blocked, not dropped"
    );
}

#[test]
fn ledger_serializes_to_grounded_links_notation() {
    let (_frame, ledger) =
        ledger_for("translate apple to Russian and write a hello world program in Python");
    let lino = ledger.to_links_notation();
    assert!(
        lino.contains("record_type \"need_ledger\""),
        "the ledger must declare its record_type:\n{lino}"
    );
    assert!(
        lino.contains("record_type \"need_ledger_row\""),
        "every row must serialize as its own record:\n{lino}"
    );
    assert!(
        lino.contains(&format!("row_count \"{}\"", ledger.rows.len())),
        "the ledger must record its row count:\n{lino}"
    );
}

#[test]
fn frame_formalizes_requests_in_every_supported_language() {
    // The meta core is language-agnostic by design: every message is translated
    // into the same link-based meta language and worked on directly, so a request
    // in any supported language must produce a frame that records that language
    // and detects at least one need. This pins that property across all four
    // supported languages (issue #559: "translate every message to a meta
    // language and work on it"), so a language-specific regression cannot land
    // with only one language pinned.
    let cases = [
        ("english", "en", "translate apple to Russian"),
        ("russian", "ru", "переведи яблоко на английский"),
        ("hindi", "hi", "सेब का अंग्रेज़ी में अनुवाद करें"),
        ("chinese", "zh", "把苹果翻译成英文"),
    ];
    for (name, code, prompt) in cases {
        let frame = frame_for_lang(prompt, code);
        assert_eq!(
            frame.language, code,
            "the {name} frame must record its own language tag, got {:?}",
            frame.language
        );
        assert!(
            frame.need_count() >= 1,
            "a {name} request must surface at least one need: {frame:?}"
        );
        assert_eq!(
            frame.needs.len(),
            frame.need_count(),
            "the {name} frame's need list must match its reported count"
        );
        // The frame must still serialize to grounded Links Notation regardless of
        // the source language's script.
        let lino = frame.to_links_notation();
        assert!(
            lino.contains(&format!("language \"{code}\"")),
            "the {name} frame must serialize its language tag:\n{lino}"
        );
    }
}

#[test]
fn need_ids_are_stable_and_unique() {
    let first = frame_for("translate apple to Russian and write a hello world program in Python");
    let second = frame_for("translate apple to Russian and write a hello world program in Python");
    let ids_first: Vec<&str> = first.needs.iter().map(|n| n.need_id.as_str()).collect();
    let ids_second: Vec<&str> = second.needs.iter().map(|n| n.need_id.as_str()).collect();
    assert_eq!(ids_first, ids_second, "need ids must be deterministic");

    let mut unique = ids_first.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        ids_first.len(),
        "need ids must be unique within a frame"
    );
}
