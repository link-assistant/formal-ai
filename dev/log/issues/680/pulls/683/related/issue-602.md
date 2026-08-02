title:	OpenAI-compatible server cannot be driven by Codex CLI: no SSE streaming on /v1/responses
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
number:	602
--
## Summary

Our OpenAI-compatible HTTP server (`formal-ai serve`) cannot be driven by the [Codex CLI](https://github.com/openai/codex) (`codex-cli 0.142.0`). A trivial `codex exec "hi"` pointed at our server never completes: Codex retries 5 times and dies with `stream disconnected before completion`.

This matters because the project's direction (see issue #538 and the `agent` subcommand's docstring) is that Formal AI should be drivable by *any* external agentic CLI through our OpenAI-compatible endpoint. Codex is a mainstream one, and today it fails at the very first turn. The in-repo agentic driver never surfaces this because it calls the solver in-process and does not exercise the HTTP transport an external CLI actually uses.

Both underlying endpoints return correct answers for a plain (non-streaming) request — the gap is purely protocol/transport, not reasoning.

## Reproduction

```sh
# 1. Build and start the server (stable rustc >= 1.96)
cargo build --bin formal-ai
./target/debug/formal-ai serve --host 127.0.0.1 --port 8080 &

# 2. Drive it with Codex (codex-cli 0.142.0)
OPENAI_API_KEY="local-formal-ai" codex exec \
  -c 'model_providers.formalai.name="Formal AI"' \
  -c 'model_providers.formalai.base_url="http://127.0.0.1:8080/v1"' \
  -c 'model_providers.formalai.wire_api="responses"' \
  -c 'model_providers.formalai.env_key="OPENAI_API_KEY"' \
  -c 'model_provider="formalai"' \
  -c 'model="formal-ai"' \
  --skip-git-repo-check --sandbox read-only \
  "hi"
```

### Observed

```
ERROR codex_models_manager::manager: failed to refresh available models:
  ... missing field `models` at line 15 ...
warning: Model metadata for `formal-symbolic-production` not found. Defaulting to fallback metadata ...
ERROR: Reconnecting... 1/5
ERROR: Reconnecting... 2/5
ERROR: Reconnecting... 3/5
ERROR: Reconnecting... 4/5
ERROR: Reconnecting... 5/5
ERROR: stream disconnected before completion: stream closed before response.completed
```

### Expected

Codex prints the assistant reply (`Hi, how may I help you?`) and exits 0.

## Root causes (two distinct gaps)

### 1. No SSE streaming on `/v1/responses` (fatal)

Codex 0.142 speaks the **Responses** wire API and always opens the request as a stream (`{"input":"hi","stream":true}`), expecting a `text/event-stream` body that ends with a `response.completed` event.

Our server **ignores `stream: true`** and answers with a single non-streaming JSON body:

```
$ curl -s -o /dev/null -D - http://127.0.0.1:8080/v1/responses \
    -H 'content-type: application/json' \
    -d '{"model":"formal-ai","input":"hi","stream":true}'
HTTP/1.1 200 OK
content-type: application/json          <-- should be text/event-stream when stream:true
content-length: 5719
connection: close                       <-- closes before any response.completed event
```

Codex sees the socket close before `response.completed` and reports `stream disconnected before completion`.

The same request **without** streaming works and returns the correct answer with the full symbolic thinking trace:

```
$ curl -s http://127.0.0.1:8080/v1/responses \
    -H 'content-type: application/json' \
    -d '{"model":"formal-ai","input":"hi"}'
{ "output":[{"type":"message","role":"assistant",
   "content":[{"type":"output_text","text":"Hi, how may I help you?"}]}], ... }
```

**Fix:** when `stream: true`, emit an SSE stream (`content-type: text/event-stream`) with the Responses event sequence — at minimum `response.created` → `response.output_item.added` → `response.output_text.delta` (one or more) → `response.output_item.done` → `response.completed`. The same applies to `/v1/chat/completions` (`data: {chunk}` / `data: [DONE]`) so streaming Chat-API clients work too.

### 2. `/v1/models` uses `data` instead of `models` (non-fatal warning)

Codex's model manager expects `{"models":[...]}`; we return the OpenAI-standard `{"data":[...]}`:

```
$ curl -s http://127.0.0.1:8080/v1/models
{ "data": [ { "id": "formal-symbolic-production", ... } ], "object": "list", ... }
```

This only produces a warning (`failed to refresh available models`, then falls back to default model metadata), but it degrades the experience and should be addressed so Codex recognizes the model.

## Acceptance criteria

- [ ] `POST /v1/responses` with `stream: true` returns `content-type: text/event-stream` and emits the Responses SSE event sequence ending in `response.completed`.
- [ ] `POST /v1/chat/completions` with `stream: true` returns an SSE stream of `chat.completion.chunk` events terminated by `data: [DONE]`.
- [ ] The reproduction command above (`codex exec "hi"`) completes and prints `Hi, how may I help you?` (or an equivalent greeting), exit 0.
- [ ] Codex no longer emits the `failed to refresh available models` error for our `/v1/models`.
- [ ] An automated end-to-end test drives the server over its real HTTP transport with a streaming client (ideally Codex itself, or a minimal SSE client) so this regression is caught in CI — not only by the in-process in-repo driver.

## Environment

- `codex-cli 0.142.0`
- `formal-ai 0.254.0`, `serve` on `127.0.0.1:8080`
- rustc 1.96.1 (stable), macOS (aarch64)

## Notes

- The reasoning core is fine: both `/v1/chat/completions` and `/v1/responses` return `"Hi, how may I help you?"` with a complete symbolic thinking trace for a non-streaming request. This is strictly a transport/protocol-compatibility gap.
- Codex 0.142 removed support for `wire_api = "chat"` (it errors: *"`wire_api = "chat"` is no longer supported"*), so the `responses` wire API is the only path for this client — which makes SSE support on `/v1/responses` the priority.

---

## Documentation requirement (applies to this issue)

This issue is not complete until it ships **clear, copy-pasteable, end-to-end-verified documentation** (in `README.md` / `ARCHITECTURE.md` and/or `docs/`) for how a user configures and uses the relevant CLI/tooling against Formal AI — such that following the doc produces a working result (e.g. a `hi` reply, or the documented command succeeding) **without reverse-engineering the wire format or config**. Every accepted client/model/config surface introduced here must be documented with a concrete, tested example.

---

> **Model id note (see #605):** command examples above use the canonical model id `formal-ai`. The `formal-symbolic-production` strings that remain are **verbatim observed server output** from the time of testing; #605 renames that id to `formal-ai`, after which the server's `/v1/models` listing and any metadata warning will use `formal-ai`.
