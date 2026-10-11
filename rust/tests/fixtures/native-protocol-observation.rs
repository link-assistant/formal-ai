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
    let old_verification = historical["steps"].as_array().unwrap().last().unwrap();
    assert_eq!(receipt["stdout"], old_verification["result"]);
    verification["result"] = old_verification["result"].clone();
    assert_eq!(
        actual, historical,
        "every other original session field is preserved"
    );
}
