use super::*;
use crate::formalization::text_statements::clause_statement;
#[test]
fn canonical_numbers_and_independent_boundaries_round_trip() {
    for (text, language) in [
        ("695,000千米", "zh"),
        ("3,827⋅1026", "ru"),
        ("225—250", "ru"),
        ("1：397", "zh"),
        ("१४,९६,००,००० किलोमीटर", "hi"),
    ] {
        let statement = clause_statement(text, language, None, false);
        assert!(fact_survives(&statement), "{text}");
    }
}
#[test]
fn logical_grouping_retains_source_word_counts_unknowns_and_byte_spans() {
    for (text, value, components) in [
        ("987 654 blarg", "987654", 2),
        ("987654 blarg", "987654", 1),
        ("9,007,199,254,740,993 blarg", "9007199254740993", 1),
    ] {
        let statement = clause_statement(text, "en", None, false);
        assert_eq!(statement_terms(&statement).len(), 2);
        assert_eq!(source_term_count(&statement), components + 1);
        assert_eq!(statement.unknown, ["blarg"]);
        let quantity = &statement.dialogue.quantities[0];
        assert_eq!(quantity.value, value);
        assert_eq!(quantity.source_components.len(), components);
        for component in &quantity.source_components {
            assert_eq!(&text[component.start..component.end], component.surface);
        }
    }
}
