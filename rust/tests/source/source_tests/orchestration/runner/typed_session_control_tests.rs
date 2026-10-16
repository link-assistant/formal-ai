use super::*;
#[test]
fn native_session_control_preserves_legacy_identity_and_requires_control_schema() {
    let session =
        serde_json::json!({"id":"ses_control", "resume_command":"agent --resume ses_control"});
    let prefix = crate::dialog_log::structured_log::ORCHESTRATION_SESSION_PREFIX;
    let message = format!("{prefix}{session}");
    let line = crate::dialog_log::structured_log::encode(&serde_json::json!({
        "schema":"formal-ai-control-v1", "kind":"orchestration_session", "message":message
    }))
    .expect("typed machine evidence");
    let typed = parse_native_session(&line, "", None).expect("typed native session");
    let legacy = parse_native_session(&message, "", None).expect("legacy native session");
    assert_eq!(typed.id, legacy.id);
    assert_eq!(typed.resume_command, legacy.resume_command);
    for (schema, kind) in [
        ("formal-ai-diagnostic-v1", "orchestration_session"),
        ("formal-ai-control-v1", "other"),
    ] {
        let line = crate::dialog_log::structured_log::encode(
            &serde_json::json!({"schema":schema,"kind":kind,"message":message}),
        )
        .expect("unrelated typed record");
        assert!(parse_native_session(&line, "", None).is_none());
    }
}
