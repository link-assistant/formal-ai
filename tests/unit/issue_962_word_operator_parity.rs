//! Issue #962: Hindi/Chinese word-operator arithmetic fell to the unknown
//! handler while English/Russian succeeded.
//!
//! Doctrine (README / USER-JOURNEYS): "every operation is recognized equally
//! across en | ru | hi | zh". Symbolic "2 + 2" already worked in all four
//! languages, but the spelled infix operators did not: the
//! `arithmetic_operation` meanings in `data/seed/meanings-calculator.lino`
//! carried only "जोड़"/"加上" and not the bare infix forms "जमा"/"加", and the
//! Hindi `calculation_result_query` cue list carried "कितना है" but not the
//! equally common "कितना होता है". Both gaps are fixed in the seed, so this
//! file pins the three live-verified prompts from the issue alongside their
//! already-working English and Russian counterparts.

use formal_ai::FormalAiEngine;

fn assert_answers(prompt: &str, expected: &str) {
    let response = FormalAiEngine.answer(prompt);
    assert_eq!(
        response.intent, "calculation",
        "prompt {prompt:?} should resolve to calculation, got intent={:?} answer={:?}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains(expected),
        "prompt {prompt:?} answer {:?} should contain {expected:?}",
        response.answer,
    );
}

#[test]
fn word_operator_addition_answers_in_all_four_languages() {
    for prompt in [
        "What is 2 plus 2?",
        "Сколько будет 2 плюс 2?",
        "2 जोड़ 2 कितना होता है?",
        "2 जमा 2 कितना होता है?",
        "2 加 2 等于多少?",
    ] {
        assert_answers(prompt, "4");
    }
}

#[test]
fn symbolic_addition_stays_answered_in_all_four_languages() {
    for prompt in [
        "What is 2 + 2?",
        "Сколько будет 2 + 2?",
        "2 + 2 कितना है?",
        "2 + 2 等于多少?",
    ] {
        assert_answers(prompt, "4");
    }
}

/// The holistic pass the issue asked for: minus/times/divide carry the same
/// infix-word gap in Hindi and Chinese, so spot-check every operator, not just
/// the two reported addition words.
#[test]
fn other_word_operators_answer_in_hindi_and_chinese() {
    for (prompt, expected) in [
        ("4 घटा 2 कितना होता है?", "2"),
        ("3 गुणा 2 कितना होता है?", "6"),
        ("6 भाग 2 कितना होता है?", "3"),
        ("6 बटा 2 कितना होता है?", "3"),
        ("4 减 2 等于多少?", "2"),
        ("3 乘 2 等于多少?", "6"),
        ("6 除 2 等于多少?", "3"),
    ] {
        assert_answers(prompt, expected);
    }
}
