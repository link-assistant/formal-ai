use super::*;
#[test]
fn maintained_javascript_goldens_match_canonical_native_conversion() {
    let fixtures: Value =
        serde_json::from_str(include_str!("../fixtures/logs-cli-runtime-parity.json")).unwrap();
    for fixture in fixtures["cases"].as_array().unwrap() {
        let input = fixture["input"].as_str().unwrap();
        let expected = fixture["lino"].as_str().unwrap();
        assert_eq!(
            convert(input, InputFormat::Jsonl, LogFormat::Lino).unwrap(),
            expected
        );
        assert_eq!(
            convert(expected, InputFormat::Lino, LogFormat::Lino).unwrap(),
            expected
        );
        let json = convert(expected, InputFormat::Lino, LogFormat::Json).unwrap();
        let pretty = convert(expected, InputFormat::Lino, LogFormat::Pretty).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&json).unwrap(),
            serde_json::from_str::<Value>(&pretty).unwrap()
        );
    }
}
#[test]
fn malformed_or_partial_streams_never_return_a_prefix() {
    for input in ["{}", "{}\nnot-json\n", "\n", "[\n"] {
        assert!(convert(input, InputFormat::Jsonl, LogFormat::Lino).is_err());
    }
    assert!(
        convert(
            "(formal-ai-log-v1 (null))",
            InputFormat::Lino,
            LogFormat::Json
        )
        .is_err()
    );
    assert!(
        convert(
            "(formal-ai-log-v1 (null))\r\n",
            InputFormat::Lino,
            LogFormat::Json
        )
        .is_err()
    );
}
#[test]
fn empty_stream_and_large_integer_and_float_types_survive() {
    assert_eq!(convert("", InputFormat::Lino, LogFormat::Lino).unwrap(), "");
    assert_eq!(
        convert("", InputFormat::Jsonl, LogFormat::Json).unwrap(),
        "[]\n"
    );
    let line = "{\"integer\":18446744073709551615,\"float\":1.5,\"negative_zero\":-0.0}\n";
    let encoded = convert(line, InputFormat::Jsonl, LogFormat::Lino).unwrap();
    assert_eq!(
        convert(&encoded, InputFormat::Lino, LogFormat::Lino).unwrap(),
        encoded
    );
}
