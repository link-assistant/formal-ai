---
bump: minor
---

### Added
- Added a TDD-style MVP test suite under `tests/unit/mvp/` that pins down the chat surface, code generation, multilingual chat, OpenAI compatibility, Telegram surface, links network, reasoning loop, source cache, agent isolation, translation-via-Links, network visualization, and transparent-state requirements drawn from `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `docs/REQUIREMENTS.md`. Tests describing not-yet-implemented MVP behavior are marked `#[ignore = "MVP-target: ..."]` so they document expectations without blocking CI; run them locally with `cargo test -- --include-ignored`.
