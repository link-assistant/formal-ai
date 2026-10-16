use super::*;
#[test]
fn new_typed_conversations_preserve_legacy_evidence_and_numeric_types() {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-conversation-migration-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).expect("fixture directory");
    let old = DialogConversationLog {
        timestamp_unix_ms: 1,
        dialog_id: "migration".into(),
        request_id: "old".into(),
        messages: vec![json!({"role":"user","content":"legacy provenance"})],
    };
    let new = DialogConversationLog {
        timestamp_unix_ms: 2,
        dialog_id: "migration".into(),
        request_id: "new".into(),
        messages: vec![
            json!({"role":"assistant","content":"typed", "float":1.0,"large":9_007_199_254_740_993_u64}),
        ],
    };
    let legacy_path = directory.join("migration.conversation.jsonl");
    let original = format!("{}\n", serde_json::to_string(&old).expect("legacy fixture"));
    fs::write(&legacy_path, &original).expect("retain legacy");
    let path = write_conversation_record(&directory, &new)
        .expect("typed writer")
        .expect("turns");
    assert_eq!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("lino")
    );
    assert_eq!(
        fs::read_to_string(legacy_path).expect("legacy unchanged"),
        original
    );
    assert_eq!(
        load_conversation_records(&directory, "migration").expect("both readers"),
        vec![old, new]
    );
    assert_eq!(
        latest_conversation_id(&directory).as_deref(),
        Some("migration")
    );
    fs::remove_dir_all(directory).expect("fixture cleanup");
}
#[test]
fn accepted_large_and_deep_conversation_writes_are_readable() {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-large-conversation-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut deep = json!("leaf");
    for _ in 0..65 {
        deep = Value::Array(vec![deep]);
    }
    let record = DialogConversationLog {
        timestamp_unix_ms: 3,
        dialog_id: "large".into(),
        request_id: "large-request".into(),
        messages: vec![
            json!({"role":"user", "content":"x".repeat(2 * 1024 * 1024), "nested":deep}),
        ],
    };
    let path = write_conversation_record(&directory, &record)
        .expect("large deep write")
        .expect("turns");
    assert!(fs::metadata(path).expect("persisted row").len() > 1024 * 1024);
    assert_eq!(
        load_conversation_records(&directory, "large").expect("read accepted row"),
        vec![record]
    );
    assert_eq!(latest_conversation_id(&directory).as_deref(), Some("large"));
    fs::remove_dir_all(directory).expect("fixture cleanup");
}
