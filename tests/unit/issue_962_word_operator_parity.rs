//! Issue #962: Hindi/Chinese word-operator arithmetic fell to the unknown
//! handler while English/Russian succeeded.
//!
//! Doctrine (README / USER-JOURNEYS): "every operation is recognized equally
//! across en | ru | hi | zh". Symbolic `2 + 2` already worked in all four
//! languages, but the spelled infix operators did not, for two separate
//! reasons in `data/seed/meanings-calculator.lino`:
//!
//! 1. the `arithmetic_operation` meanings carried only the compound surfaces
//!    (`जोड़`, `加上`, `减去`, `乘以`, `除以`, `भाग`) and not the bare infix
//!    forms speakers type (`जमा`, `加`, `减`, `乘`, `除`, `बटा`), so
//!    `contains_word_operator` saw no operator and the prompt was rejected as
//!    prose; and
//! 2. the Hindi `calculation_result_query` cue list carried `कितना है` but not
//!    `कितना होता है` — cues are matched as phrases, not stems — so even the
//!    already-seeded `जोड़` failed on the exact phrasing the issue reported.
//!
//! Every case below spells out the full answer string rather than a fragment,
//! so the file doubles as documentation of what the engine actually says.

use formal_ai::FormalAiEngine;

/// Assert the prompt is routed to the calculator and produces exactly `answer`.
fn assert_exact_calculation(prompt: &str, answer: &str) {
    let response = FormalAiEngine.answer(prompt);
    assert_eq!(
        response.intent, "calculation",
        "prompt {prompt:?} should resolve to calculation, got intent={:?} answer={:?}",
        response.intent, response.answer,
    );
    assert_eq!(
        response.answer, answer,
        "prompt {prompt:?} should answer exactly {answer:?}",
    );
}

/// The three prompts reported in the issue, next to the two that already
/// worked. All five ask the same question and all five now answer it.
#[test]
fn word_operator_addition_answers_in_all_four_languages() {
    for (prompt, answer) in [
        ("What is 2 plus 2?", "2 plus 2 = 4"),
        ("Сколько будет 2 плюс 2?", "2 плюс 2 = 4"),
        ("2 जोड़ 2 कितना होता है?", "2 जोड़ 2 = 4"),
        ("2 जमा 2 कितना होता है?", "2 जमा 2 = 4"),
        ("2 加 2 等于多少?", "2 加 2 = 4"),
    ] {
        assert_exact_calculation(prompt, answer);
    }
}

/// The symbolic forms worked before the fix in all four languages; pinning them
/// here means a future vocabulary change cannot regress them.
#[test]
fn symbolic_addition_stays_answered_in_all_four_languages() {
    for (prompt, answer) in [
        ("What is 2 + 2?", "2 + 2 = 4"),
        ("Сколько будет 2 + 2?", "2 + 2 = 4"),
        ("2 + 2 कितना है?", "2 + 2 = 4"),
        ("2 + 2 等于多少?", "2 + 2 = 4"),
    ] {
        assert_exact_calculation(prompt, answer);
    }
}

/// The holistic pass the issue asked for: minus, times and divide carried the
/// identical infix-vs-compound gap, so every operator is spot-checked in both
/// Hindi and Chinese rather than only the two reported addition words.
#[test]
fn other_word_operators_answer_in_hindi_and_chinese() {
    for (prompt, answer) in [
        ("4 घटा 2 कितना होता है?", "4 घटा 2 = 2"),
        ("3 गुणा 2 कितना होता है?", "3 गुणा 2 = 6"),
        ("6 भाग 2 कितना होता है?", "6 भाग 2 = 3"),
        ("6 बटा 2 कितना होता है?", "6 बटा 2 = 3"),
        ("4 减 2 等于多少?", "4 减 2 = 2"),
        ("3 乘 2 等于多少?", "3 乘 2 = 6"),
        ("6 除 2 等于多少?", "6 除 2 = 3"),
    ] {
        assert_exact_calculation(prompt, answer);
    }
}
