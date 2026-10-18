use super::source_fetch_observation;
use crate::protocol::ChatMessage;
use serde_json::Value;

#[test]
fn observed_source_metadata_matches_shared_vectors_without_promoting_authority() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("source-fetch-provenance.json")).unwrap();
    for row in rows {
        let messages: Vec<ChatMessage> = row["messagesJson"].as_str().map_or_else(
            || serde_json::from_value(row["messages"].clone()).unwrap(),
            |source| serde_json::from_str(source).unwrap(),
        );
        let start = row["start"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .unwrap();
        let observation = source_fetch_observation(
            &messages,
            start,
            row["url"].as_str().unwrap(),
            row["text"].as_str().unwrap(),
        );
        assert_eq!(observation, row["expected"], "{}", row["id"]);
        assert_eq!(observation["authority"], "Unknown");
    }
}

#[test]
fn canonical_source_capture_policy_vectors() {
    use super::super::super::coding_research::kernel as policy;
    let fixture: Value = serde_json::from_str(include_str!("source-fetch-policy.json")).unwrap();
    for row in fixture["policyCases"].as_array().unwrap() {
        let arguments = row["arguments"].as_array().unwrap();
        let boolean = |index: usize| arguments[index].as_bool().unwrap();
        let number = |index: usize| arguments[index].as_f64().unwrap();
        let actual = match row["function"].as_str().unwrap() {
            "sourceCaptureUniqueAccepted" => policy::source_capture_unique_accepted(number(0)),
            "sourceCaptureDeclarationAccepted" => policy::source_capture_declaration_accepted(
                boolean(0),
                boolean(1),
                boolean(2),
                number(3),
            ),
            "sourceCaptureReceiptIdentityAccepted" => {
                policy::source_capture_receipt_identity_accepted(number(0), boolean(1), boolean(2))
            }

            "sourceCaptureAllObserved" => {
                policy::source_capture_all_observed(boolean(0), boolean(1), boolean(2))
            }

            "sourceCaptureTimestampTextAccepted" => policy::source_capture_timestamp_text_accepted(
                boolean(0),
                boolean(1),
                boolean(2),
                number(3),
            ),

            "sourceCaptureTrustAccepted" => policy::source_capture_trust_accepted(
                boolean(0),
                number(1),
                boolean(2),
                number(3),
                boolean(4),
                number(5),
            ),
            "sourceCaptureLicenceAccepted" => policy::source_capture_licence_accepted(
                boolean(0),
                number(1),
                boolean(2),
                number(3),
                boolean(4),
                number(5),
            ),
            _ => panic!("UnknownCanonicalPolicyVector"),
        };
        assert_eq!(actual, row["expected"].as_bool().unwrap(), "{}", row["id"]);
    }
}

#[test]
fn held_out_source_capture_observer_vectors() {
    let fixture: Value = serde_json::from_str(include_str!("source-fetch-policy.json")).unwrap();
    for row in fixture["observerCases"].as_array().unwrap() {
        let messages: Vec<ChatMessage> = serde_json::from_value(row["messages"].clone()).unwrap();
        let actual = source_fetch_observation(
            &messages,
            0,
            row["url"].as_str().unwrap(),
            row["text"].as_str().unwrap(),
        );
        assert_eq!(actual, row["expected"], "{}", row["id"]);
    }
}
