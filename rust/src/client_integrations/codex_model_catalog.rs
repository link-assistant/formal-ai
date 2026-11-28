//! Model metadata written for Codex client configurations.

use std::error::Error;

use crate::context_capacity::ContextCapacity;

use super::RenderContext;

pub(super) fn codex_model_catalog(render: &RenderContext) -> Result<String, Box<dyn Error>> {
    let context = ContextCapacity::current()?;
    let (window, _) = render.model_limits();
    let window: u64 = window.parse()?;
    let model = &render.model;
    let catalog = serde_json::json!({
        "models": [{
            "slug": model,
            "display_name": model,
            "description": "Formal AI symbolic model",
            "default_reasoning_level": "none",
            "supported_reasoning_levels": [],
            "shell_type": "shell_command",
            "visibility": "list",
            "supported_in_api": true,
            "priority": 0,
            "availability_nux": null,
            "upgrade": null,
            "base_instructions": "",
            "supports_reasoning_summaries": false,
            "supports_reasoning_summary_parameter": false,
            "default_reasoning_summary": "none",
            "support_verbosity": false,
            "default_verbosity": null,
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text",
            "truncation_policy": {"mode": "tokens", "limit": 8192},
            "supports_parallel_tool_calls": true,
            "context_window": window,
            "max_context_window": window,
            "context": context,
            "effective_context_window_percent": 100,
            "experimental_supported_tools": [],
            "input_modalities": ["text"]
        }]
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&catalog)?))
}
