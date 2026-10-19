use super::*;
#[test]
fn actual_tracing_events_reuse_spans_and_preserve_types_without_stdout() {
    let rows = Arc::new(Mutex::new(Vec::new()));
    let observed = rows.clone();
    let subscriber = LinoSubscriber {
        spans: Mutex::new(HashMap::new()),
        maximum_level: Some(tracing::Level::INFO),
        sink: Arc::new(move |level, target, message, fields| {
            observed
                .lock()
                .unwrap()
                .push((level.to_owned(), target.to_owned(), message, fields));
        }),
    };
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!(
            "request",
            session = "test-session",
            password = "not-for-output"
        );
        span.in_scope(|| {
            tracing::info!(answer = 42_u64, ready = true, ratio = 0.5_f64, "first");
            tracing::info!(answer = 43_u64, "second");
        });
    });
    let rows = rows.lock().unwrap();
    let events: Vec<_> = rows
        .iter()
        .filter(|row| row.3["kind"] == "tracing_event")
        .collect();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].3["span_id"], events[1].3["span_id"]);
    assert!(events[0].3["span_id"].is_u64());
    assert_eq!(events[0].3["fields"]["answer"], 42);
    assert_eq!(events[0].3["fields"]["ready"], true);
    assert!(rows.iter().all(|row| {
        !serde_json::to_string(&row.3)
            .unwrap()
            .contains("not-for-output")
    }));
    assert_eq!(
        rows.iter()
            .filter(|row| row.3["kind"] == "tracing_span")
            .count(),
        1
    );
    for row in rows.iter() {
        let line = structured_log::encode(&row.3).unwrap();
        assert_eq!(structured_log::decode(&line).unwrap(), row.3);
    }
}
#[test]
fn disabled_trace_payload_never_formats() {
    struct Forbidden;
    impl std::fmt::Debug for Forbidden {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("disabled payload formatted")
        }
    }
    tracing::subscriber::with_default(
        LinoSubscriber {
            maximum_level: Some(tracing::Level::INFO),
            ..LinoSubscriber::default()
        },
        || tracing::trace!(payload=?Forbidden,"disabled_trace"),
    );
}
