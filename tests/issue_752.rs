use std::sync::{Mutex, OnceLock};

use formal_ai::gemini::{gemini_model_metadata, vertex_model_list};
use formal_ai::handle_api_request;

fn memory_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn models_report_real_disk_context_and_memory_usage() {
    let _guard = memory_env_lock();
    let dir =
        std::env::temp_dir().join(format!("formal-ai-context-capacity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp memory directory");
    let memory_path = dir.join("memory.lino");
    std::fs::write(&memory_path, vec![b'x'; 4_096]).expect("write memory fixture");

    let previous_path = std::env::var_os("FORMAL_AI_MEMORY_PATH");
    let previous_average = std::env::var_os("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", &memory_path);
    std::env::remove_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR");
    let free_before = fs2::available_space(&dir).expect("free space before request");
    let response = handle_api_request("GET", "/v1/models", "");
    let gemini = gemini_model_metadata("models/formal-ai");
    let vertex = vertex_model_list("test-project", "test-location");
    let anthropic_response = handle_api_request(
        "POST",
        "/v1/messages",
        r#"{"model":"formal-ai","max_tokens":32,"messages":[{"role":"user","content":"2 + 2"}]}"#,
    );
    std::env::set_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", "4");
    let configured_response = handle_api_request("GET", "/v1/models", "");
    let memory_dir = dir.join("memory-store");
    std::fs::create_dir_all(memory_dir.join("nested")).expect("memory directory");
    std::fs::write(memory_dir.join("memory.lino"), vec![b'x'; 100]).expect("lino file");
    std::fs::write(memory_dir.join("nested/event-log-1"), vec![b'x'; 50]).expect("event log");
    std::fs::write(memory_dir.join("ignored.txt"), vec![b'x'; 1_000]).expect("ignored file");
    std::env::set_var("FORMAL_AI_MEMORY_PATH", &memory_dir);
    std::env::set_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", "2");
    let directory_capacity = formal_ai::context_capacity::ContextCapacity::current()
        .expect("directory-backed context capacity");
    let free_after = fs2::available_space(&dir).expect("free space after request");
    match previous_path {
        Some(value) => std::env::set_var("FORMAL_AI_MEMORY_PATH", value),
        None => std::env::remove_var("FORMAL_AI_MEMORY_PATH"),
    }
    match previous_average {
        Some(value) => std::env::set_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR", value),
        None => std::env::remove_var("FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR"),
    }
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).expect("models JSON");
    let model = &json["models"][0];
    let context = &model["context"];
    let disk_free = context["disk_free_bytes"]
        .as_u64()
        .expect("disk_free_bytes");
    assert!((free_after.min(free_before)..=free_after.max(free_before)).contains(&disk_free));
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
    assert!(anthropic["context"]["context_window_tokens"]
        .as_u64()
        .is_some_and(|value| value > 0));
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
