//! Regression coverage for issue #841's published TUI integration metadata.

use std::collections::BTreeSet;

use formal_ai::client_contract_learning::{
    ClientContractObservation, learn_client_contracts,
};
use formal_ai::seed::{client_integrations, response_for};

#[test]
fn invalid_typed_json_settings_have_messages_for_every_supported_language() {
    let cases = [
        ("en", "invalid typed JSON setting"), // English
        ("ru", "недопустимая типизированная настройка JSON"),
        ("hi", "अमान्य टाइप की गई JSON सेटिंग"),
        ("zh", "无效的带类型 JSON 设置"),
    ];

    for (language, expected_text) in cases {
        let response = response_for("client_integration_invalid_typed_json_setting", language)
            .unwrap_or_else(|| panic!("missing {language} invalid-setting response"));
        assert!(response.contains(expected_text), "{language}: {response}");
        assert!(response.contains("{rendered}"), "{language}: {response}");
        assert!(response.contains("{error}"), "{language}: {response}");
    }
}

#[test]
fn repeated_tui_captures_propose_only_stable_human_gated_contract_facts() {
    let observations = [
        r#"{
            "client_id": "opencode",
            "capability": "tui_replay",
            "task_wording": "Find the hive-mind-control center folder on my Desktop",
            "delivery": "tool_call",
            "evidence": "first/recording.svg",
            "observed_contract": {
                "tui_initial_geometry": ["80x30"],
                "tui_artifact": ["recording.gif", "recording.svg"],
                "tui_renderer_feature": ["css_keyframes", "one_capture_only"]
            }
        }"#,
        r#"{
            "client_id": "opencode",
            "capability": "tui_replay",
            "task_wording": "Locate and verify my Desktop hive-mind-control center directory",
            "delivery": "tool_call",
            "evidence": "second/recording.svg",
            "observed_contract": {
                "tui_initial_geometry": ["80x30"],
                "tui_artifact": ["recording.gif", "recording.svg"],
                "tui_renderer_feature": ["css_keyframes"]
            }
        }"#,
    ]
    .map(|json| {
        serde_json::from_str::<ClientContractObservation>(json)
            .expect("TUI observation should deserialize")
    });

    let report = learn_client_contracts(&observations, &client_integrations());
    let proposed = report
        .proposals
        .iter()
        .map(|proposal| (proposal.field.as_str(), proposal.value.as_str()))
        .collect::<BTreeSet<_>>();

    assert_eq!(report.independently_worded_groups, 1);
    assert!(report.awaiting_human_review);
    assert_eq!(
        proposed,
        BTreeSet::from([
            ("tui_artifact", "recording.gif"),
            ("tui_artifact", "recording.svg"),
            ("tui_initial_geometry", "80x30"),
            ("tui_renderer_feature", "css_keyframes"),
        ])
    );
    assert!(!report.links_notation().contains("one_capture_only"));
    assert!(report.links_notation().contains("awaiting_human_review"));
}
