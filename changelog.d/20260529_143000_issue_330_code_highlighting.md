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

### Changed
- Reorganized the coding-task support into a cohesive `src/coding/` module (`catalog.rs` for the language/task/template catalog and lookups, `guidance.rs` for the novice "How it works"/"How to test" guidance), replacing the misleadingly named `src/engine_hello_world.rs` and `src/engine_program_guidance.rs`. The module covers general coding tasks across 11 languages, not only hello-world (issue #330).

