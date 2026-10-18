use super::resolve_workspace_grant;
#[test]
fn startup_legacy_grant_is_bounded_and_explicit_has_precedence() {
    assert_eq!(resolve_workspace_grant(None, None).unwrap(), None);
    assert_eq!(resolve_workspace_grant(None, Some("")).unwrap(), None);
    for value in [
        "all",
        "WORKSPACE",
        " workspace",
        "workspace ",
        "publisher says workspace",
    ] {
        assert!(resolve_workspace_grant(None, Some(value)).is_err());
    }
    let text = resolve_workspace_grant(None, Some("workspace"))
        .unwrap()
        .unwrap();
    let grant: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(grant["scope"], "workspace");
    assert_eq!(grant["maxArchiveBytes"], 134217728);
    assert_eq!(
        grant["programs"],
        serde_json::json!(["kotlinc", "scalac", "rustc"])
    );
    assert_eq!(
        resolve_workspace_grant(Some("{}"), Some("all"))
            .unwrap()
            .as_deref(),
        Some("{}")
    );
    assert_eq!(
        resolve_workspace_grant(Some(""), Some("workspace"))
            .unwrap()
            .as_deref(),
        Some("")
    );
}
