# Issue 1 Case Study: Formal AI Proof of Concept

## Summary

Issue [#1](https://github.com/link-assistant/formal-ai/issues/1) requests a non-neural formal AI that can be consumed through OpenAI-style APIs while remaining inspectable, symbolic, and compatible with Links Notation workflows.

This PR establishes the baseline:

- a Rust library and CLI named `formal-ai`
- deterministic symbolic prompt handling for greetings and Rust hello world requests
- OpenAI-shaped Chat Completions and Responses objects
- a small HTTP server for `/v1/chat/completions`, `/v1/responses`, `/v1/models`, and `/health`
- Links Notation export through `lino-objects-codec`
- Docker packaging
- a GitHub Pages React chat demo with a Rust-generated WebAssembly worker

## Collected Data

Raw GitHub data is stored in `raw-data/`:

- `issue-1.json`: issue details
- `issue-1-comments.json`: issue comments
- `pr-2.json`: prepared PR metadata
- `pr-2-conversation-comments.json`: PR conversation comments
- `pr-2-review-comments.json`: inline PR review comments
- `pr-2-reviews.json`: PR reviews

There were no issue comments or PR comments at the time of collection on 2026-05-12.

## Online Research

Sources checked for the baseline architecture:

- [OpenAI Chat Completions API reference](https://platform.openai.com/docs/api-reference/chat/create) for the request/response shape.
- [OpenAI Responses API reference](https://platform.openai.com/docs/api-reference/responses/create) for the modern response object shape.
- [link-foundation/links-notation](https://github.com/link-foundation/links-notation), described as a way to describe data using references and links.
- [link-foundation/link-cli](https://github.com/link-foundation/link-cli), described as a CLI tool to manipulate links.
- [link-foundation/lino-objects-codec](https://github.com/link-foundation/lino-objects-codec), used here to encode objects as Links Notation.
- [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic), a later reasoning integration candidate.
- [linksplatform/doublets-rs](https://github.com/linksplatform/doublets-rs), a later storage integration candidate.
- [Hello World Collection](http://helloworldcollection.de), used as the seed idea for language-specific hello world examples.

Crates.io checks found `links-notation = 0.13.0` and `lino-objects-codec = 0.2.1`; this prototype uses `lino-objects-codec` directly.

## Requirements Extracted

The full requirement matrix is maintained in [`../../REQUIREMENTS.md`](../../REQUIREMENTS.md). The main groups are:

- API compatibility: Chat Completions and Responses
- symbolic execution: deterministic rules, no neural network inference
- link-native representation: stable link IDs and Links Notation exports
- product surfaces: library, CLI, server, Docker, GitHub Pages demo
- research roadmap: datasets, Markov/Bayesian methods, formal reasoning, desktop app

## Design

The prototype keeps the runtime intentionally small:

1. Normalize the prompt to lower-case ASCII words.
2. Select a symbolic rule by exact greeting or Rust hello world token match.
3. Return the answer, confidence, evidence link IDs, and a Links Notation encoding.
4. Wrap the same answer into Chat Completions or Responses shapes.

This keeps every answer traceable. For example, a greeting response carries evidence links such as `response:greeting` and `intent:greeting`.

## Solution Plan by Requirement Group

API compatibility:

- Implement typed Rust DTOs for the fields used by common OpenAI clients.
- Keep unsupported fields ignored rather than rejected unless the request body is invalid JSON.
- Add streaming later as a separate protocol feature.

Symbolic AI:

- Start with deterministic symbolic rules.
- Add Markov or Bayesian scoring only after a link-native dataset exists.
- Keep random behavior optional and seedable for reproducibility.

Links data:

- Keep stable link identifiers in the response evidence.
- Encode exported knowledge and answer traces with `lino-objects-codec`.
- Add direct `link-cli` import/export tests once a stable library boundary is available.

Datasets:

- Use small repository seed facts first.
- Convert larger public datasets in follow-up PRs with scripts under `experiments/` before promoting them to `examples/`.
- Keep generated datasets reviewable and chunked.

UI and deployment:

- Use the Rust HTTP server as the agent and microservice boundary.
- Keep the GitHub Pages demo static so it can run without a backend.
- Use the same Rust logic direction in the WebAssembly worker and native library.

Desktop:

- Treat desktop as a wrapper around the same library/server boundary.
- A native desktop shell should be a follow-up after the API contract stabilizes.

## Verification

Local tests added for this PR:

- `formal_ai::greeting_prompt_returns_symbolic_greeting`
- `formal_ai::rust_hello_world_prompt_returns_code_block`
- `formal_ai::chat_completion_has_openai_compatible_shape`
- `formal_ai::responses_api_shape_contains_output_text`
- `formal_ai::knowledge_export_is_valid_links_notation`
- `formal_ai::server_handler_supports_chat_completions_route`
- `formal_ai_cli::cli_chat_command_prints_text_response`
- `formal_ai_cli::cli_chat_command_can_emit_chat_completion_json`

The first test run failed before implementation because the repository still exposed only the template sum app. After the implementation, the targeted unit and CLI tests pass.
