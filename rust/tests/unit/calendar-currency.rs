use formal_ai::FormalAiEngine;

#[test]
fn calendar_calculations_share_the_worker_acceptance_cases() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/parity/issue-1589-calendar-currency.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for row in rows.as_array().unwrap() {
        let prompt = row["prompt"].as_str().unwrap();
        let answer = FormalAiEngine.answer(prompt);
        if let Some(reject) = row["reject"].as_str() {
            if answer.intent == reject {
                failures.push(format!("{prompt}: unexpectedly selected {reject}"));
            }
            continue;
        }
        for (field, actual) in [("answer", &answer.answer), ("intent", &answer.intent)] {
            let expected = row[field].as_str().unwrap();
            if actual != expected {
                failures.push(format!(
                    "{prompt}: {field}: expected {expected:?}, got {actual:?}"
                ));
            }
        }
        let text = row["text"].as_str().unwrap();
        if !answer.answer.contains(text) {
            failures.push(format!("{prompt}: missing result {text:?}"));
        }
        // Native evidence addresses content-addressed event records; their payload
        // is not embedded in the address as it is in the JavaScript worker.
        // The exact answer and derivation checks independently verify the value.
        let evidence = row["evidence"].as_str().unwrap();
        let (event_kind, _) = evidence.rsplit_once(':').unwrap();
        let event_prefix = format!("{event_kind}:");
        if !answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with(&event_prefix))
        {
            failures.push(format!(
                "{prompt}: missing native evidence record {event_prefix:?}"
            ));
        }
        if !answer.thinking_steps.iter().any(|step| {
            step.source_event.contains("derivation")
                && (step.detail.contains('=') || step.detail.contains('×'))
        }) {
            failures.push(format!("{prompt}: missing actual arithmetic derivation"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} acceptance failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
