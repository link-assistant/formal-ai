# Desktop runtime: in-process agent and the optional server API

formal-ai Desktop is the Electron shell that packages the same web chat shipped to
[GitHub Pages](https://link-assistant.github.io/formal-ai/). It has two runtime modes:

1. **In-process agent (default).** No server, nothing listening — the reasoning
   agent runs inside the app, exactly like the in-browser demo.
2. **Local OpenAI-compatible server (opt-in).** A loopback HTTP API you can turn
   on to expose `POST /v1/chat/completions` to the bundled UI *and* to external
   coding CLIs such as [`claude`](https://github.com/anthropics/claude-code),
   [`codex`](https://github.com/openai/codex), and
   [`agent`](https://github.com/link-assistant/agent).

This document explains both modes, how to enable and configure the server, and
how the web UI reuses the same code in the browser and on the desktop.

> **Scope.** The server speaks **OpenAI-compatible REST only**. Every other
> exchange inside formal-ai (seed data, memory bundles, traces) prefers
> [Links Notation](https://github.com/link-foundation/lino). REST is the
> interop boundary; Links Notation is the internal format.

---

## 1. In-process agent (default)

When you launch the desktop app normally:

```bash
npm --prefix desktop run dev      # development
npm --prefix desktop run build    # packaged installers (build:linux / build:mac / build:win)
```

it starts **only** a static file server on a random loopback port to serve the
web bundle, and reports `mode: "in-process"` to the UI. No `formal-ai serve`
child process is spawned, so nothing binds an API port. The UI answers prompts
with the same engine the public web demo uses.

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
before launching. The shell then starts `formal-ai serve` on a free loopback
port, waits for `GET /health`, and routes chat through
`POST /v1/chat/completions`. The status badge switches to **“API local”**.

```bash
FORMAL_AI_DESKTOP_SERVER=1 npm --prefix desktop run dev
```

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
# {"status":"ok","model":"formal-symbolic-production"}
```

### Endpoints

| Method   | Path                   | Purpose                                          |
| -------- | ---------------------- | ------------------------------------------------ |
| `GET`    | `/health`              | Liveness probe (never requires a token)          |
| `GET`    | `/v1/models`           | OpenAI-style model list + advertised rate limits |
| `GET`    | `/v1/graph`            | Reasoning-graph nodes/edges for a trace          |
| `POST`   | `/v1/chat/completions` | OpenAI Chat Completions (supports `stream`)      |
| `POST`   | `/v1/responses`        | OpenAI Responses API                             |
| `OPTIONS`| *(any)*                | CORS preflight → `204 No Content`                |

The single model id is **`formal-symbolic-production`**. `/v1/models` advertises
a rate limit of 60 requests/min and 60,000 tokens/min.

---

## 3. Authentication

The standalone server reads an **optional** bearer token from the first non-empty
of these environment variables:

- `FORMAL_AI_API_BEARER_TOKEN`
- `FORMAL_AI_HTTP_BEARER_TOKEN`
- `FORMAL_AI_API_TOKEN`

Behaviour:

- **No token set** → `/v1/*` is open. Safe for loopback-only use.
- **Token set** → every `/v1/*` request must send `Authorization: Bearer <token>`;
  otherwise the server replies `401`. `GET /health` and `OPTIONS` never require a
  token.

```bash
FORMAL_AI_API_BEARER_TOKEN=sk-local-demo formal-ai serve --host 127.0.0.1 --port 8080
curl http://127.0.0.1:8080/v1/models -H "Authorization: Bearer sk-local-demo"
```

---

## 4. Point coding CLIs at the local server

All three CLIs below talk to the **same** `POST /v1/chat/completions` endpoint.
The examples assume the standalone server from [§2b](#2b-from-the-cli-standalone)
at `http://127.0.0.1:8080`. If you set a bearer token, use it as the API key;
otherwise any non-empty placeholder works because the loopback server ignores it.

### 4a. `codex` (OpenAI Codex CLI) — native, OpenAI-shaped

`codex` configures providers in `~/.codex/config.toml`. A provider needs a
`base_url`, the env var that holds the key (`env_key`), and `wire_api = "chat"`
for the Chat Completions protocol (which is what formal-ai exposes):

```toml
# ~/.codex/config.toml
model_provider = "formal-ai"
model = "formal-symbolic-production"

[model_providers.formal-ai]
name = "formal-ai local server"
base_url = "http://127.0.0.1:8080/v1"
env_key = "FORMAL_AI_API_KEY"
wire_api = "chat"
```

```bash
export FORMAL_AI_API_KEY="sk-local-demo"   # match your bearer token, or any placeholder
codex "summarise the reasoning graph for a greeting"
```

You can also select the provider per-invocation without editing the file:
`codex --config model_provider=formal-ai --config model=formal-symbolic-production`.

See the upstream
[Codex configuration reference](https://developers.openai.com/codex/config-reference)
for the full `model_providers` schema.

### 4b. `agent` (link-assistant/agent) — OpenAI-compatible client

[`agent`](https://github.com/link-assistant/agent) is an OpenAI-compatible
coding agent. Point it at the local server using the standard OpenAI client
environment variables, then select the formal-ai model:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
export OPENAI_API_KEY="sk-local-demo"       # match your bearer token, or any placeholder
agent --model formal-symbolic-production "explain the last trace"
```

`agent` documents its provider/model selection in
[`MODELS.md`](https://github.com/link-assistant/agent/blob/main/MODELS.md) and
its [README](https://github.com/link-assistant/agent#readme); consult those for
the exact provider override syntax in your version.

### 4c. `claude` (Anthropic Claude Code) — needs an adapter

[`claude`](https://github.com/anthropics/claude-code) speaks the **Anthropic
Messages** protocol (`/v1/messages`), not OpenAI Chat Completions. Its
`ANTHROPIC_BASE_URL` only routes to **Anthropic-protocol-compatible** backends,
so it cannot call formal-ai's OpenAI endpoint directly.

To use `claude` with formal-ai, run a translating proxy in front of the server
that converts Anthropic requests to OpenAI Chat Completions, then point
`claude` at the proxy. Options include
[LiteLLM](https://docs.litellm.ai/docs/tutorials/claude_responses_api) and
standalone adapters such as
[`anthropic-proxy`](https://github.com/maxnowack/anthropic-proxy) or
[`claude-code-proxy`](https://github.com/fuergaosi233/claude-code-proxy):

```bash
# 1. start formal-ai (OpenAI side)
formal-ai serve --host 127.0.0.1 --port 8080
# 2. start an Anthropic→OpenAI adapter that forwards to http://127.0.0.1:8080/v1
#    (configure the adapter per its own docs)
# 3. point Claude Code at the adapter
export ANTHROPIC_BASE_URL="http://127.0.0.1:<adapter-port>"
export ANTHROPIC_API_KEY="sk-local-demo"
claude
```

A first-party adapter is tracked on the
[issue #347 roadmap](../case-studies/issue-347/ROADMAP.md).

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

---

## 6. Mode summary

| | In-process (default) | Server (opt-in) |
| --- | --- | --- |
| How to enable | nothing — it's the default | `FORMAL_AI_DESKTOP_SERVER=1`, or run `formal-ai serve` |
| Listening port | static file server only | `127.0.0.1:<port>` OpenAI API |
| Chat path | in-browser engine | `POST /v1/chat/completions` |
| External CLIs | — | `codex` / `agent` natively, `claude` via adapter |
| Status badge | “in-process” | “API local” |

---

## Related

- [/download landing page](https://link-assistant.github.io/formal-ai/download/) — installers + checksums
- [Issue #347 case study](../case-studies/issue-347/README.md) — requirements, prior art, decisions
- [Issue #347 roadmap](../case-studies/issue-347/ROADMAP.md) — deferred items (DB sync, request routing, first-party Anthropic adapter)
- [Links Notation](https://github.com/link-foundation/lino) — the internal data format (R7)
