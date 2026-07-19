# formal-ai

[![CI/CD Pipeline](https://github.com/link-assistant/formal-ai/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/link-assistant/formal-ai/actions/workflows/release.yml)
[![Desktop Release](https://github.com/link-assistant/formal-ai/actions/workflows/desktop-release.yml/badge.svg?branch=main)](https://github.com/link-assistant/formal-ai/actions/workflows/desktop-release.yml)
[![Crates.io](https://img.shields.io/crates/v/formal-ai?label=crates.io&style=flat)](https://crates.io/crates/formal-ai)
[![Docs.rs](https://img.shields.io/docsrs/formal-ai?label=docs.rs&style=flat)](https://docs.rs/formal-ai)
[![Rust Version](https://img.shields.io/badge/rust-1.96%2B-blue.svg)](https://www.rust-lang.org/)
[![Codecov](https://codecov.io/gh/link-assistant/formal-ai/branch/main/graph/badge.svg)](https://codecov.io/gh/link-assistant/formal-ai)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

Formal AI is a Rust implementation of a symbolic, deterministic assistant that exposes OpenAI-shaped interfaces without neural-network inference.

It belongs to the tradition of [symbolic artificial intelligence](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence) (a.k.a. GOFAI): its knowledge is an inspectable [semantic network](https://en.wikipedia.org/wiki/Semantic_network) of human-readable links rather than hidden neural weights. The case study in [docs/case-studies/issue-451](docs/case-studies/issue-451/README.md) maps the field's best practices onto this associative stack; the design study in [docs/case-studies/issue-649](docs/case-studies/issue-649/README.md) audits how the same stack expresses symbolic **world models** — a current-state and target-state context, their difference, context merge/split, and predicting the consequences of an action, all as links networks rather than embeddings; and the study in [docs/case-studies/issue-686](docs/case-studies/issue-686/README.md) adds **usage-weighted persistence** of meta-language expressions — counting reads and writes and incoming/outgoing link degree so the most used, most changed, and most connected knowledge persists longest, all as a links network.

The current implementation covers the surface area requested in issue #1:

- library API for symbolic prompt handling
- CLI chat command
- HTTP API server with protocol namespaces under `/api/openai/v1`,
  `/api/anthropic/v1`, `/api/gemini/v1beta`, and `/api/vertex/v1`
- Telegram bot CLI with long polling by default and an opt-in webhook server, configured through [`lino-arguments`](https://github.com/link-foundation/lino-arguments)
- human-readable Links Notation knowledge and dataset export through `lino-objects-codec`
- Prepared Docker-in-Docker Telegram bot image published as `ghcr.io/link-assistant/formal-ai:latest` and based on `konard/box-dind:2.1.1`
- GitHub Pages markdown chat demo backed by a Rust-generated WebAssembly worker
- Electron desktop shell that starts the local Rust HTTP API and reuses the web chat
- VS Code extension (desktop **and** web/`vscode.dev`) that embeds the same chat in a Webview around the same HTTP/web boundary

Project direction is tracked in [VISION.md](VISION.md), [GOALS.md](GOALS.md), and [NON-GOALS.md](NON-GOALS.md). Who the project is for, what pain it closes, and the concrete user journeys it supports today (plus the ones it could support next) are documented in [docs/USER-JOURNEYS.md](docs/USER-JOURNEYS.md). Implementation progress against the vision is tracked in [ROADMAP.md](ROADMAP.md). The issue #12 synthesis is in [docs/case-studies/issue-12/README.md](docs/case-studies/issue-12/README.md).

## Install

Every interface has a dedicated landing page on the
[site](https://link-assistant.github.io/formal-ai/) with copy-paste install
instructions, and a single universal installer covers all of them. The script is
published at [`scripts/install.sh`](scripts/install.sh) (POSIX `sh`, for
macOS/Linux) and [`scripts/install.ps1`](scripts/install.ps1) (PowerShell, for
Windows). Pass a target to choose what to install:

```bash
# Desktop app (default), VS Code extension, the CLI, the Telegram bot, or all:
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- desktop
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- vscode
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- cli
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- telegram
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- all
```

The Telegram bot ships inside the CLI, so the `telegram` target installs the CLI;
then run `formal-ai telegram` with a `@BotFather` token (see the
[Telegram page](https://link-assistant.github.io/formal-ai/telegram/)).

```powershell
# Windows PowerShell (set FORMAL_AI_INSTALL_TARGET to pick a target):
$env:FORMAL_AI_INSTALL_TARGET='vscode'; irm https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1 | iex
```

| Interface | Landing page | Notes |
| --- | --- | --- |
| Desktop app | [`/download/`](https://link-assistant.github.io/formal-ai/download/) | Electron shell with one-click services and a one-click VS Code extension install in Settings. |
| VS Code extension | [`/vscode/`](https://link-assistant.github.io/formal-ai/vscode/) | Not on the Marketplace yet, so install it manually — the one-liner above, a downloaded `.vsix` ("VS Code Extension only" mode), or one click from the desktop app. |
| CLI | [`/cli/`](https://link-assistant.github.io/formal-ai/cli/) | `cargo install formal-ai` or the universal installer. |
| Telegram bot | [`/telegram/`](https://link-assistant.github.io/formal-ai/telegram/) | `telegram` installer target (installs the CLI that powers the bot); needs a `@BotFather` token. |

## Universal Problem-Solving Algorithm

Every prompt — a greeting, a math question, a translation, a "write a program"
request, an agent action, or something the assistant has never seen — walks the
same loop. The skeleton is **decompose the problem into tasks, derive a test
(experiment) for each task, draft candidate solutions, select the smallest
sufficient draft, and compose the drafts back into one solution**:

![Universal Problem Solving Algorithm: a problem fans out into tasks, each task into tests, each test into candidate drafts, and the selected drafts compose back into a single solution](docs/assets/universal-problem-solving-algorithm.jpg)

This is not a separate subsystem bolted onto a catalogue of intents — it is the
main solver path. `src/solver.rs::UniversalSolver` runs the same 11-step loop
for every impulse:

| Diagram stage | Loop steps in `src/solver.rs` |
| --- | --- |
| `Problem` | 1. Impulse, 2. Formalization (Links-Notation intent), 3. Context, 4. History lookup |
| `decomposition → tasks` | 5. Decomposition into sub-impulses (`record_decomposition`) |
| `experiment → tests` | 6. TDD-style validation (`record_validation`) |
| `selection → drafts` | 7. Solution synthesis (`record_candidates`), 8. Combination of the smallest sufficient candidate |
| `composition → solutions → Solution` | 9. Verification, 10. Simplification, 11. Documentation with a `trace:` pointer |

Every step appends its own event to the append-only log, so the chat can answer
"why did you do that?" from the recorded experience. The full algorithm,
including the reasoning-under-unknowns path (`src/solver_unknown_reasoning.rs`)
and intent formalization (`src/intent_formalization.rs`), is documented in
[VISION.md](VISION.md#universal-problem-solving-algorithm) and
[ARCHITECTURE.md](ARCHITECTURE.md). How much of the vision is built versus still
planned — including the open industry-benchmark coverage gap — is tracked in
[ROADMAP.md](ROADMAP.md). Every benchmark suite the repository has ever touched
is catalogued in [docs/benchmarks.md](docs/benchmarks.md), and the grounded
meta-algorithm that reproduces a topic's Rust handler on demand is described in
[docs/meta-algorithm.md](docs/meta-algorithm.md).

## Quick Start

```bash
cargo run -- chat --prompt "Hi"
cargo run -- chat --prompt "Write me hello world program in Rust" --format chat
cargo run -- chat --prompt "What is 8% of $50?"
cargo run -- chat --prompt "Посчитай 1000 рублей в долларах"
cargo run -- dataset
rust-script scripts/mine-hive-mind-dataset.rs --plan
cargo run -- serve --host 127.0.0.1 --port 8080
npm install --prefix desktop
npm run desktop:dev
npm run vscode:test                                                    # VS Code extension node tests
npm run vscode:dev                                                     # run the extension in a web host (vscode-test-web)
TELEGRAM_BOT_TOKEN=123:abc cargo run -- telegram                       # long polling (default)
cargo run -- telegram --mode webhook --host 127.0.0.1 --port 8080      # webhook server (opt-in)
rust-script scripts/download-datasets.rs
experiments/verify-hello-world-examples.sh
```

Example API call:

```bash
curl -s http://127.0.0.1:8080/api/openai/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Hi"}]}'
```

The canonical model id is `formal-ai`. The API also accepts
`@link-assistant/formal-ai`, `link-assistant/formal-ai`, `formal-ai-latest`,
and `latest` as case-insensitive aliases, and responses return `formal-ai`.

To require bearer authentication on `/api/*` and `/v1/*` routes, set
`FORMAL_AI_API_BEARER_TOKEN` before starting the server and send the matching
bearer token or protocol API-key header:

```bash
FORMAL_AI_API_BEARER_TOKEN=local-test-token cargo run -- serve --host 127.0.0.1 --port 8080
curl -s http://127.0.0.1:8080/api/openai/v1/models \
  -H 'authorization: Bearer local-test-token'
```

Model discovery reports the context window from real disk capacity instead of a
fixed token ceiling. The server divides the free bytes on the filesystem that
contains the shared memory store by an average UTF-8 width of 2 bytes per
character. It reports the memory file size the same way as used context. Set
`FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR` to a positive integer to tune that estimate;
`FORMAL_AI_MEMORY_PATH` selects the measured memory store. Without that override,
the measured path is `~/.formal-ai/memory.lino` on Unix/macOS or
`%APPDATA%\formal-ai\memory.lino` on Windows. OpenAI model metadata,
generated Codex catalogs, Gemini/Vertex model metadata, and Anthropic responses
include the same `context` object with `context_window_tokens`,
`context_used_tokens`, `context_used_fraction`, `disk_free_bytes`,
`memory_used_bytes`, and `avg_utf8_bytes_per_char`. Usage cost remains zero.

### Reasoning traces over the API

Every symbolic answer still includes the structured `thinking_steps` trace. The
OpenAI-compatible surfaces also project the same trace into the standard fields
most clients display:

```bash
# Chat Completions, non-streaming: assistant reasoning_content
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Hi"}]}' \
  | jq -r '.choices[0].message.reasoning_content'
```

```bash
# Chat Completions, streaming: choices[].delta.reasoning_content
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Hi"}],"stream":true}' \
  | grep '"reasoning_content"'
```

```bash
# Responses, streaming: reasoning summary events before output text events
curl -N http://127.0.0.1:8080/v1/responses \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","input":"Hi","stream":true}' \
  | grep -E 'response.reasoning_summary_text.delta|response.output_text.delta'
```

Anthropic extended thinking remains opt-in, matching Anthropic's request shape:

```bash
curl -s http://127.0.0.1:8080/v1/messages \
  -H 'content-type: application/json' \
  -d '{"model":"formal-ai","max_tokens":128,"thinking":{"type":"enabled","budget_tokens":1024},"messages":[{"role":"user","content":"Hi"}]}' \
  | jq '.content[0]'
```

The native CLI also has a visible thinking mode:

```bash
cargo run -- chat --thinking --prompt "Hi"
```

## Agentic AI Tools

Run the local HTTP server before connecting terminal agents. The server binds to
loopback in these examples and exposes the same symbolic engine through
protocol-specific API namespaces:

```bash
cargo run -- serve --host 127.0.0.1 --port 8080
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/api/openai/v1/models
curl -s http://127.0.0.1:8080/api/gemini/v1beta/models
```

Primary routes live under `/api/<protocol>/...`: OpenAI at
`/api/openai/v1`, Anthropic at `/api/anthropic/v1`, Gemini at
`/api/gemini/v1beta`, and Vertex at `/api/vertex/v1`. Existing `/v1/models`,
`/v1/chat/completions`, `/v1/responses`, and `/v1/messages` aliases remain for
older local configs.

If you enabled bearer auth, export the same value for the CLI you connect:

```bash
export FORMAL_AI_API_KEY="local-test-token"
```

When no bearer token is configured, any non-empty API key value is enough for
clients that require one. Keep the server on `127.0.0.1` unless you are
deliberately exposing it behind your own authentication boundary.

### `formal-ai with` / `with-formal-ai`

The wrapper command reads client templates from
`data/seed/client-integrations.lino`, adds the right per-tool environment or
config, and then runs the external CLI with the remaining arguments unchanged:

```bash
formal-ai with codex "hi"
formal-ai with opencode run "hi"
formal-ai with agent -p "hi"
formal-ai with cursor -p "hi"
formal-ai with gemini -p "hi"
formal-ai with claude -p "hi"
formal-ai with qwen -p "hi"
formal-ai with grok -p "hi"
formal-ai with aider --message "hi"
# Hi, how may I help you?
```

When the loopback port is idle, the wrapper starts a temporary
`formal-ai serve --agent-mode`, prints a security notice, and tears it down when
the CLI exits. It reuses an existing listener. Use `--no-start-server` to require
an already-running server. Use `--base-url` when the server is not on
`http://127.0.0.1:8080`; the wrapper
adds the tool's protocol path such as `/api/openai/v1` or `/api/gemini` from
seed data. `--protocol vertex` switches Gemini-shaped setup to
`GOOGLE_VERTEX_BASE_URL` and `/api/vertex`.

Cursor CLI uses the MCP path instead of a custom model base URL. For a one-shot
run, the wrapper launches the `cursor-agent` binary with a temporary
`~/.cursor/mcp.json` that registers the local `/mcp` endpoint; the server
exposes `formal_ai_chat` and instructs Cursor to use it for each request. Both
interactive mode and headless `-p` mode are supported.

The existing explicit form remains supported:

```bash
formal-ai with --start-server codex "hi"
```

For one-shot Gemini runs, the wrapper also uses a temporary `GEMINI_CLI_HOME`
with API-key auth selected and workspace trust enabled. That keeps cached
OAuth settings from `~/.gemini` from taking over the invocation.

For one-shot Agent CLI runs, the wrapper injects the OpenCode-compatible
provider JSON through `LINK_ASSISTANT_AGENT_CONFIG_CONTENT`, so no temporary
config file is needed. It also passes `--no-summarize-session`; use
`--summarize` (alias `--keep-summarization`) to retain the client's default.

Every one-shot integration uses only command-line overrides, environment
variables, inline config, or a temporary config/home directory. Persistent tool
configuration is written only by explicit `--global` runs. Agent CLI compaction
is pinned to the Formal AI model rather than a remote fallback.

For one-shot Codex runs, the wrapper starts from
`codex exec --skip-git-repo-check --sandbox read-only` and injects the Responses
provider overrides through `-c` before appending your remaining arguments. It
also generates a model catalog inside the temporary Codex home and passes it as
`model_catalog_json`, so Codex recognizes the Formal AI model's context window
and capabilities without a missing-metadata warning.

For permanent setup, use the standalone wrapper or the subcommand with `-g`.
It backs up the original file next to the edited config, merges the Formal AI
provider without removing unrelated settings, and can restore the backup:

```bash
with-formal-ai -g codex
with-formal-ai -g opencode
with-formal-ai -g agent
with-formal-ai -g cursor
with-formal-ai -g gemini
with-formal-ai -g claude
with-formal-ai -g qwen
with-formal-ai -g grok
with-formal-ai -g aider
with-formal-ai -g --all
with-formal-ai -g --undo codex
```

Persistent targets are `~/.codex/config.toml`,
`~/.codex/formal-ai-model-catalog.json`,
`~/.config/opencode/opencode.json`,
`~/.config/link-assistant-agent/opencode.json`, and a managed block in
`~/.profile` for environment-configured clients, plus `~/.cursor/mcp.json` for
Cursor. Re-running `-g` is idempotent.

### Codex CLI

Codex 0.142+ does not support `wire_api = "chat"`. Custom providers use the
Responses wire API and Codex always streams, so keep `wire_api = "responses"`
and point Codex at the server's `/api/openai/v1` base URL in
`~/.codex/config.toml`:

```toml
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

For a one-shot invocation without editing `~/.codex/config.toml`, pass the same
provider block through `-c`:

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

### Claude Code

Claude Code talks to the Anthropic Messages API. formal-ai serves that adapter
at `/api/anthropic/v1/messages`, so use the Anthropic namespace as
`ANTHROPIC_BASE_URL`:

```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080/api/anthropic"
export ANTHROPIC_API_KEY="${FORMAL_AI_API_KEY:-local-test-token}"
claude
```

### OpenCode

OpenCode can call formal-ai through its OpenAI-compatible provider package,
which targets `/api/openai/v1/chat/completions`. Add a local provider in
`~/.config/opencode/opencode.json`. This example uses the provider id
`formalai`, so the model selector is `formalai/formal-ai`:

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

### Gemini CLI

Gemini-compatible clients can use the native Gemini `generateContent` and
`streamGenerateContent` routes under `/api/gemini/v1beta`:

```bash
export GEMINI_CLI_HOME="$(mktemp -d)"
mkdir -p "${GEMINI_CLI_HOME}/.gemini"
printf '%s\n' '{"security":{"auth":{"selectedType":"gemini-api-key"}}}' \
  > "${GEMINI_CLI_HOME}/.gemini/settings.json"
export GEMINI_API_KEY="${FORMAL_AI_API_KEY:-local-test-token}"
export GEMINI_DEFAULT_AUTH_TYPE="gemini-api-key"
export GEMINI_CLI_TRUST_WORKSPACE="true"
export GOOGLE_GEMINI_BASE_URL="http://127.0.0.1:8080/api/gemini"
gemini -m formal-ai -p "hi"
```

Equivalent raw request:

```bash
curl -s http://127.0.0.1:8080/api/gemini/v1beta/models/formal-ai:generateContent \
  -H 'content-type: application/json' \
  -d '{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}'
```

### Vertex AI Clients

Vertex-shaped clients can use the Google publisher-model route under
`/api/vertex/v1`:

```bash
export GOOGLE_VERTEX_BASE_URL="http://127.0.0.1:8080/api/vertex"
curl -s \
  http://127.0.0.1:8080/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models/formal-ai:generateContent \
  -H 'content-type: application/json' \
  -H "x-goog-api-key: ${FORMAL_AI_API_KEY:-local-test-token}" \
  -d '{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}'
```

### Link Assistant Agent CLI

The Link Assistant Agent CLI accepts OpenCode-style provider/model selection.
Start the local OpenAI-compatible server with agent-mode enabled, then use the
same provider shape in `~/.config/link-assistant-agent/opencode.json` and select
the `formalai/formal-ai` model:

```bash
formal-ai serve --agent-mode --host 127.0.0.1 --port 8080
```

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
agent --model formalai/formal-ai --permission-mode plan -p \
  "run ls to list files here"
```

Run autonomous coding CLIs only in a repository, VM, or container where their
file and shell actions are acceptable. formal-ai's local server answers model
requests; it does not sandbox the client process that is driving tools. In
agent mode, a directory-listing request like the example above makes the server
emit a `bash` / `shell` / `run_command` `tool_calls` turn with
`{"command":"ls"}`; the Agent CLI executes or refuses that command according to
its own permission mode. Natural-language paraphrases such as "what files are
in this folder?" and "show me the contents of this directory" route to the same
read-only listing command. Use `--permission-mode plan` when read-only shell
commands such as `ls` may run, and use hard `--read-only` when shell execution
should be disabled entirely.

Formal AI keeps each client tool result unchanged in the conversation transcript,
then presents a normalized, localized answer after the client returns it. This
means a later turn can ask for the full result, a numbered line, or a URL without
losing the original bytes. To retain those tool calls beyond the current client
conversation, start the server with `FORMAL_AI_MEMORY_PATH=memory.lino`; completed
tool names, arguments, and raw outputs are then appended to the durable memory log.

Example Telegram webhook update:

```bash
curl -s http://127.0.0.1:8080/telegram/webhook \
  -H 'content-type: application/json' \
  -d '{"update_id":1,"message":{"message_id":7,"date":1,"chat":{"id":42,"type":"private"},"text":"Write me hello world program in Rust"}}'
```

Docker-in-Docker Telegram bot image:

```bash
TELEGRAM_BOT_TOKEN=123:abc docker compose up

docker run --rm --privileged \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  -v formal-ai-telegram-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest

# Local build fallback:
docker build -t formal-ai .
docker run --rm --privileged \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -v formal-ai-telegram-docker:/var/lib/docker \
  formal-ai

# Preferred when Sysbox is available:
docker run --rm --runtime=sysbox-runc \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -v formal-ai-telegram-docker:/var/lib/docker \
  formal-ai
```

Prebuilt image quick start: the released image is
`ghcr.io/link-assistant/formal-ai:latest`, so the only required setting for a
polling Telegram bot is `TELEGRAM_BOT_TOKEN`. The root `compose.yaml` uses that
image by default and preserves the inner Docker daemon under the named
`formal-ai-telegram-docker` volume. Set `FORMAL_AI_DOCKER_IMAGE` to run a locally
built image or an optional Docker Hub mirror with the same compose file.

The same image and compose file also run the **OpenAI-compatible API server** for
agentic mode and the idle **Agent CLI environment**, under opt-in Compose
profiles so `docker compose up` keeps starting only the Telegram bot:

```bash
TELEGRAM_BOT_TOKEN=123:abc docker compose up -d   # Telegram bot only (default)
docker compose --profile server up -d             # OpenAI-compatible server on 127.0.0.1:8080
docker compose --profile agent up -d              # agent + agent-commander environment
docker compose --profile all up -d                # all services
```

All three containers (`formal-ai-telegram`, `formal-ai-server`, and
`formal-ai-agent`) are the **exact same ones the desktop app manages with one
click**. They bind the host's `~/.formal-ai` directory to `/root/.formal-ai`,
so Telegram, API, Agent CLI, desktop, and host CLI writes converge on the same
`memory.lino`; their inner-Docker volumes remain separate. See
[One-click services and agent environment](docs/desktop/service-control.md) for
the full desktop + server walkthrough.

The root image is intentionally the only supported Docker runtime: it inherits
from `konard/box-dind:2.1.1`, starts `/usr/local/bin/dind-entrypoint.sh`, and
defaults to `formal-ai telegram --mode polling`. Do not bind-mount the host
`/var/run/docker.sock`; the image expects its own inner Docker daemon and uses
`/var/lib/docker` for that daemon's storage.

Verify the container contract and the `start-command` Docker isolation wrapper:

```bash
docker run --rm --privileged formal-ai verify-formal-ai-dind
docker run --rm --privileged formal-ai bash -lc \
  '$ --isolated docker --auto-remove-docker-container -- echo formal-ai-dind-ok'
```

The static demo lives in `src/web/index.html`. Serve it from a local web server or GitHub Pages so the WebAssembly worker can be fetched by the browser. The demo starts with a user greeting, renders markdown in messages, previews markdown input, and includes a randomized dialog mode for hello-world prompts across several programming languages. Rust/WASM owns parity-sensitive worker primitives such as prompt normalization, language detection, arithmetic evaluation, stable ids, intent-route matching, unknown-answer variation, and web-search fusion; JavaScript remains responsible for UI state, seed fetching, browser fetch/CORS orchestration, and no-WASM fallbacks. Browser JavaScript dependencies are prebundled with Bun into `src/web/vendor.bundle.js`; run `bun install` once and `bun run build:web` after changing web dependencies. The companion connectivity diagnostics page lives in `src/web/tests/index.html` and is deployed at `/formal-ai/tests/`; it checks direct browser fetches, public knowledge APIs, iframe embeddability, and a configurable local `web-capture` proxy.

### Desktop app

The desktop app lives in [`desktop/`](desktop/) and follows the same boundary as the browser and HTTP server. Electron starts a loopback `formal-ai serve` process, serves `src/web/` from a local static server, and loads the existing chat UI with a preload bridge that reports the API, graph, memory, and permission status.

```bash
npm install --prefix desktop
cargo build
npm run desktop:dev
npm run desktop:smoke
```

In desktop mode, prompt sends use `POST /v1/chat/completions` on the local Rust API, and the network link points to `GET /v1/graph`. The same **Export memory** and **Import memory** controls read and write the full `formal_ai_bundle`; no separate desktop memory format exists. Agent mode remains off by default, and the desktop sidebar shows whether agent/tool-call actions are permission-gated or explicitly opted in.

Persistent memory needs no configuration. Formal AI creates
`~/.formal-ai/memory.lino` on Unix/macOS or
`%APPDATA%\formal-ai\memory.lino` on Windows and uses it for the CLI, local
server, desktop shell, dreaming worker, and VS Code desktop host. Set
`FORMAL_AI_MEMORY_PATH` only when an explicit alternate file is required.

Packaging starts from the same shell:

```bash
cargo build --release
npm --prefix desktop run build
```

Set `FORMAL_AI_DESKTOP_BINARY=/path/to/formal-ai` before packaging to bundle a specific binary. Release builds copy the web assets and seed mirror into `desktop/dist-web/`, copy the binary into `desktop/bin/` when available, and produce OS artifacts under `desktop/release/`.

#### One-click services and agent environment

The desktop sidebar has a **Services** panel that starts, stops, or installs the
prepared Docker containers with a single click:

- **Telegram bot** (`formal-ai-telegram`) — runs the image's default polling bot.
  An inline field captures `TELEGRAM_BOT_TOKEN`; the bot will not start without it.
- **OpenAI-compatible server** (`formal-ai-server`) — runs `formal-ai serve` for
  agentic mode and publishes `http://127.0.0.1:8080`.
- **Agent environment** (`formal-ai-agent`) — pulls the prepared image, recreates
  an idle container, and health-checks `formal-ai --version`, `agent --version`,
  and `start-agent --help` inside the container.

Each row shows a live Docker-backed indicator and **Start**/**Stop** or
**Install agent environment** controls. The lifecycle logic lives in the testable
[`desktop/lib/service-control.cjs`](desktop/lib/service-control.cjs) module, wired
into the Electron main process over IPC (`formalAiDesktop:serviceStatus` /
`startService` / `installAgentEnvironment` / `stopService`) and exposed to the
renderer through the preload bridge. Docker is required; the panel disables
itself with a clear note when Docker is unavailable.

The **same containers** run on a server from the root `compose.yaml` with one
line (`docker compose --profile all up -d`). Autonomous tools are run only inside
the Formal-AI container path; the desktop provider never invokes host
`agent`/`claude`/`codex` binaries. The complete desktop + server walkthrough —
configuration, raw `docker run` equivalents, health checks, and the isolation
model — is in
[docs/desktop/service-control.md](docs/desktop/service-control.md).

### VS Code extension

The VS Code extension lives in [`vscode/`](vscode/) and embeds the same web chat in a Webview around the same HTTP/web boundary as the browser, the HTTP server, and the desktop shell. It ships **two hosts from one manifest** so the same extension runs on the desktop and in the browser:

- **Desktop / remote (Node) host** — [`src/extension.node.cjs`](vscode/src/extension.node.cjs) reports `shell: "VS Code"`. With the opt-in `formal-ai.server.enabled` setting it starts a loopback `formal-ai serve` process and routes prompt sends through `POST /v1/chat/completions`, just like the desktop shell; it can also drive Docker code execution (`formal-ai.docker.image`). It reuses the desktop tool-router and memory-sync helpers.
- **Web (Web Worker) host** — [`src/extension.web.cjs`](vscode/src/extension.web.cjs) reports `shell: "VS Code Web"` and runs on `vscode.dev` / `github.dev`. The Web Worker host cannot spawn a process, open a socket, or touch `child_process`/`fs`, so it stays on the in-process WebAssembly symbolic engine — no local server, no Docker — while exposing the same chat, network, memory, and permission surfaces.

```bash
npm run vscode:test     # node:test unit suite + static smoke check (no install needed)
npm run vscode:dev      # launch in a browser host via @vscode/test-web
npm run vscode:smoke    # static manifest/contract smoke check
npm run vscode:package  # produce a .vsix (runs prepare-resources first)
```

The web app labels both hosts **"VS Code"** in the status line and sidebar, and only routes to the local server when it is genuinely ready (`apiReady && apiBase`); otherwise it falls back to the in-process engine and reads `VS Code - in-process`. The same **Export memory** / **Import memory** controls read and write the full `formal_ai_bundle`; agent mode is off by default and tool calls are permission-gated until the user opts in. Settings (`formal-ai.server.*`, `formal-ai.docker.image`, `formal-ai.tools.allowByDefault`, `formal-ai.agent.defaultOn`) map directly onto that status shape.

Packaging mirrors the desktop flow: `prepare-resources` copies `src/web/` into `vscode/dist-web/` (with the seed mirror at `vscode/dist-web/seed/`) and the desktop `lib/` helpers into `vscode/src/lib/vendor/`; both generated trees are git-ignored. See [docs/vscode/extension.md](docs/vscode/extension.md) for the full architecture, the Webview sandbox reconciliation (CSP nonce, same-origin Worker bootstrap, seed rebasing), and the honest list of what is and isn't verifiable inside the test sandbox.

### Full-memory export and import

Every interface produces the same self-contained Links Notation document by default. In the browser, the **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — the entire seed (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, and the full append-only event log — so a single click is enough to reconstitute the session. **Import memory** auto-detects bundle vs legacy `demo_memory` files and surfaces migration suggestions when the imported seed version differs from the running app's. The CLI matches:

Native Rust builds now select `doublets-rs` by default through the
`doublets-native` feature. Links Notation stays the recovery and migration
projection: existing `demo_memory` logs and full `formal_ai_bundle` exports
import into the native store, export back to deterministic `.lino`, and can
still be handled by compiling with `--no-default-features` when a pure
`MemoryStore` projection is needed.

```bash
cargo run -- memory export --from memory.lino --path full.lino           # default: full bundle
cargo run -- memory export --from memory.lino --path events.lino --events-only  # legacy demo_memory
cargo run -- memory import --path full.lino --into memory.lino           # accepts either format
cargo run -- memory show --path memory.lino                              # print every recorded event
cargo run -- memory query --path memory.lino --query "Find Rust in another conversation"
cargo run -- memory dream --path memory.lino                             # plan low-priority cleanup
cargo run -- memory dream --path memory.lino --storage-capacity-bytes 1000000 --free-bytes 50000
cargo run -- memory dream --path memory.lino --apply --confirm           # persist learning; cleanup asks consent
cargo run -- memory purge-deleted --path memory.lino --backup before-purge.lino --confirm
cargo run -- memory reset --path memory.lino --backup before-reset.lino --confirm
cargo run -- bundle export --path bundle.lino --memory memory.lino
cargo run -- bundle import --path bundle.lino --into memory.lino
```

Memory normally remains append-only: deleting a conversation first records a
`conversation_deleted` event and hides the thread. The explicit
`purge-deleted` maintenance action physically removes every event for those
soft-deleted conversations, and `reset` clears the event log completely. Both
browser actions show an export-first prompt and then an irreversible
confirmation prompt. The CLI refuses both destructive commands unless
`--confirm` is present, and `--backup` writes a full `formal_ai_bundle` before
the memory file is changed.

`memory dream` is the default-on background maintenance planner from issue
#540. It follows memory links, recalculates cached/seed usage, proposes duplicate
recomputable cleanup, and measures the real filesystem to target a 20%
free-space reserve including the next incoming write. Dreaming also learns while
the core server or desktop is idle: it ranks frequent topics, reads multilingual
standing-requirement cues from data, derives candidate tasks, replays proposed
meta-algorithm amendments, and mines recurring task structures.
Only a passing replay may mark a specific test run as covered. Retained amendments are
read by
later chat and Responses requests, so similar future answers apply the learned
rule without the user repeating it.

The manual CLI remains plan-only unless `--apply --confirm` is supplied. The
default background runtime may retain amendments and patterns, but it never
removes links without a persisted auto-free-space choice. CLI/Electron prompts
persist acceptance or refusal, free only enough recomputable data for the next
operation, and recommend larger storage when the reserve cannot be met.

The Rust library re-exports the same helpers — `export_memory_full`, `import_memory_full`, `suggest_memory_migrations`, `BundleInfo`, `ParsedBundle` — so embedders writing their own surface get the same defaults. The prefilled **Report issue** link records the dialog as a single compact `U:`/`A:` code block and points to [`docs/upload-memory.md`](docs/upload-memory.md) for attaching the full memory export (GitHub Gist or `.zip` workflow, plus redaction reminders) instead of repeating those instructions inline.

### Teaching behavior in chat

The chat surface can explain and modify behavior rules without leaving the dialog. Behavior is surfaced as a series of `When X then Y` (or `When X do Y`) statements grouped by topic, and the same grammar can also update the dialog:

```text
List behavior rules
Покажи правила
Show behavior rule unknown
List all facts you know about yourself
When `Какая у тебя модель личности?` then `У меня символьная модель личности.`
When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`
```

`List behavior rules` shows the current built-in routing rules, grouped by topic (Greetings, Farewells, Identity, Capabilities, Hello-world programs, Unknown fallback) and rendered as `When X then Y` statements. `Show behavior rule unknown` renders one rule as Links Notation with its topic, intent, match condition, response, source, and the canonical `when_then` statement. The `When X then Y` and `When X do Y` forms (and the explicit `When I say ... answer ...` form) record an append-only, dialog-local override, so the same prompt can answer differently in that conversation. The grammar is recognized in English (`When ... then ...`, `When ... do ...`, `If I ask ... reply ...`), Russian (`Когда ... тогда ...`, `Когда ... делай ...`, `Если ... то ...`), Hindi (`जब ... तब ...`, `जब ... तो ...`), and Chinese (`当 ... 时 ...`, `当 ... 则 ...`). Use **Export memory** to preserve that rule message with the session, or **Report issue** when the fact or rule should become part of the built-in seed.

## Telegram Bot

The `formal-ai telegram` subcommand defaults to long polling and keeps the webhook server available as an opt-in mode. The CLI is configured through [`lino-arguments`](https://github.com/link-foundation/lino-arguments) (a clap-compatible derive), so every flag also reads from the matching environment variable and from `.lenv`/`.env` files in the working directory.

### Long polling (default)

```bash
export TELEGRAM_BOT_TOKEN=123:abc
cargo run -- telegram                                                   # polling by default
cargo run -- telegram --mode polling \
  --timeout 30 --limit 100 \
  --allowed-updates message,edited_message
```

The polling client shells out to `curl`, calls Telegram's `getUpdates`, advances the offset after each batch, and replies through `sendMessage` with HTML formatting. The same `FormalAiEngine` used by the library, HTTP API, and web demo is reused, so polling answers match the other surfaces.

The Docker image uses this polling command as its default `CMD`, so the
container starts the bot as soon as `TELEGRAM_BOT_TOKEN` is set. Commands that
need nested containers should run through the bundled `$` wrapper with
`--isolated docker`; `start-command` records command output and metadata under
`/tmp/start-command/logs/`. Issue #195 called this `--isolation docker`; the
current Link Foundation Start CLI documents the flag as `--isolated docker`.

### Webhook (opt-in)

```bash
cargo run -- telegram --mode webhook --host 127.0.0.1 --port 8080
# or equivalently for backwards compatibility:
cargo run -- serve --host 127.0.0.1 --port 8080

# Docker override for webhook mode:
docker run --rm --privileged -p 8080:8080 \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  formal-ai formal-ai telegram --mode webhook --host 0.0.0.0 --port 8080
```

Expose the server through HTTPS and register the endpoint with Telegram:

```bash
curl -s "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/setWebhook" \
  -d "url=https://example.com/telegram/webhook"
```

The webhook accepts Telegram `message`, `edited_message`, `channel_post`, and `edited_channel_post` updates. It returns a direct Telegram `sendMessage` response for both private chats and group/channel chat IDs, using Telegram HTML formatting so code blocks survive the chat surface. This implementation does not store a bot token or perform outbound Telegram API calls from the webhook path; large file attachments require an outbound bot-client layer.

## Rust Library

```rust
use formal_ai::{create_chat_completion, ChatCompletionRequest, ChatMessage, MessageContent};

let request = ChatCompletionRequest {
    model: None,
    messages: vec![ChatMessage {
        role: String::from("user"),
        content: MessageContent::Text(String::from("Hi")),
    }],
    stream: false,
    temperature: None,
};

let completion = create_chat_completion(&request);
assert_eq!(
    completion.choices[0].message.content.plain_text(),
    "Hi, how may I help you?"
);
```

## Current Symbolic Behavior

The engine normalizes a prompt, selects a deterministic symbolic rule, and returns the rule output with evidence link identifiers and indented Links Notation. It can also consume an explicit `ProbabilityStore`: append-only Bayesian-style evidence and Markov transition evidence rank symbolic candidate IDs before the temperature / clarify-vs-guess policy runs. This stays non-neural; evidence is Links Notation data with provenance, timestamps, cached-source fingerprints, and deterministic replay.

Seed rules currently cover:

- greetings and polite follow-ups: `Hi`, `Hello`, `Hey`, `I am fine, thank you`, `thanks`
- hello world requests for Rust, Python, JavaScript, TypeScript, Go, and C
- open-ended software artifact requests such as extensions, plugins, bots, apps, and tools, first returning a Links Notation meaning record with a requirement graph, subtasks, delivery mode, approval gates, reasoning, and plan steps, then returning language-aware starter domain code after the user approves the plan
- calculator-parsable math, unit, currency, percentage, and datetime prompts through `link-calculator`, with the local arithmetic evaluator retained for unsupported word-operator and binary-modulo syntax
- URL requests such as `Navigate to github.com`, `fetch example.com`, and `Сделай запрос к google.com`; navigation prompts check CORS-readable frame-policy metadata and render an iframe only when `X-Frame-Options` and CSP `frame-ancestors` do not block embedding, while explicit fetch prompts attempt a browser `fetch()` first and use the same frame-policy check before any embedded fallback
- web-search, information-search, and implicit research prompts such as `Search the web for Nikola Tesla`, `Найди яблоко в интернете`, `Найди информацию о Rust программировании`, `Rust programming के बारे में जानकारी खोजो`, `查找关于 Rust 编程的信息`, and `What is the most popular dataset for translation quality validation?`; the browser demo queries DuckDuckGo, Internet Archive, Wikipedia, Wikidata, and Wiktionary, then returns reciprocal-rank-fused links
- merged definition prompts such as `Merge Wikipedia definitions of IIR`, which combine localized definition blocks for the same seed/Wikidata concept, deduplicate repeated facts, and cite every source language; use `--definition-fusion auto`, `FORMAL_AI_DEFINITION_FUSION=auto`, or the browser Settings control to make plain prompts like `What is IIR?` use the same fusion path
- generic project lookups for GitHub/GitLab/Bitbucket repository URLs plus default-on promotion for matching `link-assistant`, `link-foundation`, and `linksplatform` projects such as `What is Hive Mind?`, `Что такое Hive Mind?`, and `What is link-cli?`; promoted answers are generated from `data/seed/projects.lino` through the deterministic `formalize → summarize → deformalize` pipeline in `src/summarization/` (split into `mod.rs`, `markdown.rs`, `dialog.rs`, `file.rs`, and `resource.rs`), which also drives README ingestion, repository-file summaries with recursive Markdown embedded-grammar formalization, recursive repository-resource summaries that generalize from files to whole folders (`resource.rs`, via the decompose → summarize → compose meta-algorithm loop bounded by the summarization mode ladder), multi-turn dialog summaries, and chat-title generation (see [ARCHITECTURE.md § 7.1](ARCHITECTURE.md#71-project-lookups-and-summarization))
- behavior-rule inspection and dialog-local rule updates through `List behavior rules` (grouped by topic, each rendered as a `When X then Y` statement), `Show behavior rule unknown`, and the multilingual `When ... then ...` / `When ... do ...` / `When I say ... answer ...` grammar
- unknown prompts, which return a larger learnable-rule fallback with exact commands for inspecting rules, teaching the current dialog, exporting memory, or reporting a missing built-in rule

Hello-world answers include execution metadata. Rust, Python, JavaScript, Go, and C examples are compiled or syntax-checked and run by the issue-8 local verification harness with captured output. TypeScript is returned with an explicit warning because no `tsc` toolchain is installed in the current repository runtime.

No GPU, neural network, remote model, or random sampling is used.

## Dataset Seeds

Issue #1 source indexes and seed prompts are stored as indented Links Notation in `data/`.

```bash
rust-script scripts/download-datasets.rs
rust-script scripts/check-file-size.rs
```

The generator writes source, greeting, hello-world, and demo-dialog records. `.lino` files are kept below 1500 lines and validated by the unit tests.

## Hive Mind Dataset Mining

Issue #115 adds an operator script for mining GitHub evidence from
`link-assistant/hive-mind` pull requests, issues, reviews, diffs, and Actions
logs into a case-study dataset. Keep this outside the seed tool registry; it is
a repository maintenance command, not an in-agent reasoning tool.

```bash
rust-script scripts/mine-hive-mind-dataset.rs --plan
rust-script scripts/mine-hive-mind-dataset.rs --collect
```

The script wraps `formal-ai github-logs plan|collect` with the focused Hive
Mind defaults used by `docs/case-studies/issue-115/`.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
rust-script scripts/check-file-size.rs
```

Decode an overlong prefilled GitHub issue URL into readable Markdown during
report-link triage:

```bash
rust-script scripts/decode-github-issue-url.rs --url 'https://github.com/link-assistant/formal-ai/issues/new?...'
```

See [REQUIREMENTS.md](REQUIREMENTS.md) for the cumulative requirement matrix (now alongside [VISION.md](VISION.md)) and [docs/case-studies/issue-1/README.md](docs/case-studies/issue-1/README.md) for the collected research and implementation plan.
