//! Regression coverage for issue #841's published TUI integration metadata.

use formal_ai::seed::response_for;

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
