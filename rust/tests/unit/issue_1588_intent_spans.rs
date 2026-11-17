use formal_ai::FormalAiEngine;

#[test]
fn document_intent_and_bound_quantities_match_worker_fixtures() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/parity/issue-1588-intent-spans.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let prompt = row["prompt"].as_str().unwrap();
        let result = FormalAiEngine.answer(prompt);
        let expected = row["kind"].as_str().unwrap();
        match expected {
            "document" => assert!(
                !["unit_conversion", "calendar_create_event"].contains(&result.intent.as_str()),
                "{prompt}: {result:?}"
            ),
            "conversion" => assert_eq!(result.intent, "unit_conversion", "{prompt}: {result:?}"),
            "date" => {
                assert_eq!(
                    result.intent, "calendar_date_offset",
                    "{prompt}: {result:?}"
                );
                let link = format!("calendar:result_date:{}", row["result"].as_str().unwrap());
                assert!(
                    result.evidence_links.contains(&link),
                    "{prompt}: {result:?}"
                );
            }
            _ => panic!("unknown fixture"),
        }
        if let Some(expected) = row["answer"].as_str() {
            assert_eq!(result.answer, expected, "{prompt}");
        }
        if let Some(text) = row["text"].as_str() {
            assert!(result.answer.contains(text), "{prompt}: {result:?}");
        }
    }
}
