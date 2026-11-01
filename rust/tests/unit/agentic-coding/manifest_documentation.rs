//! Install as nested unit source of manifest_documentation.rs; CI only.
use super::derive_manifest_documentation;
use serde_json::Value;
#[test]
fn shared_manifest_documentation_cases() {
    let fixtures: Value = serde_json::from_str(include_str!(
        "../../fixtures/manifest-documentation-cases.json"
    ))
    .expect("fixture JSON");
    for entry in fixtures["cases"].as_array().expect("cases") {
        let text = |key: &str| entry[key].as_str().expect("case string");
        let actual = derive_manifest_documentation(
            text("task"),
            text("owner"),
            text("source"),
            text("guide"),
            Some(&fixtures["forms"]),
        );
        if let Some(reason) = entry["gap"].as_str() {
            assert_eq!(actual["gap"], reason, "{}", entry["name"]);
            continue;
        }
        assert!(actual.get("gap").is_none(), "{}: {actual}", entry["name"]);
        assert_eq!(
            actual["value"].as_array().expect("facts").len(),
            usize::try_from(entry["rows"].as_u64().expect("row count")).expect("bounded count")
        );
        let content = actual["content"].as_str().expect("content");
        assert!(content.starts_with(text("guide")));
        for expected in entry["contains"].as_array().expect("contains") {
            assert!(
                actual["paragraph"]
                    .as_str()
                    .expect("paragraph")
                    .contains(expected.as_str().expect("substring"))
            );
        }
        if let Some(excluded) = entry["excludes"].as_array() {
            for value in excluded {
                assert!(
                    !actual["paragraph"]
                        .as_str()
                        .expect("paragraph")
                        .contains(value.as_str().expect("substring"))
                );
            }
        }
        let repeated = derive_manifest_documentation(
            text("task"),
            text("owner"),
            text("source"),
            content,
            Some(&fixtures["forms"]),
        );
        assert_eq!(repeated["content"], actual["content"]);
    }
}
#[test]
fn refresh_preserves_other_manifest_block() {
    let fixtures: Value = serde_json::from_str(include_str!(
        "../../fixtures/manifest-documentation-cases.json"
    ))
    .expect("fixture JSON");
    let forms = Some(&fixtures["forms"]);
    let first = derive_manifest_documentation(
        "Document dependencies.",
        "package.json",
        r#"{"dependencies":{"shared-kit":"1"}}"#,
        "# Keep 雪\n",
        forms,
    );
    let other = derive_manifest_documentation(
        "Document dependencies.",
        "native/package.json",
        r#"{"dependencies":{"shared-kit":"7"}}"#,
        first["content"].as_str().expect("first"),
        forms,
    );
    let updated = derive_manifest_documentation(
        "Document dependencies.",
        "package.json",
        r#"{"dependencies":{"shared-kit":"2"}}"#,
        other["content"].as_str().expect("other"),
        forms,
    );
    let content = updated["content"].as_str().expect("updated");
    assert!(content.contains("`shared-kit` | `2`"));
    assert!(!content.contains("`shared-kit` | `1`"));
    assert!(content.contains(other["paragraph"].as_str().expect("other paragraph")));
}
