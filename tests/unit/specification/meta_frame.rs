//! Issue #559 Phase 1A: problem-frame construction and trace.
//!
//! These tests pin the explicit, link-serializable problem frame (R330): a
//! single prompt produces a frame that enumerates every detected need (R7) and
//! serializes to Links Notation (R311), while single-intent prompts still carry
//! exactly one need so routing behavior is unchanged (R13). Backward
//! compatibility of the *answer* is covered by the existing reasoning-loop and
//! handler tests; here we assert the frame itself is correct and stable.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{NeedStatus, ProblemFrame};
use formal_ai::translation::formalize_prompt;
use formal_ai::IntentKind;

fn frame_for(prompt: &str) -> ProblemFrame {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    ProblemFrame::from_formalization(&formalization)
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
