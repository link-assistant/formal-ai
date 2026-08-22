use std::ffi::OsStr;

use formal_ai::gemini::{gemini_model_metadata, vertex_model_list};
use formal_ai::handle_api_request;

#[test]
fn models_report_real_disk_context_and_memory_usage() {
    // Issue #1039: how far the reported free-space reading may sit outside the
    // pair this test brackets it with. See the assertion below.
    const DISK_SAMPLE_TOLERANCE_BYTES: u64 = 512 * 1024 * 1024;

    let dir =
        std::env::temp_dir().join(format!("formal-ai-context-capacity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp memory directory");
    let memory_path = dir.join("memory.lino");
    std::fs::write(&memory_path, vec![b'x'; 4_096]).expect("write memory fixture");
    let memory_dir = dir.join("memory-store");
    std::fs::create_dir_all(memory_dir.join("nested")).expect("memory directory");
    std::fs::write(memory_dir.join("memory.lino"), vec![b'x'; 100]).expect("lino file");
    std::fs::write(memory_dir.join("nested/event-log-1"), vec![b'x'; 50]).expect("event log");
    std::fs::write(memory_dir.join("ignored.txt"), vec![b'x'; 1_000]).expect("ignored file");

    let free_before = fs2::available_space(&dir).expect("free space before request");

    // The store and the byte average are read from the environment on every
    // request, so the three configurations this test compares are three scopes
    // rather than three assignments. Edition 2024 made `std::env::set_var`
    // unsafe -- a test binary is multi-threaded and the environment is not --
    // and this crate forbids unsafe code, so `temp-env` scopes them instead.
    // The scopes nest, and its lock is reentrant, so the whole sequence is held
    // against the rest of the suite the way the hand-held mutex used to hold it.
    let (response, gemini, vertex, anthropic_response, configured_response, directory_capacity) =
        temp_env::with_vars(
            [
                ("FORMAL_AI_MEMORY_PATH", Some(memory_path.as_os_str())),
                ("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", None),
            ],
            || {
                let response = handle_api_request("GET", "/v1/models", "");
                let gemini = gemini_model_metadata("models/formal-ai");
                let vertex = vertex_model_list("test-project", "test-location");
                let anthropic_response = handle_api_request(
                    "POST",
                    "/v1/messages",
                    r#"{"model":"formal-ai","max_tokens":32,"messages":[{"role":"user","content":"2 + 2"}]}"#,
                );
                let configured_response =
                    temp_env::with_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", Some("4"), || {
                        handle_api_request("GET", "/v1/models", "")
                    });
                let directory_capacity = temp_env::with_vars(
                    [
                        ("FORMAL_AI_MEMORY_PATH", Some(memory_dir.as_os_str())),
                        ("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", Some(OsStr::new("2"))),
                    ],
                    || {
                        formal_ai::context_capacity::ContextCapacity::current()
                            .expect("directory-backed context capacity")
                    },
                );
                (
                    response,
                    gemini,
                    vertex,
                    anthropic_response,
                    configured_response,
                    directory_capacity,
                )
            },
        );

    let free_after = fs2::available_space(&dir).expect("free space after request");
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).expect("models JSON");
    let model = &json["models"][0];
    let context = &model["context"];
    let disk_free = context["disk_free_bytes"]
        .as_u64()
        .expect("disk_free_bytes");
    // Issue #1039: the reported figure is a *third* live reading of a shared
    // filesystem, taken between the two this test brackets it with. Nothing
    // stops another process on the runner from writing between any two of them,
    // so requiring the middle reading to land inside the outer pair is a race
    // rather than an invariant. It lost that race on `macOS Core Tests / Run
    // macOS core slice 8/16` of run 32572106023, on a pull request that touches
    // no disk code at all.
    //
    // What the assertion is really for is that the server measured *this*
    // filesystem and reported it, rather than returning a placeholder or the
    // wrong volume -- so it is checked against the same readings with a
    // tolerance for concurrent writes. Ordinary CI churn moves free space by
    // kilobytes to a few megabytes between samples; a wrong volume or a stub
    // value is off by orders of magnitude and still fails.
    let lowest = free_after
        .min(free_before)
        .saturating_sub(DISK_SAMPLE_TOLERANCE_BYTES);
    let highest = free_after
        .max(free_before)
        .saturating_add(DISK_SAMPLE_TOLERANCE_BYTES);
    assert!(
        (lowest..=highest).contains(&disk_free),
        "reported disk_free_bytes {disk_free} is outside [{lowest}, {highest}], \
         bracketed by readings {free_before} and {free_after} with a \
         {DISK_SAMPLE_TOLERANCE_BYTES}-byte tolerance for concurrent writes. \
         A gap this large means the wrong filesystem was measured, not runner \
         churn."
    );
    assert_eq!(context["memory_used_bytes"], 4_096);
    assert_eq!(context["avg_utf8_bytes_per_char"], 2);
    assert_eq!(context["context_used_tokens"], 2_048);
    assert_eq!(context["context_window_tokens"], disk_free / 2);
    assert_eq!(model["context_window"], context["context_window_tokens"]);
    assert_eq!(
        model["context_window_tokens"],
        context["context_window_tokens"]
    );
    assert_ne!(model["context_window"], 60_000);
    let expected = formal_ai::context_capacity::ContextCapacity::from_bytes(disk_free, 4_096, 2);
    let reported_used_fraction = context["context_used_fraction"]
        .as_f64()
        .expect("context_used_fraction");
    assert!(
        (reported_used_fraction - expected.context_used_fraction).abs()
            <= expected.context_used_fraction.abs() * f64::EPSILON * 2.0
    );
    assert_eq!(
        gemini["inputTokenLimit"],
        gemini["context"]["context_window_tokens"]
    );
    assert_eq!(gemini["context"]["memory_used_bytes"], 4_096);
    let vertex_model = &vertex["publisherModels"][0];
    assert_eq!(
        vertex_model["inputTokenLimit"],
        vertex_model["context"]["context_window_tokens"]
    );
    assert_eq!(vertex_model["context"]["memory_used_bytes"], 4_096);
    assert_eq!(anthropic_response.status_code, 200);
    let anthropic: serde_json::Value =
        serde_json::from_str(&anthropic_response.body).expect("Anthropic JSON");
    assert!(
        anthropic["context"]["context_window_tokens"]
            .as_u64()
            .is_some_and(|value| value > 0)
    );
    assert_eq!(anthropic["context"]["avg_utf8_bytes_per_char"], 2);
    let configured: serde_json::Value =
        serde_json::from_str(&configured_response.body).expect("configured models JSON");
    let configured_context = &configured["models"][0]["context"];
    assert_eq!(configured_context["avg_utf8_bytes_per_char"], 4);
    assert_eq!(
        configured_context["context_window_tokens"],
        configured_context["disk_free_bytes"].as_u64().unwrap() / 4
    );
    assert_eq!(directory_capacity.memory_used_bytes, 150);
    assert_eq!(directory_capacity.context_used_tokens, 75);
    assert!(json.get("cost").is_none());
}

#[test]
fn configured_utf8_average_scales_capacity_and_usage() {
    let capacity = formal_ai::context_capacity::ContextCapacity::from_bytes(8_000, 400, 4);
    assert_eq!(capacity.context_window_tokens, 2_000);
    assert_eq!(capacity.context_used_tokens, 100);
    assert!((capacity.context_used_fraction - 0.05).abs() < f64::EPSILON);
    assert_eq!(capacity.avg_utf8_bytes_per_char, 4);
}
