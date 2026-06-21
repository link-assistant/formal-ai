// Issue #1963 (problem 2): "Thinking steps are not fully written, some parts are
// omitted."
//
// The naturalized thinking detail was clipped at 120 characters, truncating a
// realistic single step (a pasted prompt, a composed answer) mid-sentence so the
// visible reasoning read as incomplete rather than fully written. The cap was
// raised to 600 in `truncate_thinking_detail` (mirrored in the JS
// `thinkingDetailText` helper). These tests pin the new behavior: realistic
// detail survives in full, while pathological detail is still bounded so the
// preview cannot grow without limit.

use formal_ai::naturalize_thinking_step;

// A realistic detail that is long enough to exceed the old 120-char cap yet
// short enough to stay within the new 600-char cap, so it must now render whole.
fn medium_detail() -> String {
    "Summarize the latest deployment report, list every failing health check by \
     region, and recommend the three highest-impact mitigations for the on-call \
     engineer to action first."
        .to_string()
}

#[test]
fn thinking_detail_is_not_clipped_at_120_chars() {
    let detail = medium_detail();
    assert!(
        detail.chars().count() > 120,
        "fixture must exceed the old 120-char cap (was {})",
        detail.chars().count()
    );

    let sentence = naturalize_thinking_step("impulse", &detail);

    // The full detail must survive verbatim into the naturalized sentence...
    assert!(
        sentence.contains(detail.trim()),
        "detail was clipped before reaching the user: {sentence}"
    );
    // ...with no mid-sentence truncation ellipsis (the problem-2 symptom).
    assert!(
        !sentence.contains('…'),
        "unexpected truncation ellipsis on a sub-600-char detail: {sentence}"
    );
}

#[test]
fn thinking_detail_is_still_bounded_above_600_chars() {
    // The cap was raised, not removed: a pathological multi-hundred-char detail
    // is still truncated so the preview stays bounded.
    let detail = "word ".repeat(400); // 2000 chars
    assert!(detail.chars().count() > 600);

    let sentence = naturalize_thinking_step("impulse", &detail);

    assert!(
        sentence.contains('…'),
        "a detail beyond the cap must still be truncated with an ellipsis: {sentence}"
    );
    // The naturalized sentence stays close to the 600-char detail bound plus the
    // small "Read the request: \"…\"." wrapper.
    assert!(
        sentence.chars().count() <= 640,
        "truncated sentence longer than the bound ({}): {sentence}",
        sentence.chars().count()
    );
    assert!(
        !sentence.contains(detail.trim()),
        "the full over-cap detail must not appear verbatim: {sentence}"
    );
}
