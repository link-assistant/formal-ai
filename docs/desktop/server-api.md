# Desktop runtime: in-process agent and the optional server API

formal-ai Desktop is the Electron shell that packages the same web chat shipped to
[GitHub Pages](https://link-assistant.github.io/formal-ai/). It has two runtime modes:

1. **In-process agent (default).** No server, nothing listening — the reasoning
   agent runs inside the app, exactly like the in-browser demo.
2. **Local multi-protocol server (opt-in).** A loopback HTTP API you can turn
   on to expose OpenAI, Anthropic, Gemini, Vertex, and formal-ai native routes
   to the bundled UI *and* to external coding CLIs such as
   [`claude`](https://github.com/anthropics/claude-code),
   [`codex`](https://github.com/openai/codex), and
   [`agent`](https://github.com/link-assistant/agent).

This document explains both modes, how to enable and configure the server, and
how the web UI reuses the same code in the browser and on the desktop.

> **Scope.** The server is a local REST gateway. Client protocols live under
> `/api/<protocol>/...`, while formal-ai's own seed data, memory bundles, and
> traces still prefer [Links Notation](https://github.com/link-foundation/lino).
> REST is the interop boundary; Links Notation is the internal format.

---

## 1. In-process agent (default)

When you launch the desktop app normally:

```bash
npm --prefix desktop run dev      # development
npm --prefix desktop run build    # packaged installers (build:linux / build:mac / build:win)
```

it starts **only** a static file server on a random loopback port to serve the
web bundle, and reports `mode: "in-process"` to the UI. No `formal-ai serve`
child process is spawned at first launch, so nothing binds an API port until
the user either enables the startup opt-in below or switches the desktop UI to
Agent / Full Auto mode. The UI answers normal chat prompts with the same engine
the public web demo uses.

This mirrors the design of the upstream
[`agent`](https://github.com/link-assistant/agent) CLI: an agent you can run
locally with no external service. The badge in the desktop status bar reads
**“in-process”** in this mode.

You do not need the server for normal chat. Turn it on only when you want to
point an external CLI at formal-ai, or share the local API with another tool.

---

## 2. Enable the optional server

### 2a. From the desktop app

Set `FORMAL_AI_DESKTOP_SERVER` to a truthy value (`1`, `true`, `yes`, or `on`)
before launching when you want the API ready immediately. The shell then starts
`formal-ai serve` on a free loopback port, waits for `GET /health`, and routes
chat through `POST /v1/chat/completions`. The status badge switches to
**“API local”**.

```bash
FORMAL_AI_DESKTOP_SERVER=1 npm --prefix desktop run dev
```

Without that environment variable, switching the desktop UI to **Agent** or
**Full Auto** starts the same local server on demand, reusing an already healthy
server when one is running. The resulting `apiBase` is exposed in the desktop
status and provider metadata so the later Agent CLI integration can point at
the same local OpenAI-compatible backend.

The desktop server is bound to `127.0.0.1` and is **unauthenticated by design**
— the shell scrubs any bearer-token environment variables before spawning it,
because it is reachable only from the local machine. If you need a token (see
[§3](#3-authentication)), run the CLI directly instead.

### 2b. From the CLI (standalone)

You can also run the server without the desktop app:

```bash
formal-ai serve --host 127.0.0.1 --port 8080
# or, from a checkout:
cargo run -- serve --host 127.0.0.1 --port 8080
```

| Flag     | Environment variable | Default     |
| -------- | -------------------- | ----------- |
| `--host` | `FORMAL_AI_HOST`     | `127.0.0.1` |
| `--port` | `FORMAL_AI_PORT`     | `8080`      |

Verify it is up:

```bash
curl http://127.0.0.1:8080/health
# {"status":"ok","model":"formal-ai"}
```

### Endpoints

| Method | Primary path | Legacy alias | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | - | Liveness probe (never requires a token) |
| `GET` | `/api/openai/v1/models` | `/v1/models` | OpenAI-style model list + advertised rate limits |
| `POST` | `/api/openai/v1/chat/completions` | `/v1/chat/completions` | OpenAI Chat Completions (supports `stream`) |
| `POST` | `/api/openai/v1/responses` | `/v1/responses` | OpenAI Responses API (supports named SSE events) |
| `POST` | `/api/anthropic/v1/messages` | `/v1/messages` | Anthropic Messages adapter for `claude` ([§4d](#4d-claude-anthropic-claude-code---first-party-adapter)) |
| `GET` | `/api/gemini/v1beta/models` | - | Gemini native model list |
| `POST` | `/api/gemini/v1beta/models/{model}:generateContent` | - | Gemini native `generateContent` |
| `POST` | `/api/gemini/v1beta/models/{model}:streamGenerateContent` | - | Gemini native streaming `generateContent` SSE |
| `GET` | `/api/vertex/v1/projects/{project}/locations/{location}/publishers/google/models` | - | Vertex publisher model list |
| `POST` | `/api/vertex/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent` | - | Vertex-shaped `generateContent` |
| `GET` | `/api/formal-ai/v1/graph` | `/v1/graph` | Reasoning-graph nodes/edges for a trace |
| `OPTIONS` | *(any)* | - | CORS preflight -> `204 No Content` |

Compatibility aliases remain available for existing configs: `POST /v1/responses`,
`POST /v1/chat/completions`, `POST /v1/messages`, `GET /v1/models`, and the
native formal-ai `/v1/*` graph, bundle, links, and memory routes.

The single advertised model id is **`formal-ai`**. The OpenAI and Gemini model
list endpoints expose that id, and the OpenAI list also advertises a rate limit
of 60 requests/min and 60,000 tokens/min. Chat Completions, Responses, Messages,
Gemini, and Vertex also accept `@link-assistant/formal-ai`,
`link-assistant/formal-ai`, `formal-ai-latest`, and `latest` as
case-insensitive aliases; response payloads return the canonical `formal-ai` id.

#### Links-Notation REST + LinksQL (R6)

These endpoints speak [Links Notation](https://github.com/link-foundation/lino)
envelopes rather than OpenAI JSON - REST is the transport, Links Notation is the
payload (R7). The memory-sync flow in [§5c](#5c-local-database-sync-r5c) rides on
the `/api/formal-ai/v1/memory*` routes, with `/v1/memory*` kept as aliases.

| Method | Primary path | Legacy alias | Purpose |
| --- | --- | --- | --- |
| `GET` | `/api/formal-ai/v1/bundle` | `/v1/bundle` | Full `formal_ai_bundle` (seed + memory) as Links Notation |
| `GET` | `/api/formal-ai/v1/links` | `/v1/links` | Knowledge graph as a `knowledge_graph` Links-Notation document |
| `POST` | `/api/formal-ai/v1/links/query` | `/v1/links/query` | LinksQL `MATCH (a)-[r]->(b) WHERE ... RETURN ...` -> `links_query_result` |
| `GET` | `/api/formal-ai/v1/memory` | `/v1/memory` | Whole `demo_memory` event log |
| `GET` | `/api/formal-ai/v1/memory/since` | `/v1/memory/since` | Delta after `?event=<id>` (events not yet seen) |
| `POST` | `/api/formal-ai/v1/memory/import` | `/v1/memory/import` | Union-by-id merge of a posted `demo_memory` log |

`POST /api/formal-ai/v1/links/query` is the primary route and `/v1/links/query`
is the compatibility alias. It accepts either a JSON `{"query":"…"}` body or a
Links-Notation `query "…"` body and returns a `links_query_result` envelope. The
read-only LinksQL evaluator lives in
[`src/links_query.rs`](../../src/links_query.rs); the graph projection comes from
`KnowledgeGraph::to_links_notation`.

---

## 3. Authentication

The standalone server reads an **optional** bearer token from the first non-empty
of these environment variables:

- `FORMAL_AI_API_BEARER_TOKEN`
- `FORMAL_AI_HTTP_BEARER_TOKEN`
- `FORMAL_AI_API_TOKEN`

Behaviour:

- **No token set** -> `/api/*` and `/v1/*` are open. Safe for loopback-only use.
- **Token set** -> every `/api/*` and `/v1/*` request must send
  `Authorization: Bearer <token>` or the matching protocol API-key header
  (`x-api-key`, `x-goog-api-key`, or `anthropic-api-key`); otherwise the server
  replies `401`. `GET /health` and `OPTIONS` never require a token.

```bash
FORMAL_AI_API_BEARER_TOKEN=sk-local-demo formal-ai serve --host 127.0.0.1 --port 8080
curl http://127.0.0.1:8080/api/openai/v1/models -H "Authorization: Bearer sk-local-demo"
```

---

## 4. Point coding CLIs at the local server

The same loopback server can answer several agentic terminal clients:

- `codex` uses `POST /api/openai/v1/responses` through the Responses wire API.
- `opencode` and `agent` use `POST /api/openai/v1/chat/completions` through
  `@ai-sdk/openai-compatible`.
- `claude` uses the built-in Anthropic adapter at
  `POST /api/anthropic/v1/messages`.
- Gemini-compatible clients use
  `POST /api/gemini/v1beta/models/{model}:generateContent`.
- Vertex-shaped clients use the Google publisher-model path under
  `/api/vertex/v1`.

The examples assume the standalone server from [§2b](#2b-from-the-cli-standalone)
at `http://127.0.0.1:8080`. If you set a bearer token, use the same value as the
API key; otherwise any non-empty placeholder works because the loopback server
does not require a key.

### 4a. `codex` (OpenAI Codex CLI) - Responses API

`codex` configures providers in `~/.codex/config.toml`. Codex 0.142+ does not
support `wire_api = "chat"`. Custom providers use the Responses wire API and
Codex always streams, so keep `wire_api = "responses"` and use the server's
`/api/openai/v1` base URL:

```toml
# ~/.codex/config.toml
model_provider = "formalai"
model = "formal-ai"

[model_providers.formalai]
name = "formal-ai local server"
base_url = "http://127.0.0.1:8080/api/openai/v1"
env_key = "FORMAL_AI_API_KEY"
wire_api = "responses"
```

```bash
export FORMAL_AI_API_KEY="sk-local-demo"   # match your bearer token, or any non-empty value
codex exec --skip-git-repo-check --sandbox read-only "hi"
# Hi, how may I help you?
```

You can also select the provider per invocation without editing the file:

```bash
FORMAL_AI_API_KEY="sk-local-demo" codex exec \
  -c 'model_providers.formalai.name="formal-ai local server"' \
  -c 'model_providers.formalai.base_url="http://127.0.0.1:8080/api/openai/v1"' \
  -c 'model_providers.formalai.env_key="FORMAL_AI_API_KEY"' \
  -c 'model_providers.formalai.wire_api="responses"' \
  -c 'model_provider="formalai"' \
  -c 'model="formal-ai"' \
  --skip-git-repo-check --sandbox read-only \
  "hi"
# Hi, how may I help you?
```

See the upstream
[Codex configuration reference](https://developers.openai.com/codex/config-reference)
for the full `model_providers` schema.

### 4b. `opencode` - OpenAI-compatible provider

OpenCode supports custom providers in `~/.config/opencode/opencode.json`. Use
`@ai-sdk/openai-compatible` because formal-ai exposes the Chat Completions
endpoint at `/api/openai/v1/chat/completions`. This example uses the provider
id `formalai`, so the model selector is `formalai/formal-ai`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "formalai": {
      "name": "formal-ai local server",
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:8080/api/openai/v1",
        "apiKey": "{env:FORMAL_AI_API_KEY}"
      },
      "models": {
        "formal-ai": {
          "name": "formal-ai"
        }
      }
    }
  },
  "model": "formalai/formal-ai"
}
```

```bash
export FORMAL_AI_API_KEY="sk-local-demo"   # match your bearer token, or any non-empty value
opencode run -m formalai/formal-ai "hi"
# Hi, how may I help you?
```

See the upstream [OpenCode provider documentation](https://opencode.ai/docs/providers/)
for custom provider fields and the [OpenCode configuration reference](https://opencode.ai/docs/config/)
for the `provider` and `model` options.

### 4c. `agent` (link-assistant/agent) - OpenCode-compatible client

[`agent`](https://github.com/link-assistant/agent) accepts OpenCode-style
provider/model selection. Put the same provider record in
`~/.config/link-assistant-agent/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "name": "formal-ai local server",
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:8080/api/openai/v1",
        "apiKey": "{env:FORMAL_AI_API_KEY}"
      },
      "models": {
        "formal-ai": {
          "name": "formal-ai"
        }
      }
    }
  },
  "model": "formal-ai/formal-ai"
}
```

```bash
formal-ai serve --agent-mode --host 127.0.0.1 --port 8080
export FORMAL_AI_API_KEY="sk-local-demo"   # match your bearer token, or any non-empty value
agent --model formal-ai/formal-ai --permission-mode plan -p \
  "run ls to list files here"
```

`agent` documents its provider/model selection in
[`MODELS.md`](https://github.com/link-assistant/agent/blob/main/MODELS.md) and
its [README](https://github.com/link-assistant/agent#readme). Run autonomous
agent clients only in a workspace where their file and shell actions are
acceptable; formal-ai serves model responses but does not sandbox the client
process. In agent mode, a directory-listing prompt like the example above makes
formal-ai return `bash` / `shell` / `run_command` `tool_calls` with
`{"command":"ls"}`. Use `--permission-mode plan` when read-only shell commands
such as `ls` may run, and use hard `--read-only` when the Agent CLI should
disable shell execution entirely.

### 4d. `claude` (Anthropic Claude Code) - first-party adapter

[`claude`](https://github.com/anthropics/claude-code) speaks the **Anthropic
Messages** protocol (`/v1/messages`), not OpenAI Chat Completions. Its
`ANTHROPIC_BASE_URL` only routes to **Anthropic-protocol-compatible** backends,
so it cannot call formal-ai's OpenAI endpoint directly.

formal-ai ships a **first-party** Anthropic→OpenAI adapter built into the server,
so no third-party proxy is required. `POST /api/anthropic/v1/messages`
([`src/anthropic.rs`](../../src/anthropic.rs)) flattens Anthropic message/system
blocks into the solver's chat request, calls the same engine the OpenAI endpoints
use, and re-wraps the result as an Anthropic response — including the full
`message_start` → `content_block_delta` → `message_stop` SSE sequence when
`stream: true`. Point `claude` straight at the Anthropic namespace:

```bash
# 1. start formal-ai (it exposes /api/anthropic/v1/messages)
formal-ai serve --host 127.0.0.1 --port 8080
# 2. point Claude Code at the Anthropic base URL — no proxy in between
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080/api/anthropic"
export ANTHROPIC_API_KEY="sk-local-demo"   # match your bearer token, or any non-empty value
claude
```

If you set a bearer token (see [§3](#3-authentication)), `claude` sends it as the
`ANTHROPIC_API_KEY`; on a loopback-only server any non-empty value works when no
token is required.

The legacy server root still accepts `POST /v1/messages` for existing Claude
configs that were written before the protocol namespace existed.

### 4e. Gemini native API

Gemini-compatible clients can use formal-ai without an OpenAI-compatible shim.
The Gemini namespace exposes native `models`, `generateContent`, and
`streamGenerateContent` routes:

```bash
export GEMINI_API_KEY="sk-local-demo"   # match your bearer token, or any non-empty value
export GOOGLE_GEMINI_BASE_URL="http://127.0.0.1:8080/api/gemini"
gemini -m formal-ai -p "hi"
```

```bash
curl -s http://127.0.0.1:8080/api/gemini/v1beta/models/formal-ai:generateContent \
  -H 'content-type: application/json' \
  -H "x-goog-api-key: ${GEMINI_API_KEY}" \
  -d '{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}'
```

Streaming uses the matching Gemini method name:

```bash
curl -s http://127.0.0.1:8080/api/gemini/v1beta/models/formal-ai:streamGenerateContent \
  -H 'content-type: application/json' \
  -d '{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}'
```

### 4f. Vertex-shaped API

Vertex clients that can target a custom base URL can use the Google
publisher-model path under `/api/vertex/v1`. The adapter accepts the same
Gemini `contents` body and returns a `GenerateContentResponse`-shaped object:

```bash
export GOOGLE_VERTEX_BASE_URL="http://127.0.0.1:8080/api/vertex"
curl -s \
  http://127.0.0.1:8080/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models/formal-ai:generateContent \
  -H 'content-type: application/json' \
  -H "x-goog-api-key: ${FORMAL_AI_API_KEY:-sk-local-demo}" \
  -d '{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}'
```

List the Vertex-shaped publisher models first when a client needs discovery:

```bash
curl -s \
  http://127.0.0.1:8080/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models
```

### 4g. Agentic mode: the server drives a tool-calling loop (issue #468)

Beyond answering a single prompt, the server can drive an **agentic tool-calling
loop** on every surface above: the CLI advertises its tools, the server decides the
next tool to call, the CLI executes it and feeds the result back, and the loop runs
until the server returns the finished answer. This is the capability issue
[#468](https://github.com/link-assistant/formal-ai/issues/468) asks for — *"call
all the tools from any agentic CLI, understand errors from tools … web fetch and
web search, to actually complete the task"* — exercised on the example task of
formalizing «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.

A single deterministic planner
([`src/agentic_coding/planner.rs`](../../src/agentic_coding/planner.rs)) backs all
three surfaces, so the loop behaves identically whichever CLI you point at the
server:

- **Chat Completions** (`opencode`, `agent`) — the assistant turn carries
  `tool_calls` with `finish_reason: "tool_calls"`; the CLI replies with `tool`-role
  messages.
- **Responses** (`codex`) — the server emits a `function_call` output item; the CLI
  replies with a `function_call_output` item.
- **Anthropic Messages** (`claude`) — the server emits a `tool_use` content block
  with `stop_reason: "tool_use"`; the CLI replies with a `tool_result` block on a
  `user` message.

**Strictly opt-in.** Tools are refused unless the server is started with
`formal-ai serve --agent-mode` or `FORMAL_AI_AGENT_MODE=1`, *and* each requested
tool passes a per-tool permission gate
([`src/associative_package.rs`](../../src/associative_package.rs),
`pkg_agentic_coding`). Without agent mode the server answers with a plain policy
message and calls nothing — there is no hidden autonomous action. A non-agentic
prompt falls through to the normal symbolic answer even when tools are advertised.

**Offline reference client.** The repository ships its *own* deterministic client so
the whole loop runs in CI without a network or an external CLI:

```bash
formal-ai agent --transcript   # runs search → fetch → write → run → final offline
```

The driver and offline corpus live in
[`src/agentic_coding/`](../../src/agentic_coding/); the worked end-to-end loop is
[`examples/issue_468_agentic_loop.rs`](../../examples/issue_468_agentic_loop.rs).
External CLIs are pointed *at* this server as front-ends (the configs in
§4a–§4f); they are never embedded in the engine. See the
[issue #468 case study](../case-studies/issue-468/README.md) for the full design.

---

## 5. How the web UI reuses the same code

### 5a. One web bundle, two hosts

The desktop shell serves the **unmodified** `src/web` bundle — the same HTML,
CSS, and JavaScript published to GitHub Pages. There is no desktop-only fork of
the UI; the shell only adds a status bridge.

### 5b. Auto-detecting the local server

The browser cannot probe loopback ports, so detection happens through the
Electron preload bridge rather than a network scan. `desktop/preload.cjs`
exposes `window.FormalAiDesktop.getStatus()` (via `contextBridge`, with
`contextIsolation: true` / `nodeIntegration: false`). On startup the web app
calls it:

- If the status reports `apiReady` **and** an `apiBase`, the UI sends chat to
  `${apiBase}/v1/chat/completions` (server mode).
- Otherwise it stays on the in-process engine (default mode).

Because detection is gated behind the desktop bridge and the server binds
`127.0.0.1`, the public web build is never exposed to a local API and the
desktop never reaches across the network for one.

### 5c. Local database sync (R5c)

When server mode is on, the desktop keeps the browser memory (the IndexedDB
event log) and the native store in step automatically, so a conversation started
in one surface continues in the other without a manual export/import.

- The native side is [`src/memory_sync.rs`](../../src/memory_sync.rs): a
  file-backed `demo_memory` log (`SyncStore`) with a union-by-id merge
  (`merge_union_by_id`) where incoming non-empty fields win, exposed over the
  `/v1/memory`, `/v1/memory/since`, and `/v1/memory/import` endpoints above.
- The desktop client is
  [`desktop/lib/memory-sync.cjs`](../../desktop/lib/memory-sync.cjs), wired into
  the Electron main process as `formalAiDesktop:syncMemory`. After each turn the
  web app pushes its newest events with `POST /v1/memory/import`, then pulls the
  delta with `GET /v1/memory/since?event=<id>` and folds it back into IndexedDB,
  advancing a per-surface watermark so only unseen events cross the wire.

Payloads stay Links Notation (`demo_memory`); REST is only the transport (R7).

### 5d. Local-execution routing + Docker sandbox (R5d)

When the agent has side effects — web fetches, tool calls, shell commands, code
execution — and server mode is on, those run through the **local** app, not a
remote service. The dispatcher is
[`desktop/lib/tool-router.cjs`](../../desktop/lib/tool-router.cjs), exposed to the
renderer through `formalAiDesktop:invokeTool` /
`formalAiDesktop:setToolGrants`.

- **Default-deny.** No tool runs until the user grants it through the existing
  desktop permission gate. A denied call returns a *structured refusal*
  (`{ok:false, status:"refused", executed:false}`) and nothing executes.
- **Local I/O tools** — `http_fetch`, `url_navigate`, `read_local_file` — are
  served by the local process. `read_local_file` is confined to an allowed root;
  anything outside it is refused (`forbidden`).
- **Shell** runs on the host machine by default, after the same explicit grant,
  with output and logs returned through the tool result. A shell request may opt
  into Docker isolation with `input.isolation = "docker"`.
- **Sandboxed code tools** — `eval_js`, `code_exec`, and Docker-isolated `shell`
  requests — run inside the `konard/box-dind:2.1.1` Docker sandbox (the same
  inner-Docker image the Telegram microservice uses), with logs captured to a
  local path. If Docker is unavailable the sandboxed call is refused
  (`sandbox_unavailable`) rather than run unsandboxed.

This mirrors the `docker_microservice` environment in
[`data/seed/environments.lino`](../../data/seed/environments.lino) and keeps every
side effect on the local machine, with Docker as an explicit sandboxing target,
and behind an explicit grant.

---

## 6. Mode summary

| | In-process (default) | Server (opt-in) |
| --- | --- | --- |
| How to enable | nothing — it's the default | `FORMAL_AI_DESKTOP_SERVER=1`, or run `formal-ai serve` |
| Listening port | static file server only | `127.0.0.1:<port>` multi-protocol API |
| Chat path | in-browser engine | `POST /v1/chat/completions` desktop alias |
| External CLIs | - | `codex` / `agent` through `/api/openai/v1`, `claude` through `/api/anthropic/v1`, Gemini/Vertex through native Google-shaped routes |
| Memory | browser IndexedDB only | IndexedDB ⇄ native store sync ([§5c](#5c-local-database-sync-r5c)) |
| Tool / code execution | — | local process + `konard/box-dind` sandbox, default-deny ([§5d](#5d-local-execution-routing--docker-sandbox-r5d)) |
| Status badge | “in-process” | “API local” |

---

## Related

- [/download landing page](https://link-assistant.github.io/formal-ai/download/) — installers + checksums
- [Issue #347 case study](../case-studies/issue-347/README.md) — requirements, prior art, decisions
- [Issue #347 roadmap](../case-studies/issue-347/ROADMAP.md) — implementation roadmap for the local DB sync, request routing + sandbox, Links-Notation REST + LinksQL, and first-party Anthropic adapter
- [Links Notation](https://github.com/link-foundation/lino) — the internal data format (R7)
