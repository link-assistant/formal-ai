# Requirements for Issue #1

This document turns issue [#1](https://github.com/link-assistant/formal-ai/issues/1) into explicit implementation requirements.

| ID | Requirement | Prototype status |
| --- | --- | --- |
| R1 | Implement a formal / symbolic AI instead of a GPU-backed neural network. | Implemented as deterministic Rust rules in `FormalAiEngine`. |
| R2 | Produce an API compatible with OpenAI Chat Completions. | Implemented through `ChatCompletionRequest`, `ChatCompletion`, CLI JSON output, and `POST /v1/chat/completions`. |
| R3 | Produce an API compatible with OpenAI Responses. | Implemented through `ResponsesRequest`, `ResponseObject`, CLI JSON output, and `POST /v1/responses`. |
| R4 | Support connection from agentic tools that can call HTTP APIs. | Implemented as a JSON HTTP server with CORS headers and `/health`. |
| R5 | Avoid neural-network inference and GPU requirements. | Implemented; the crate uses only rule matching and Links Notation encoding. |
| R6 | Explore Bayesian networks, Markov chains, and similar symbolic/probabilistic methods. | Documented as next-stage architecture; the prototype has deterministic rules only. |
| R7 | Build on Links Notation and link-style references. | Implemented by exporting rules and answers through `lino-objects-codec`, with stable evidence link IDs. |
| R8 | Keep datasets in the repository where practical. | Implemented as seed rules in Rust and documented data sources in the case study. |
| R9 | Answer simple greetings such as `Hi` and `Hello`. | Implemented and tested. |
| R10 | Answer `Write me hello world program in Rust`. | Implemented and tested. |
| R11 | Provide a Rust library. | Implemented by the `formal_ai` library crate. |
| R12 | Provide a CLI. | Implemented by the `formal-ai` binary. |
| R13 | Provide an API server from the CLI. | Implemented by `formal-ai serve`. |
| R14 | Provide a Docker-ready microservice. | Implemented with the root `Dockerfile`. |
| R15 | Provide a GitHub Pages React demo chat page. | Implemented in `docs/demo`. |
| R16 | Use a Rust WebAssembly worker for the demo. | Implemented by `docs/demo/formal_ai_worker.wasm`, built from `docs/demo/wasm-worker/src/lib.rs`. |
| R17 | Provide a desktop application path similar to `vk-bot-desktop`. | Documented as a future wrapper around the same HTTP/library boundary; not expanded into a native app in this prototype PR. |
| R18 | Use formal reasoning / theorem proving components where appropriate. | Documented as a later integration point for `relative-meta-logic`. |
| R19 | Store issue research under `docs/case-studies/issue-1`. | Implemented with raw GitHub data and a case-study README. |
| R20 | Add tests for minimum API/CLI/UI interactions. | Implemented for library, protocol, server handler, CLI, and manually verified UI. |

## Current Scope Boundary

This PR is a proof of concept, not a full universal problem solver. The implemented core proves that the repository can expose OpenAI-shaped interfaces while returning symbolic, inspectable, Links Notation-backed answers. The remaining large research items should be split into focused follow-up issues after this baseline lands.
