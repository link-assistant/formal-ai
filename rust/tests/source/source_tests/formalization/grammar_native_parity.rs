use super::*;
#[test]
fn raw_grammar_vocabulary_preserves_case_defaults_and_declared_languages() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/formalization-native-parity.json"
    ))
    .unwrap();
    let parsed = crate::seed::parse_lino(vectors["grammar"].as_str().unwrap());
    let root = grammar_root_from(&parsed).expect("fixture grammar root");
    for vector in vectors["vocabulary"].as_array().unwrap() {
        let expected = vector["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            grammar_vocabulary_from(
                root,
                vector["language"].as_str().unwrap(),
                vector["kind"].as_str().unwrap()
            ),
            expected
        );
    }
    for vector in vectors["structural"].as_array().unwrap() {
        let expected = vector["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            structural_rule_values_from(
                root,
                vector["rule"].as_str().unwrap(),
                vector["kind"].as_str().unwrap()
            ),
            expected
        );
    }
    assert_eq!(grammar_root().unwrap().name, "text-formalization");
    assert_eq!(
        grammar_vocabulary("xx", "numeric-decimal"),
        Vec::<String>::new()
    );
    assert!(grammar_root_from(&crate::seed::parse_lino("unrelated root")).is_none());
}

#[test]
fn task_decomposition_preserves_decoded_ascii_quote_pairs() {
    let values = structural_rule_values("task-decomposition", "quote-pair");
    assert!(values.contains(&"\"\"".to_owned()));
    assert!(values.contains(&"''".to_owned()));
    assert!(values.iter().all(|pair| pair.chars().count() == 2));
}
