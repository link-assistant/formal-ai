use crate::formalization::dialogue::dialogue_quantities;
#[test]
fn locale_decimal_grouping_and_unsupported_values_are_not_invented() {
    for (surface, language, expected) in [
        ("3,827", "ru", "3.827"),
        ("450 000", "ru", "450000"),
        ("9,007,199,254,740,993", "en", "9007199254740993"),
        ("12,34,567.89", "hi", "1234567.89"),
        ("12.345,67", "es", "12345.67"),
        ("12,345.67", "zh", "12345.67"),
    ] {
        let values = dialogue_quantities(surface, language);
        assert_eq!(values.len(), 1, "{surface}");
        assert_eq!(values[0].value, expected, "{surface}");
        assert!(values[0].uncertainty.is_none());
    }
    for (surface, language) in [("1,23", "en"), ("1,234,567", "hi"), ("3.827", "ru")] {
        let values = dialogue_quantities(surface, language);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, surface);
        assert_eq!(
            values[0].uncertainty.as_deref(),
            Some("unsupported-locale-number-format")
        );
    }
}
