---
bump: minor
---

### Added
- Syntax highlighting for chat code blocks via a dependency-free, highlight.js-compatible tokenizer (`src/web/syntax-highlight.js`) covering rust, python, javascript/typescript, go, c, cpp, java, csharp, ruby, bash, and json (issue #330).
- A copy button on every rendered code block that copies the raw source to the clipboard with "Copied!" feedback.
- A "Copy as Markdown" button on each chat message that copies the whole message content (Markdown fences preserved).
- Localized strings for the new copy buttons in all four locales (en/ru/zh/hi).
- End-to-end Playwright tests proving highlighting renders and both copy buttons work against a freshly built `src/web`.
- Runnable example (`examples/issue-330-code-highlighting/`) with run/test instructions and a deep case study in `docs/case-studies/issue-330/`.
- Code answers now teach a novice: every generated program is followed by a localized "How it works" explanation and step-by-step "How to test it yourself" instructions (install the toolchain, save the file, compile, run, compare the output) in en/ru/hi/zh (issue #330).
- When the dialog already walked the user through running code, a follow-up code edit omits the verbose setup steps and shows a concise "test it the same way" note instead, detected from prior assistant turns in the conversation history.
- Four new deterministic coding tasks broaden the catalog beyond hello-world and list-files — FizzBuzz, factorial of 5, string reversal, and the sum from 1 to 10 — each with a verified fixed output and templates for all ten supported languages, reachable in en/ru/hi/zh (issue #330).
- The JavaScript demo worker (`src/web/formal_ai_worker.js`) now mirrors the full Rust catalog (the four new tasks, all ten languages with their setup/run/check metadata) and the novice "How it works"/"How to test" guidance, keeping the in-browser engine in lockstep with the Rust engine.

### Changed
- Reorganized the coding-task support into a cohesive `src/coding/` module — a `catalog/` submodule (`types.rs` for the records, `languages.rs`/`tasks.rs` for the catalog tables, `templates_core.rs`/`templates_extended.rs` for the per-language templates, and `mod.rs` for the lookups) plus `guidance.rs` for the novice "How it works"/"How to test" guidance — replacing the misleadingly named `src/engine_hello_world.rs` and `src/engine_program_guidance.rs`. The module covers general coding tasks across 11 languages, not only hello-world, and every file stays well under the repository's per-file line limit (issue #330).

