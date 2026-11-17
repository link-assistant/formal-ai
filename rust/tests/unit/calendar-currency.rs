use formal_ai::FormalAiEngine;

#[test]
fn calendar_and_currency_share_the_worker_acceptance_cases() {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/parity/issue-1589-calendar-currency.json"
    ))
    .unwrap();
    for row in rows.as_array().unwrap() {
        let prompt = row["prompt"].as_str().unwrap();
        let answer = FormalAiEngine.answer(prompt);
        if let Some(reject) = row["reject"].as_str() {
            assert_ne!(answer.intent, reject, "{prompt}: {answer:?}");
            continue;
        }
        assert_eq!(
            answer.answer,
            row["answer"].as_str().unwrap(),
            "{prompt}: {answer:?}"
        );
        assert_eq!(
            answer.intent,
            row["intent"].as_str().unwrap(),
            "{prompt}: {answer:?}"
        );
        assert!(
            answer.answer.contains(row["text"].as_str().unwrap()),
            "{prompt}: {answer:?}"
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link == row["evidence"].as_str().unwrap()),
            "{prompt}: {answer:?}"
        );
        assert!(
            answer
                .thinking_steps
                .iter()
                .any(|step| step.source_event.contains("derivation")
                    && (step.detail.contains('=') || step.detail.contains('×'))),
            "{prompt}: {answer:?}"
        );
        if row["evidence"].as_str().unwrap().starts_with("currency:") {
            for text in ["2026-11-17", "https://www.cbr.ru/"] {
                assert!(answer.answer.contains(text), "{prompt}: {answer:?}");
            }
        }
    }
}
