use super::*;

#[test]
fn commit_cue_masking_preserves_independent_authoring_and_word_boundaries() {
    for request in [
        "Implement a parser and haz commit.",
        "Revisa haz committer y haz commit.",
        "Revisa src/haz-committer.mjs y haz commit.",
        "Append \"haz commit\" to notes.txt.",
    ] {
        let messages = vec![ChatMessage::user(request)];
        let planned = match plan_chat_step(&messages, AGENT_TOOLS.as_slice()) {
            Some(AgenticPlan::ToolCalls(planned)) => planned,
            Some(AgenticPlan::Final(answer)) => {
                assert!(answer.contains("MissingContract"), "{request}: {answer}");
                let start = answer.find('{').expect("structured missing contract");
                let receipt: serde_json::Value =
                    serde_json::from_str(&answer[start..]).expect("complete refusal JSON");
                assert_eq!(receipt["reason"], "MissingContract");
                assert_eq!(receipt["goal"], request);
                assert_eq!(receipt["authored"], false);
                assert_eq!(receipt["verified"], false);
                assert!(receipt["missingContracts"].as_array().unwrap().len() >= 2);
                continue;
            }
            other => panic!("{request}: unexpected plan {other:?}"),
        };
        assert!(
            planned
                .iter()
                .all(|call| { call.tool != "bash" || !command_of(call).contains("git commit") }),
            "{request}: {planned:?}"
        );
    }
}
