# formal-ai

Formal AI is a Rust proof of concept for a symbolic, deterministic assistant that exposes OpenAI-shaped interfaces without neural-network inference.

The current prototype is intentionally small. It proves the surface area requested in issue #1:

- library API for symbolic prompt handling
- CLI chat command
- HTTP API server with `/v1/chat/completions` and `/v1/responses`
- human-readable Links Notation knowledge and dataset export through `lino-objects-codec`
- Docker-ready microservice
- GitHub Pages markdown chat demo backed by a Rust-generated WebAssembly worker

## Quick Start

```bash
cargo run -- chat --prompt "Hi"
cargo run -- chat --prompt "Write me hello world program in Rust" --format chat
cargo run -- dataset
cargo run -- serve --host 127.0.0.1 --port 8080
rust-script scripts/download-datasets.rs
```

Example API call:

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"formal-symbolic-poc","messages":[{"role":"user","content":"Hi"}]}'
```

Docker:

```bash
docker build -t formal-ai .
docker run --rm -p 8080:8080 formal-ai
```

The static demo lives in `docs/demo/index.html`. Serve it from a local web server or GitHub Pages so the WebAssembly worker can be fetched by the browser. The demo starts with a user greeting, renders markdown in messages, previews markdown input, and includes a randomized dialog mode for hello-world prompts across several programming languages.

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
};

let completion = create_chat_completion(&request);
assert_eq!(
    completion.choices[0].message.content.plain_text(),
    "Hi, how may I help you?"
);
```

## Current Symbolic Behavior

The engine normalizes a prompt, selects a deterministic symbolic rule, and returns the rule output with evidence link identifiers and indented Links Notation. Seed rules currently cover:

- greetings: `Hi`, `Hello`, `Hey`
- hello world requests for Rust, Python, JavaScript, TypeScript, Go, and C
- unknown prompts, which return an explicit learnable-rule fallback

No GPU, neural network, remote model, or random sampling is used.

## Dataset Seeds

Issue #1 source indexes and seed prompts are stored as indented Links Notation in `data/`.

```bash
rust-script scripts/download-datasets.rs
rust-script scripts/check-file-size.rs
```

The generator writes source, greeting, hello-world, and demo-dialog records. `.lino` files are kept below 1500 lines and validated by the unit tests.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
rust-script scripts/check-file-size.rs
```

See [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) for the issue #1 requirement matrix and [docs/case-studies/issue-1/README.md](docs/case-studies/issue-1/README.md) for the collected research and implementation plan.
