# formal-ai

Formal AI is a Rust implementation of a symbolic, deterministic assistant that exposes OpenAI-shaped interfaces without neural-network inference.

The current implementation covers the surface area requested in issue #1:

- library API for symbolic prompt handling
- CLI chat command
- HTTP API server with `/v1/chat/completions` and `/v1/responses`
- Telegram bot CLI with long polling by default and an opt-in webhook server, configured through [`lino-arguments`](https://github.com/link-foundation/lino-arguments)
- human-readable Links Notation knowledge and dataset export through `lino-objects-codec`
- Docker-ready microservice
- GitHub Pages markdown chat demo backed by a Rust-generated WebAssembly worker

Project direction is tracked in [VISION.md](VISION.md), [GOALS.md](GOALS.md), and [NON-GOALS.md](NON-GOALS.md). The issue #12 synthesis is in [docs/case-studies/issue-12/README.md](docs/case-studies/issue-12/README.md).

## Quick Start

```bash
cargo run -- chat --prompt "Hi"
cargo run -- chat --prompt "Write me hello world program in Rust" --format chat
cargo run -- chat --prompt "What is 8% of $50?"
cargo run -- chat --prompt "Посчитай 1000 рублей в долларах"
cargo run -- dataset
rust-script scripts/mine-hive-mind-dataset.rs --plan
cargo run -- serve --host 127.0.0.1 --port 8080
TELEGRAM_BOT_TOKEN=123:abc cargo run -- telegram                       # long polling (default)
cargo run -- telegram --mode webhook --host 127.0.0.1 --port 8080      # webhook server (opt-in)
rust-script scripts/download-datasets.rs
experiments/verify-hello-world-examples.sh
```

Example API call:

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-symbolic-production","messages":[{"role":"user","content":"Hi"}]}'
```

Example Telegram webhook update:

```bash
curl -s http://127.0.0.1:8080/telegram/webhook \
  -H 'content-type: application/json' \
  -d '{"update_id":1,"message":{"message_id":7,"date":1,"chat":{"id":42,"type":"private"},"text":"Write me hello world program in Rust"}}'
```

Docker:

```bash
docker build -t formal-ai .
docker run --rm -p 8080:8080 formal-ai
```

The static demo lives in `src/web/index.html`. Serve it from a local web server or GitHub Pages so the WebAssembly worker can be fetched by the browser. The demo starts with a user greeting, renders markdown in messages, previews markdown input, and includes a randomized dialog mode for hello-world prompts across several programming languages. The companion connectivity diagnostics page lives in `src/web/tests/index.html` and is deployed at `/formal-ai/tests/`; it checks direct browser fetches, public knowledge APIs, iframe embeddability, and a configurable local `web-capture` proxy.

### Full-memory export and import

Every interface produces the same self-contained Links Notation document by default. In the browser, the **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — the entire seed (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, and the full append-only event log — so a single click is enough to reconstitute the session. **Import memory** auto-detects bundle vs legacy `demo_memory` files and surfaces migration suggestions when the imported seed version differs from the running app's. The CLI matches:

```bash
cargo run -- memory export --from memory.lino --path full.lino           # default: full bundle
cargo run -- memory export --from memory.lino --path events.lino --events-only  # legacy demo_memory
cargo run -- memory import --path full.lino --into memory.lino           # accepts either format
cargo run -- bundle export --path bundle.lino --memory memory.lino
cargo run -- bundle import --path bundle.lino --into memory.lino
```

The Rust library re-exports the same helpers — `export_memory_full`, `import_memory_full`, `suggest_memory_migrations`, `BundleInfo`, `ParsedBundle` — so embedders writing their own surface get the same defaults. The prefilled **Report issue** link records the dialog as a single compact `U:`/`A:` code block and points to [`docs/upload-memory.md`](docs/upload-memory.md) for attaching the full memory export (GitHub Gist or `.zip` workflow, plus redaction reminders) instead of repeating those instructions inline.

### Teaching behavior in chat

The chat surface can explain and modify behavior rules without leaving the dialog. Behavior is surfaced as a series of `When X then Y` (or `When X do Y`) statements grouped by topic, and the same grammar can also update the dialog:

```text
List behavior rules
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

### Webhook (opt-in)

```bash
cargo run -- telegram --mode webhook --host 127.0.0.1 --port 8080
# or equivalently for backwards compatibility:
cargo run -- serve --host 127.0.0.1 --port 8080
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

The engine normalizes a prompt, selects a deterministic symbolic rule, and returns the rule output with evidence link identifiers and indented Links Notation. Seed rules currently cover:

- greetings and polite follow-ups: `Hi`, `Hello`, `Hey`, `I am fine, thank you`, `thanks`
- hello world requests for Rust, Python, JavaScript, TypeScript, Go, and C
- open-ended software artifact requests such as extensions, plugins, bots, apps, and tools, first returning a Links Notation meaning record with a requirement graph, subtasks, delivery mode, approval gates, reasoning, and plan steps, then returning language-aware starter domain code after the user approves the plan
- calculator-parsable math, unit, currency, percentage, and datetime prompts through `link-calculator`, with the local arithmetic evaluator retained for unsupported word-operator and binary-modulo syntax
- URL requests such as `Navigate to github.com`, `fetch example.com`, and `Сделай запрос к google.com`; navigation prompts check CORS-readable frame-policy metadata and render an iframe only when `X-Frame-Options` and CSP `frame-ancestors` do not block embedding, while explicit fetch prompts attempt a browser `fetch()` first and use the same frame-policy check before any embedded fallback
- explicit web-search prompts such as `Search the web for Nikola Tesla` and `Найди в интернете Никола Тесла`; the browser demo uses the CORS-enabled Wikipedia search endpoint and returns ranked links
- merged definition prompts such as `Merge Wikipedia definitions of IIR`, which combine localized definition blocks for the same seed/Wikidata concept, deduplicate repeated facts, and cite every source language; use `--definition-fusion auto`, `FORMAL_AI_DEFINITION_FUSION=auto`, or the browser Settings control to make plain prompts like `What is IIR?` use the same fusion path
- behavior-rule inspection and dialog-local rule updates through `List behavior rules` (grouped by topic, each rendered as a `When X then Y` statement), `Show behavior rule unknown`, and the multilingual `When ... then ...` / `When ... do ...` / `When I say ... answer ...` grammar
- unknown prompts, which return a larger learnable-rule fallback with exact commands for inspecting rules, teaching the current dialog, exporting memory, or reporting a missing built-in rule

Hello-world answers include execution metadata. Rust, Python, JavaScript, Go, and C examples are compiled or syntax-checked and run by the issue-8 local verification harness with captured output. TypeScript is returned with an explicit warning because `tsc` is not configured in the current repository runtime.

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

See [REQUIREMENTS.md](REQUIREMENTS.md) for the cumulative requirement matrix (now alongside [VISION.md](VISION.md)) and [docs/case-studies/issue-1/README.md](docs/case-studies/issue-1/README.md) for the collected research and implementation plan.
