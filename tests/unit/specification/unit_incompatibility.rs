//! Unit-incompatibility regressions for dimensionally incompatible measures.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn russian_meters_in_kilogram_returns_unit_incompatibility() {
    let response = answer("Сколько метров в килограмме?");
    assert_eq!(
        response.intent, "unit_incompatibility",
        "mixing length and mass units must not fall through to unknown: {:?}",
        response.answer,
    );
    assert!(
        response.answer.contains("length") || response.answer.contains("длин"),
        "answer should mention the length dimension: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("mass") || response.answer.contains("масс"),
        "answer should mention the mass dimension: {}",
        response.answer,
    );
}

#[test]
fn incompatible_length_mass_unit_variations_return_unit_incompatibility() {
    for (language, prompt) in [
        ("English", "How many meters are in a kilogram?"),
        ("English", "How many metres are in 1 kg?"),
        ("Russian", "Сколько метров в кг?"),
        ("Russian", "Сколько килограммов в метре?"),
        ("Hindi", "एक किलोग्राम में कितने मीटर हैं?"),
        ("Chinese", "5千克 多少 米？"),
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "unit_incompatibility",
            "{language} prompt {prompt:?} must explain incompatible unit dimensions: {}",
            response.answer,
        );
        assert!(
            response.answer.contains("length"),
            "{language} prompt {prompt:?} should mention length: {}",
            response.answer,
        );
        assert!(
            response.answer.contains("mass"),
            "{language} prompt {prompt:?} should mention mass: {}",
            response.answer,
        );
    }
}
