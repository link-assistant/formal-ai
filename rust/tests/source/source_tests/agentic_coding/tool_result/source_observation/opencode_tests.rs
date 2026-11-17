use super::*;

#[test]
fn shared_opencode_read_envelopes_preserve_source_and_provenance() {
    let fixtures: Value = serde_json::from_str(include_str!(
        "../../../../../../../data/parity/issue-1580-opencode-read-envelope.json"
    ))
    .unwrap();
    for row in fixtures.as_array().unwrap() {
        let raw = row["raw"].as_str().unwrap();
        let observed = source_read_observation(
            raw,
            row["failed"].as_bool().unwrap_or(false),
            row.get("metadata"),
            row["path"].as_str().unwrap(),
        );
        assert_eq!(
            observed.source.as_deref(),
            row["source"].as_str(),
            "{}",
            row["id"]
        );
        assert_eq!(
            observed.complete,
            row["complete"].as_bool().unwrap(),
            "{}",
            row["id"]
        );
    }
}
