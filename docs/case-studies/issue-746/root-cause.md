# Root cause

`ChatCompletionRequest::requested_tool_names` delegates each advertisement to `tool_definition_name`. That helper only inspected `function.name` and top-level `name`. OpenAI hosted definitions such as `{"type":"web_search"}` therefore produced no names. The permission gate interpreted the empty set as wildcard `tool:*`, refused it, and the planner never ran.

Gemini's adapter separately flattened `functionDeclarations`, but silently discarded key-only hosted declarations such as `googleSearch` and `url_context`. With no advertised matching capability, normal symbolic web handlers returned their browser-demo explanation.

The fix places canonical hosted-type normalization at the shared protocol boundary, adds Google-hosted normalization in the Gemini adapter, and classifies the hosted file/execution aliases needed by the existing capability router. No natural-language phrase is hardcoded in production; multilingual intent recognition remains seed-driven.

A real Codex replay exposed a second boundary error. Once routing worked, the Responses serializer still converted every planned call to `function_call`. Codex correctly rejected `web_search` because a hosted tool runs on the API server and is not a local function the client can execute. The Responses output model and SSE renderer now preserve the native completed `web_search_call` item and its search action, while named function tools retain the existing `function_call`/`function_call_output` loop.

A real Qwen replay exposed a third classification error: any advertised name containing `search`, including `grep_search` and deferred `tool_search`, was treated as internet search. Web-search classification now requires both `web` and `search`; repository search keeps its dedicated route, and a client that only advertises `tool_search` receives one deferred-discovery call. Tool-result detection resolves missing result names through the prior call id, preventing a repeated discovery call.
