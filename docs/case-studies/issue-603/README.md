# Issue 603 Case Study

## Collected Data

Raw issue, PR, and CI evidence is preserved under `raw-data/`:

- `issue-603.json` and `issue-603-comments.json`
- `pr-612.json`, `pr-612-conversation-comments.json`,
  `pr-612-review-comments.json`, and `pr-612-reviews.json`
- `recent-runs.json`

The prepared PR started as a draft with only task context committed. The latest
captured branch run was successful, but the code checks were skipped because no
implementation commits existed yet.

## Requirements

- Expose a universal local gateway under `/api/<protocol>/...`.
- Keep OpenAI-compatible endpoints under `/api/openai/v1` and retain existing
  `/v1/*` aliases for older Codex, OpenCode, desktop, and agent configs.
- Serve Anthropic Messages at `/api/anthropic/v1/messages`.
- Serve Gemini native `models`, `generateContent`, and `streamGenerateContent`
  routes under `/api/gemini/v1beta`.
- Serve Vertex-shaped publisher-model `generateContent` routes under
  `/api/vertex/v1/projects/{project}/locations/{location}/publishers/google`.
- Fix OpenAI Responses streaming so Codex receives named Responses SSE events
  ending in `response.completed`.
- Document copy-paste configs for Codex, OpenCode, Claude Code, Gemini, and
  Vertex-shaped clients.

## Root Cause

Before this change the server exposed only flat `/v1/*` routes. That made it
usable for OpenAI-compatible clients and the existing Anthropic alias, but it
could not distinguish protocol families or provide native Gemini/Vertex route
shapes. The Responses endpoint also ignored `stream: true`, returning one JSON
object instead of the named Server-Sent Events expected by Responses clients.

## Implemented Path

The server now routes protocol namespaces before the legacy flat aliases:

- `/api/openai/v1/models`, `/api/openai/v1/chat/completions`, and
  `/api/openai/v1/responses`
- `/api/anthropic/v1/messages`
- `/api/gemini/v1beta/models/{model}:generateContent` and
  `:streamGenerateContent`
- `/api/vertex/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent`
- `/api/formal-ai/v1/*` for graph, bundle, links, and memory routes

The Gemini and Vertex adapters translate `contents` into the same shared chat
request used by the OpenAI and Anthropic surfaces, then wrap the solver result
as a `GenerateContentResponse`. OpenAI Responses streaming emits a compact
Responses SSE sequence: `response.created`, output item events, text/function
delta events, and a final `response.completed`.

## Reproduction Test

`tests/unit/specification/openai_compatibility.rs` includes
`responses_stream_true_emits_responses_sse_protocol`, which failed before the
fix because `/api/openai/v1/responses` returned `404` and no Responses SSE event
sequence existed.

`tests/integration/multi_protocol_api.rs` starts `formal-ai serve` on a loopback
port and verifies the public protocol routes through real HTTP requests.

## Verification Plan

- `cargo test --test unit responses_stream_true_emits_responses_sse_protocol -- --nocapture`
- `cargo test --test unit protocol_namespaces_route_to_the_same_openai_and_formal_ai_surfaces -- --nocapture`
- `cargo test --test unit gemini_and_vertex_protocols_share_the_solver_with_native_model_lists -- --nocapture`
- `cargo test --test integration cli_serve_exposes_namespaced_protocols_over_loopback_http -- --nocapture`
- `cargo test --test unit issue_603_multi_protocol_gateway_docs_are_traceable -- --nocapture`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `cargo test --all-features`
