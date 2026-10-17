use super::verification_receipt_gap;
#[test]
fn recognized_counter_lines_cannot_hide_malformed_or_duplicate_evidence() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("coding-research/tap-counter-lines.json"))
            .expect("declarative counter vectors");
    for case in cases.as_array().expect("counter vector list") {
        let payload = case["payload"].as_str().expect("counter payload");
        for raw in [
            format!("Output: {payload}\nExit Code: 0"),
            serde_json::json!({"output":payload,"exit_code":0}).to_string(),
        ] {
            assert_eq!(
                verification_receipt_gap(&raw).as_deref(),
                case["expected"].as_str(),
                "{}",
                case["id"]
            );
        }
    }
}
