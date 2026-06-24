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

// Realistic details translated into the other supported UI languages. problem 2
// ("some parts are omitted") must hold for every language, not only English, so
// each fixture exceeds the old 120-char cap yet stays within the new 600-char
// cap and must render whole regardless of script.
fn russian_detail() -> String {
    "Кратко изложите последний отчёт о развёртывании, перечислите все неудачные проверки работоспособности по регионам и порекомендуйте три наиболее важные меры по устранению последствий, которые дежурный инженер должен выполнить в первую очередь.".to_string()
}

fn hindi_detail() -> String {
    "नवीनतम परिनियोजन रिपोर्ट का सारांश प्रस्तुत करें, प्रत्येक विफल स्वास्थ्य जाँच को क्षेत्र के अनुसार सूचीबद्ध करें, और ऑन-कॉल इंजीनियर के लिए सबसे अधिक प्रभाव डालने वाले तीन समाधानों की अनुशंसा करें।".to_string()
}

#[test]
fn thinking_detail_renders_in_full_for_each_supported_language() {
    // The fix must reach every supported UI language (en, ru, hi, zh), not only
    // English: a realistic, sub-600-char detail in any script must survive
    // verbatim with no truncation ellipsis. English is exercised above; this
    // pins Russian (Cyrillic) and Hindi (Devanagari).
    for (language, detail) in [("ru", russian_detail()), ("hi", hindi_detail())] {
        let chars = detail.chars().count();
        assert!(
            chars > 120 && chars <= 600,
            "{language} fixture must clear the old 120-char cap and stay within the new 600-char cap (was {chars})"
        );

        let sentence = naturalize_thinking_step("impulse", &detail);

        assert!(
            sentence.contains(detail.trim()),
            "{language} detail was clipped before reaching the user: {sentence}"
        );
        assert!(
            !sentence.contains('…'),
            "unexpected truncation ellipsis on a sub-600-char {language} detail: {sentence}"
        );
    }
}

#[test]
fn thinking_detail_cap_counts_unicode_scalar_values_not_bytes() {
    // The cap counts Unicode scalar values, not bytes. A Chinese (zh) detail
    // whose UTF-8 byte length exceeds 600 but whose char count stays under 600
    // must render whole; a byte-based cap would wrongly clip multibyte scripts.
    let detail = "总结最新的部署报告并按区域列出每一项失败的健康检查。".repeat(8);

    let chars = detail.chars().count();
    assert!(
        chars <= 600,
        "zh fixture must stay within the 600-char cap (was {chars})"
    );
    assert!(
        detail.len() > 600,
        "zh fixture must exceed 600 bytes to exercise the char-vs-byte distinction (was {} bytes)",
        detail.len()
    );

    let sentence = naturalize_thinking_step("impulse", &detail);

    assert!(
        !sentence.contains('…'),
        "a multibyte detail under the char cap must not be truncated: {sentence}"
    );
    assert!(
        sentence.contains(detail.trim()),
        "the multibyte zh detail must survive verbatim: {sentence}"
    );
}
