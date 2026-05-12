---
bump: minor
---

### Added
- Added the formal-ai proof-of-concept library, CLI, OpenAI-compatible JSON API server, Docker packaging, and GitHub Pages demo.
- Added a markdown-capable chat demo with randomized dialog mode, greeting-first examples, and multi-language hello-world prompts.
- Added Rust-script dataset generation into human-readable Links Notation `.lino` files under `data/`.
- Added Playwright e2e test suite under `tests/e2e/` covering page load, initial messages, quick prompts, chat interactions, demo mode toggle, trace panel, and preview mode.
- Added `test-e2e-local` CI job that serves the demo with a local HTTP server and runs Playwright tests on every PR.
- Added `test-e2e-pages` CI job that runs Playwright tests against the live GitHub Pages URL after each deployment.

### Changed
- Expanded deterministic hello-world responses from Rust-only to Rust, Python, JavaScript, TypeScript, Go, and C.
- Extended repository file-size checks to enforce the 1500-line limit for `.lino` dataset chunks.
- Fixed GitHub Pages deployment to publish the React chat demo from `docs/demo/` instead of Rust API documentation.
