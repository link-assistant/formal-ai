use crate::formalization::text_statements::{clause_statement, statement_terms};
use crate::translation::round_trip::render_term;
#[test]
fn target_locale_rendering_preserves_numeric_value() {
    for (surface, source, target, expected) in [
        ("3,827", "ru", "en", "3.827"),
        ("1.25", "en", "ru", "1,25"),
        ("12.345,67", "es", "en", "12345.67"),
        ("9,007,199,254,740,993", "en", "ru", "9007199254740993"),
    ] {
        let statement = clause_statement(surface, source, None, false);
        let terms = statement_terms(&statement);
        assert_eq!(render_term(terms[0], source, target).target, expected);
        let translated = clause_statement(expected, target, None, false);
        assert_eq!(statement_terms(&translated)[0].id, terms[0].id);
    }
}
