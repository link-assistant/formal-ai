use super::*;
use crate::formalization::text_statements::{formalize_text, statement_identity};
use crate::summarization::dependency::restates;
#[test]
fn question_currency_and_units_do_not_collapse_or_disappear() {
    for (one, two, language) in [
        (
            "You are located on Earth.",
            "You are located on Earth?",
            "en",
        ),
        ("450000 ₽", "450000 $", "ru"),
        ("3 years", "3 months", "en"),
    ] {
        let a = formalize_text(one, language).remove(0);
        let b = formalize_text(two, language).remove(0);
        assert_ne!(statement_identity(&a), statement_identity(&b));
        assert_ne!(statement_notation(&a), statement_notation(&b));
        assert!(!restates(&a, &b) && !restates(&b, &a));
        assert!(fact_survives(&a) && fact_survives(&b));
    }
}
#[test]
fn grouped_spelling_is_not_meaning_and_forged_currency_fails_survival() {
    let a = formalize_text("987 654 ₽", "ru").remove(0);
    let b = formalize_text("987654 ₽", "ru").remove(0);
    assert_eq!(statement_identity(&a), statement_identity(&b));
    assert!(restates(&a, &b));
    let mut changed = a.clone();
    changed.dialogue.quantities[0].currency = Some("USD".to_owned());
    assert_ne!(statement_identity(&a), statement_identity(&changed));
    assert!(!fact_survives(&changed));
}
