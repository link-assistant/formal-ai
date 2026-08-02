#!/usr/bin/env node

import { writeFileSync } from "node:fs";

const [path] = process.argv.slice(2);
if (!path) {
  throw new Error("usage: write-model-catalog.mjs <output-path>");
}

const contextWindow = 32_768;
const context = {
  avg_utf8_bytes_per_char: 2,
  context_used_fraction: 0,
  context_used_tokens: 0,
  context_window_tokens: contextWindow,
  disk_free_bytes: 0,
  memory_used_bytes: 0,
};
const catalog = {
  models: [{
    slug: "formal-ai",
    display_name: "formal-ai",
    description: "Formal AI symbolic model",
    default_reasoning_level: "none",
    supported_reasoning_levels: [],
    shell_type: "shell_command",
    visibility: "list",
    supported_in_api: true,
    priority: 0,
    availability_nux: null,
    upgrade: null,
    base_instructions: "",
    supports_reasoning_summaries: false,
    supports_reasoning_summary_parameter: false,
    default_reasoning_summary: "none",
    support_verbosity: false,
    default_verbosity: null,
    apply_patch_tool_type: "freeform",
    web_search_tool_type: "text",
    truncation_policy: { mode: "tokens", limit: 8_192 },
    supports_parallel_tool_calls: true,
    context_window: contextWindow,
    max_context_window: contextWindow,
    context,
    effective_context_window_percent: 100,
    experimental_supported_tools: [],
    input_modalities: ["text"],
  }],
};

writeFileSync(path, `${JSON.stringify(catalog, null, 2)}\n`);
