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
combinations, length and numeric bounds, and bounded ASCII patterns.
The schema can declare draft-07, 2019-09, or 2020-12 in `$schema`. Each uses
the same supported subset. The validator treats `$id`, `$comment`,
`deprecated`, `readOnly`, `writeOnly`, `contentMediaType`, `contentEncoding`,
and `format` as annotations. References resolve within the supplied schema.

When extracting values from text, Formal AI recognizes equivalent camelCase,
PascalCase, snake_case, and kebab-case property names, including acronyms such
as `HTTPStatus`. If the property name yields no evidence, an explicit source
label can match the property's `description`. The output uses the original
property names from the schema.

String-array properties can extract comma-separated, coordinated, and
bulleted lists in English and Russian. The server uses the property name and
description to select the relevant list. It prefers recognized concepts
among otherwise comparable lists and returns matching enum labels in source
order. If no list can be established and the schema allows an empty array,
the server returns an empty array and records the property name in
the derivation's `no_evidence` array. An explicitly supplied empty JSON array
needs no evidence marker.

To evaluate a criterion, supply a `Criterion:` and a `Resume:`, `Text:`, or
`Evidence:` section in the prompt. Russian labels include `Критерий:`,
`Резюме:`, and `Текст:`. The schema needs a property that accepts the enum
`"met"`, `"not_met"`, and `"unknown"` or the three values `true`, `false`,
and `null`. If several fields qualify, name the intended property `verdict`.

Skill criteria use affirmative, negative, and exclusive statements about the
skill. Minimum-duration criteria compare the corresponding dated period with
the threshold. Year-to-year periods use the difference in calendar years.
Full ISO dates use completed anniversaries. With mixed precision, the rule
returns a determinate verdict if every date consistent with the stated
precision gives the same comparison. An open-ended period such as
`since 2018` needs a reference in the prompt such as
`Today is 2026-11-01` or `Reference date: 2026-11-01`. An open-ended period
without a reference yields `"unknown"` or `null`, independent of the server clock.

The `structured_output` detail contains `verdicts` keyed by the original
schema property name. Each entry gives its `rule` and quotes the source in
`evidence`. A duration comparison that uses a reference also includes
`reference`. For example, the text `Go developer from 2018 to 2024`
supports `"met"` for `at least 3 years of Go` under `minimum_duration`.

Formal AI validates both strict and non-strict requests. It can return a
schema-allowed `null` or `"unknown"` for a missing required value.
If the schema uses unsupported constraints or the result cannot be
derived, Chat Completions returns `message.refusal` and Responses returns
a `refusal` content item. Schema diagnostics name the keyword and give
the JSON Pointer of its containing schema, such as `unsupported keyword
"multipleOf" at /properties/count`. The empty pointer (`""` in the message)
identifies the root schema. Streaming clients receive the corresponding
refusal delta or events.

The `structured_output` thinking step contains the format and the validated
value or refusal. When derivation capture is enabled, pass the locator from
the `derivation_record:` evidence link to `formal-ai explain` to read the
same data from the stored derivation.

## OpenAI concept-link embeddings

`POST /v1/embeddings` and `POST /api/openai/v1/embeddings` embed a string or a
batch of strings. Both model listings advertise `formal-ai-embeddings` with
`dimensions: 4096` as the default and `supported_dimensions` with
`min: 1` and `max: 4096`. The endpoint also accepts the `formal-ai` model ID
and its configured aliases.

```bash
curl -s http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","input":["warehouse supervisor","forklift operator"],"dimensions":256}' \
  | jq '{model, usage, dimensions: [.data[].embedding | length]}'
```

The response is an OpenAI-style list with an `embedding` and zero-based
`index` for each input in request order. Set `dimensions` to any integer from
1 to 4096 to match your vector store.
The default `encoding_format` is `float`, which returns a JSON array of
numbers. Use `base64` for the same values encoded as little-endian float32.
Each batch accepts up to 128 nonblank text inputs with up to 8192 Unicode
characters each. Usage reports the sum of input characters as both
`prompt_tokens` and `total_tokens`.

Coordinates come from the seeded multilingual meaning lexicon. A Wikidata
identifier, when present, or the lexicon meaning ID identifies a concept.
The server sorts those identifiers in UTF-8 byte order and assigns a
coordinate to each, with unused coordinates padded with zeros. It lowercases
and Unicode-normalizes text, then matches the longest seeded word or phrase
at each position. A coordinate is 1 when a matching surface links to that
concept and 0 otherwise. A surface with several meanings activates all of
its linked concepts. Repeating a match leaves its weight at 1.

At 4096 dimensions, cosine similarity measures exact concept overlap. For
example, `storekeeper` and `кладовщик` activate the same coordinate. The
lexicon also links occupations such as `sales manager` and
`менеджер по продажам`, and recognizes `B2B` as a separate qualifier.

For a smaller vector, the server sums the fixed pseudorandom sign rows for
the active concepts and normalizes to unit length before rounding to
float32. Each sign depends on the original concept coordinate and output
coordinate, with the same fixed seed in both servers. If the sum cancels
entirely, the server uses the row for the lowest active coordinate. Projected
cosine similarity approximates concept overlap and loses precision as the
dimension decreases.

If any input has no matching concepts, the request returns HTTP 400 with
`error.code` set to `no_concepts` and `error.input_index` set to the
zero-based position of the first unmatched input. Single inputs use index 0.
The error applies to the whole batch in both encodings.

Identical text gives identical coordinates across restarts with the same
seed and dimensions. Conversation memory has no effect. Re-embed stored
documents when changing dimensions or updating the seeded vocabulary so
indexed vectors and queries use the same coordinates.

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
