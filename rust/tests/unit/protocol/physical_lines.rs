//! The shared physical-line vectors are independently checked against Rust std.
#[test]
fn shared_physical_lines_vectors_match_standard_library() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/physical-lines/vectors.json"))
            .expect("shared physical-lines fixture");
    for row in fixture["cases"].as_array().expect("cases") {
        let source = row["source"].as_str().expect("source");
        let actual: Vec<&str> = source.lines().collect();
        let expected: Vec<&str> = row["expected"]
            .as_array()
            .expect("expected")
            .iter()
            .map(|value| value.as_str().expect("line"))
            .collect();
        assert_eq!(actual, expected, "{}", row["id"]);
    }
}
