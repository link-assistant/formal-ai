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

## OpenAI structured output

`GET /v1/models` advertises the `json_schema` capability. To request a
validated JSON object, send `response_format` on Chat Completions or
`text.format` on Responses. The response echoes the selected format,
including the `strict` value when supplied.

| Endpoint | JSON schema | JSON object mode |
| --- | --- | --- |
| `POST /v1/chat/completions` | `response_format: {type: "json_schema", json_schema: {name, strict, schema}}` | `response_format: {type: "json_object"}` |
| `POST /v1/responses` | `text.format: {type: "json_schema", name, strict, schema}` | `text.format: {type: "json_object"}` |

The supported schema constraints include JSON types, required keys,
`additionalProperties`, `enum`, `const`, arrays, local references, schema
combinations, length and numeric bounds, and simple ASCII patterns.
Formal AI validates both strict and non-strict requests. It can return a
schema-allowed `null` or `"unknown"` for a missing required value.
If the schema uses unsupported constraints or the result cannot be
derived, Chat Completions returns `message.refusal` and Responses returns
a `refusal` content item. Streaming clients receive the corresponding
refusal delta or events.

The `structured_output` thinking step contains the format and the validated
value or refusal. When derivation capture is enabled, pass the locator from
the `derivation_record:` evidence link to `formal-ai explain` to read the
same data from the stored derivation.

## UTF-8 usage

Usage counts Unicode characters consistently across protocol adapters. It does
not treat each UTF-8 byte as a token, so multibyte text is not inflated. Formal
AI performs no paid inference; usage envelopes deliberately omit cost and
cache fields, and the usage counts describe the request/response size for
client displays.

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
