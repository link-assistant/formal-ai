//! Complete agentic conversation export and learning endpoints (#822).

use serde_json::json;

use super::{error_response, json_response, links_notation_response, query_param, ApiHttpResponse};
use crate::memory_sync::SyncStore;

pub(super) fn handle_context_request(dialog_id: &str, query: &str) -> ApiHttpResponse {
    let mut context = match crate::conversation_context::load_conversation_context(dialog_id) {
        Ok(context) => context,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {
            return error_response(400, &error.to_string());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return error_response(404, &error.to_string());
        }
        Err(error) => return error_response(500, &error.to_string()),
    };
    match query_param(query, "include").as_deref() {
        Some("harness") => {
            if let Some(object) = context.as_object_mut() {
                object.remove("server_logs");
            }
        }
        Some("server") => {
            if let Some(object) = context.as_object_mut() {
                object.remove("messages");
            }
        }
        Some("both") | None => {}
        Some(_) => return error_response(400, "include must be harness, server, or both"),
    }
    if query_param(query, "format").as_deref() == Some("json") {
        return json_response(200, &context);
    }
    links_notation_response(
        200,
        crate::conversation_context::conversation_context_to_lino(dialog_id, &context),
    )
}

pub(super) fn handle_learning_request(dialog_id: &str) -> ApiHttpResponse {
    let context = match crate::conversation_context::load_conversation_context(dialog_id) {
        Ok(context) => context,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {
            return error_response(400, &error.to_string());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return error_response(404, &error.to_string());
        }
        Err(error) => return error_response(500, &error.to_string()),
    };
    let document = crate::conversation_context::conversation_context_to_lino(dialog_id, &context);
    let mut store = SyncStore::open();
    match store.record_chat_exchange(&format!("agentic_report_{dialog_id}"), &document) {
        Ok(events_recorded) => json_response(
            200,
            &json!({
                "dialog_id": dialog_id,
                "learned": true,
                "events_recorded": events_recorded,
            }),
        ),
        Err(error) => error_response(
            500,
            &config("context_learning_failed").replace("{error}", &error.to_string()),
        ),
    }
}

fn config(key: &str) -> String {
    crate::seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}
