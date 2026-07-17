title:	OpenAI Chat Completions streaming is malformed: /v1/chat/completions stream:true sends chat.completion not chat.completion.chunk/delta (breaks opencode & AI SDK clients)
state:	CLOSED
author:	konard (Konstantin Diachenko)
labels:	bug, enhancement
comments:	2
assignees:	
projects:	
milestone:	
issue-type:	
parent:	
sub-issues:	
sub-issues-completed:	
blocked-by:	
blocking:	
number:	604
--
## Summary

Our OpenAI-compatible server produces a **malformed streaming response** on `POST /v1/chat/completions` when `stream: true`. Streaming clients that use the standard OpenAI Chat Completions SSE format — including [opencode](https://opencode.ai) via `@ai-sdk/openai-compatible`, and any Vercel AI SDK client — receive an **empty assistant turn**: the request round-trips, tokens are counted, `finish_reason: stop` is reported, but **no text is surfaced**.

This is the **Chat Completions** analog of the Responses-API SSE gap tracked in #602, and it is the one that affects the widest set of clients (anything built on the Vercel AI SDK's OpenAI-compatible provider).

## Reproduction

```sh
# 1. Build + run the server (stable rustc >= 1.96)
cargo build --bin formal-ai
./target/debug/formal-ai serve --host 127.0.0.1 --port 8080 &

# 2. opencode 1.17.13, custom OpenAI-compatible provider — opencode.json:
# {
#   "$schema": "https://opencode.ai/config.json",
#   "provider": { "formalai": {
#     "npm": "@ai-sdk/openai-compatible",
#     "options": { "baseURL": "http://127.0.0.1:8080/v1", "apiKey": "local-formal-ai" },
#     "models": { "formal-symbolic-production": { "name": "Formal Symbolic Production" } }
#   }}
# }
OPENCODE_CONFIG=./opencode.json opencode run -m formalai/formal-symbolic-production "hi"
```

### Observed

opencode prints the session header and exits with **no assistant text**. `--format json` shows the turn completing empty:

```json
{"type":"step_start", ...}
{"type":"step_finish", "part":{"reason":"stop","tokens":{"total":22,"input":1,"output":21,...}}}
```

`output: 21` tokens were produced by the solver, but opencode surfaced nothing.

### Expected

opencode prints `Hi, how may I help you?`.

## Root cause

When `stream: true`, `sse_response()` (`src/server.rs:259`) serializes the **non-streaming** `chat.completion` object and wraps it in a single `data:` line:

```
$ curl -s -N http://127.0.0.1:8080/v1/chat/completions \
    -d '{"model":"formal-symbolic-production","messages":[{"role":"user","content":"hi"}],"stream":true}'
data: {"id":"...","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"Hi, how may I help you?", ...}}]}
data: [DONE]
```

But the OpenAI **streaming** protocol requires **`chat.completion.chunk`** objects whose text lives in `choices[].delta.content` (not `choices[].message.content`):

```
data: {"object":"chat.completion.chunk","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}
data: {"object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Hi, how may I help you?"},"finish_reason":null}]}
data: {"object":"chat.completion.chunk","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}
data: [DONE]
```

opencode's AI-SDK parser reads `choices[].delta`; our payload has `choices[].message` and `object: "chat.completion"`, so it extracts no text and closes the turn empty.

The **non-streaming** path is correct — the same request without `stream:true` returns `message.content = "Hi, how may I help you?"`. This is strictly a streaming-shape bug.

## Fix

When `stream: true` on `/v1/chat/completions`, emit a proper SSE stream of `chat.completion.chunk` events:
- first chunk: `delta: {"role":"assistant"}`
- one or more content chunks: `delta: {"content": "<piece>"}` (a single chunk with the whole text is acceptable; incremental is nicer)
- final chunk: `delta: {}`, `finish_reason: "stop"`
- terminate with `data: [DONE]`

`object` must be `"chat.completion.chunk"` on every event. (Move the `thinking_steps` out of the streamed text path, or carry them on the final chunk / a vendor extension field — they must not replace the `delta` shape.)

## Documentation ask

Alongside the fix, please add **clear, copy-pasteable documentation** (in `README.md` / `ARCHITECTURE.md` and/or `docs/`) for **how to configure opencode against Formal AI** end to end: the exact `opencode.json` provider block, the `baseURL`, model id, and the `opencode run -m <provider>/<model> "hi"` invocation — verified to print a reply. A user should be able to follow the doc and get a working `hi` without reverse-engineering the wire format.

## Acceptance criteria

- [ ] `POST /v1/chat/completions` with `stream:true` returns `object: "chat.completion.chunk"` events with `choices[].delta.content`, terminated by `data: [DONE]`.
- [ ] `opencode run -m formalai/formal-symbolic-production "hi"` prints `Hi, how may I help you?` (or equivalent), non-empty.
- [ ] An automated end-to-end test drives `/v1/chat/completions` with `stream:true` over real HTTP and asserts a chunk-shaped stream with recoverable text (catches this regression in CI, which the in-process in-repo driver does not).
- [ ] Documentation for configuring opencode (and generally any `@ai-sdk/openai-compatible` client) against Formal AI is added and verified.

## Environment

- `formal-ai 0.254.0`, `serve` on `127.0.0.1:8080`, stable rustc 1.96.1, macOS aarch64
- `opencode 1.17.13` (`@ai-sdk/openai-compatible`)

## Related

- #602 — OpenAI **Responses** API SSE (codex). This issue is the **Chat Completions** SSE sibling.
- #603 — umbrella multi-protocol server (this is sub-issue **S3**: OpenAI Chat SSE correctness).

