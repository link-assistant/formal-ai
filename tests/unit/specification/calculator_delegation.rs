//! Issue #96: delegate calculator-parsable expressions to link-calculator.
//!
//! These tests cover the formal-ai boundary: natural-language prompt wrappers
//! are stripped in the four currently supported languages, calculator-native
//! expressions are handled by the upstream crate, and local arithmetic remains
//! as a fallback for syntax that calculator does not support yet.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn assert_calculation(prompt: &str, expected_fragments: &[&str]) -> SymbolicAnswer {
    let response = answer(prompt);
    assert_eq!(
        response.intent, "calculation",
        "prompt {prompt:?} should be delegated to calculator, got intent={} answer={}",
        response.intent, response.answer,
    );
    for fragment in expected_fragments {
        assert!(
            response.answer.contains(fragment),
            "prompt {prompt:?} answer should contain {fragment:?}, got {}",
            response.answer,
        );
    }
    response
}

fn assert_calculation_error(prompt: &str, expected_fragments: &[&str]) -> SymbolicAnswer {
    let response = answer(prompt);
    assert_eq!(
        response.intent, "calculation_error",
        "prompt {prompt:?} should report calculator parse failure, got intent={} answer={}",
        response.intent, response.answer,
    );
    for fragment in expected_fragments {
        assert!(
            response.answer.contains(fragment),
            "prompt {prompt:?} answer should contain {fragment:?}, got {}",
            response.answer,
        );
    }
    response
}

#[test]
fn calculator_handles_english_variations() {
    for (prompt, expected) in [
        ("What is 8% of $50?", &["4", "USD"][..]),
        ("Please calculate sqrt(16)", &["4"][..]),
        ("Compute 300000 ms in seconds", &["300"][..]),
        ("Evaluate 741 KB as MB", &["0.741", "MB"][..]),
        ("How much is 10 tons to kg?", &["10000", "kg"][..]),
        ("Solve 2^3", &["8"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_explains_fuzzy_calculate_typo() {
    assert_calculation(
        "Calcualte 2+5050",
        &[
            "Interpreted \"Calcualte\" as \"calculate\".",
            "2+5050 = 5052",
        ],
    );
}

#[test]
fn calculator_fuzzy_prefix_is_not_limited_to_one_spelling() {
    assert_calculation(
        "Calcuate 2+5050",
        &[
            "Interpreted \"Calcuate\" as \"calculate\".",
            "2+5050 = 5052",
        ],
    );
}

#[test]
fn calculator_error_keeps_fuzzy_interpretation() {
    assert_calculation_error(
        "Calcualte 2+",
        &[
            "Interpreted \"Calcualte\" as \"calculate\".",
            "could not evaluate",
        ],
    );
}

#[test]
fn calculator_handles_compact_question_equals_suffix() {
    for prompt in ["2*2+2=?", "2*2+2 = ?"] {
        assert_calculation(prompt, &["2*2+2 = 6"]);
    }
}

#[test]
fn calculator_handles_russian_variations() {
    for (prompt, expected) in [
        ("Сколько будет 2 + 2?", &["4"][..]),
        ("Сколько будет два плюс два?", &["4"][..]),
        ("Посчитай 1000 рублей в долларах", &["USD"][..]),
        ("Вычисли 1 тонна в кг", &["1000", "kg"][..]),
        ("Рассчитай 17 февраля 2027 - 6 месяцев", &["2026-08"][..]),
        ("Сколько будет 300000 ms in seconds?", &["300"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_handles_chinese_variations() {
    for (prompt, expected) in [
        ("计算 2 + 2", &["4"][..]),
        ("1000 克 换成 公斤 是多少?", &["1", "kg"][..]),
        ("请计算 1000 美元 换成 欧元", &["EUR"][..]),
        ("17 二月 2027 - 6 个月 等于多少", &["2026-08"][..]),
        ("1 一月 2027 + 7 天 等于多少", &["2027-01-08"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_handles_hindi_variations() {
    for (prompt, expected) in [
        ("2 + 2 कितना है?", &["4"][..]),
        ("गणना करें 1000 डॉलर में यूरो", &["EUR"][..]),
        ("1000 ग्राम में किलोग्राम कितना है?", &["1", "kg"][..]),
        ("17 फरवरी 2027 - 6 महीने की गणना करें", &["2026-08"][..]),
        ("1 जनवरी 2027 + 7 दिन कितना है?", &["2027-01-08"][..]),
    ] {
        assert_calculation(prompt, expected);
    }
}

#[test]
fn calculator_delegation_is_visible_in_evidence() {
    let response = assert_calculation("What is 8% of $50?", &["4", "USD"]);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:link-calculator"),
        "calculator-backed answers should record the delegated engine: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation:lino:")),
        "calculator-backed answers should preserve the upstream LINO: {:?}",
        response.evidence_links,
    );
}

#[test]
fn local_arithmetic_fallback_keeps_word_operators() {
    let word_response = assert_calculation("What is 10 plus 20 times 3?", &["70"]);
    assert!(
        word_response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:formal-ai-fallback"),
        "word-operator fallback should be observable: {:?}",
        word_response.evidence_links,
    );
}

#[test]
fn calculator_handles_binary_modulo_after_upstream_fix() {
    let modulo_response = assert_calculation("Compute 100 - 25 % 7", &["96"]);
    assert!(
        modulo_response
            .evidence_links
            .iter()
            .any(|link| link == "calculation:engine:link-calculator"),
        "binary modulo should be delegated after link-calculator v0.17.0: {:?}",
        modulo_response.evidence_links,
    );
}

#[test]
fn simple_variable_equations_are_solved_after_calculator_delegation() {
    for (prompt, expected) in [
        ("x*2 = 123", &["x = 61.5"][..]),
        ("Solve x * 2 = 123", &["x = 61.5"][..]),
        ("2 * x + 3 = 11", &["x = 4"][..]),
        ("10 = y / 3 + 1", &["y = 27"][..]),
    ] {
        let response = assert_calculation(prompt, expected);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "calculation:engine:link-calculator"),
            "simple equations should be delegated after link-calculator v0.17.1: {:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn calculator_extraction_does_not_steal_named_entities_with_digits() {
    for prompt in [
        "What is Python 3?",
        "What is GPT-5?",
        "What is Python 3 in programming?",
    ] {
        let response = answer(prompt);
        assert_ne!(
            response.intent, "calculation",
            "named entity prompt {prompt:?} should not be treated as a calculation: {}",
            response.answer,
        );
        assert_ne!(
            response.intent, "calculation_error",
            "named entity prompt {prompt:?} should fall through instead of becoming a calculator error: {}",
            response.answer,
        );
    }
}
