//! Real native numeric observation bridge against shared JavaScript Number bit witnesses.
use super::{research_length_number, research_number_index};
#[test]
fn decimal_lengths_match_javascript_number_at_safe_and_platform_boundaries() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("coding-research/numeric-boundaries.json")).unwrap();
    for row in vectors["lengths"].as_array().unwrap() {
        let decimal = row["decimal"].as_str().unwrap();
        if let Ok(value) = decimal.parse::<usize>() {
            assert_eq!(
                research_length_number(value).to_bits().to_string(),
                row["bits"].as_str().unwrap()
            );
        }
    }
}
#[test]
fn generated_indices_preserve_zero_and_refuse_fractional_nonfinite_or_overflow() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("coding-research/numeric-boundaries.json")).unwrap();
    for row in vectors["indices"].as_array().unwrap() {
        let value = row["value"].as_str().unwrap().parse::<f64>().unwrap();
        let expected = row["expected"]
            .as_str()
            .and_then(|text| text.parse::<usize>().ok());
        assert_eq!(research_number_index(value), expected, "{row}");
    }
}
