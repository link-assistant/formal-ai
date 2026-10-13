/// Compare immutable legacy task/content operands with a complete observed
/// process receipt while preserving every other field of the whole session.
fn assert_protocol_transition(legacy: &str, observed: &str) {
    let historical: serde_json::Value = serde_json::from_str(legacy).expect("legacy JSON");
    let mut actual: serde_json::Value = serde_json::from_str(observed).expect("observed JSON");
    let steps = actual["steps"].as_array_mut().expect("observed steps");
    let verification = steps.last_mut().expect("observed verification");
    assert_eq!(verification["tool"], "run_command");
    let receipt: serde_json::Value =
        serde_json::from_str(verification["result"].as_str().expect("receipt bytes"))
            .expect("process receipt");
    assert_eq!(receipt["schema"], "command-execution-receipt/v1");
    assert_eq!(receipt["command"], verification["arguments"]["command"]);
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["timed_out"], false);
    assert_eq!(receipt["truncated"], false);
    assert!(receipt["stderr"].is_string());
    assert!(receipt.get("signal").is_none_or(serde_json::Value::is_null));
    let old_verification = historical["steps"].as_array().unwrap().last().unwrap();
    let old_result = old_verification["result"]
        .as_str()
        .expect("historical result");
    let expected_receipt = serde_json::from_str::<serde_json::Value>(old_result).map_or_else(
        |_| {
            assert!(
                !(old_result.trim_start().starts_with('{')
                    && old_result.contains("\"schema\"")
                    && old_result.contains("command-execution-receipt/")),
                "malformed historical process receipt"
            );
            None
        },
        Some,
    );
    if let Some(expected) = expected_receipt.filter(|value| {
        value["schema"]
            .as_str()
            .is_some_and(|schema| schema.starts_with("command-execution-receipt/"))
    }) {
        assert_eq!(expected["schema"], "command-execution-receipt/v1");
        assert_eq!(
            expected["command"],
            old_verification["arguments"]["command"]
        );
        assert_eq!(expected["exit_code"], 0);
        assert_eq!(expected["complete"], true);
        assert_eq!(expected["timed_out"], false);
        assert_eq!(expected["truncated"], false);
        assert!(expected["stderr"].is_string());
        assert!(expected["stdout"].is_string());
        assert!(
            expected
                .get("signal")
                .is_none_or(serde_json::Value::is_null)
        );
        assert_eq!(actual, historical, "every typed session field is preserved");
        return;
    }
    assert_eq!(receipt["stdout"], old_verification["result"]);
    verification["result"] = old_verification["result"].clone();
    assert_eq!(
        actual, historical,
        "every other original session field is preserved"
    );
}
