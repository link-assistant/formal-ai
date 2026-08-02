# Server and API

Start a normal local API server with:

```bash
formal-ai serve --host 127.0.0.1 --port 8080
```

Use `--agent-mode` only when a connected harness should receive tool calls:

```bash
formal-ai serve --agent-mode --host 127.0.0.1 --port 8080
```

Agent mode enables the permission-gated tool-call path; it does not execute a
client's shell by itself. Protocol-native hosted types (`web_search`,
`web_search_preview`, versioned web-search variants) and function tools are
normalized by capability. The advertising host executes hosted tools and sends
their results back to the server.

Primary protocol namespaces are OpenAI `/api/openai/v1`, Anthropic
`/api/anthropic/v1`, Gemini `/api/gemini/v1beta`, and Vertex `/api/vertex/v1`.
Legacy `/v1` aliases remain available.

## UTF-8 usage

Usage counts Unicode characters consistently across protocol adapters. It does
not treat each UTF-8 byte as a token, so multibyte text is not inflated. Formal
AI performs no paid inference and reports `cost: 0`; usage fields describe the
request/response size for client displays.

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Привет"}]}' \
  | jq '{usage, context: .context}'
```

## Disk-backed context window

The context window is recalculated from free bytes on the filesystem containing
the memory file, using an average UTF-8 width (default two bytes per character).
Used context comes from the memory file size. API model/response metadata uses:

```json
{
  "context_window_tokens": 1000000,
  "context_used_tokens": 1250,
  "context_used_fraction": 0.00125,
  "disk_free_bytes": 2000000,
  "memory_used_bytes": 2500,
  "avg_utf8_bytes_per_char": 2
}
```

Read it from `GET /api/openai/v1/models`, OpenAI completion responses, generated
Codex catalogs, Gemini/Vertex model metadata, or Anthropic response metadata.
Clients turn `context_used_fraction` into their “% used” display. Set
`FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR` to tune the estimate and
`FORMAL_AI_MEMORY_PATH` to select the measured store.
