//! Serve retention policy, passive health status, and request metadata.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde_json::{Value, json};

use crate::solver_config::ephemeral_enabled;

use super::ApiHttpResponse;

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn log_health_status() -> Value {
    let mut status = crate::dialog_log::structured_log::health_status();
    status["dialog_capture"] = json!(crate::dialog_log::configured_directory().is_some());
    status["derivation_capture"] = json!(!ephemeral_enabled());
    status
}

pub(super) fn memory_health_status() -> Value {
    let mode = if ephemeral_enabled() {
        "read_only"
    } else {
        "read_write"
    };
    let Some(path) = crate::memory_sync::configured_memory_path() else {
        return json!({
            "mode": mode,
            "schema_version": Value::Null,
            "minimum_readable_schema_version": crate::memory::MINIMUM_READABLE_MEMORY_SCHEMA_VERSION,
            "maximum_readable_schema_version": crate::memory::MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION,
            "target_schema_version": crate::memory::TARGET_MEMORY_SCHEMA_VERSION,
            "compatible": true,
            "migration_required": false,
            "migration_state": "missing",
        });
    };
    let original_path = path;
    let path = crate::memory_sync::active_memory_path(&original_path);
    let status = crate::memory::preflight_memory_upgrade(&path);
    json!({
        "mode": mode,
        "active_path":path,"configured_path":original_path,
        "schema_version": status.detected_schema_version,
        "minimum_readable_schema_version": status.minimum_readable_schema_version,
        "maximum_readable_schema_version": status.maximum_readable_schema_version,
        "target_schema_version": status.target_schema_version,
        "compatible": status.compatible,
        "migration_required": status.migration_required,
        "migration_state": status.migration_state,
    })
}

pub(super) fn request_start() -> Option<Instant> {
    (!cfg!(target_arch = "wasm32") && crate::dialog_log::structured_log::enabled("debug"))
        .then(Instant::now)
}

pub(super) fn log_request(
    method: &str,
    normalized_path: &str,
    request_bytes: usize,
    response: &ApiHttpResponse,
    started: Option<Instant>,
) {
    crate::dialog_log::structured_log::event(
        "debug",
        module_path!(),
        || String::from("request.terminal"),
        || {
            json!({
                "request_id": REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1,
                "method": method,
                "path": normalized_path,
                "request_bytes": request_bytes,
                "response_bytes": response.body.len(),
                "elapsed_ms": started.map(|instant| instant.elapsed().as_millis()),
                "status": response.status_code,
            })
        },
    );
}
